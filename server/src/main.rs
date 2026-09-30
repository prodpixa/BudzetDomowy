//! Budżet Domowy – serwer webowy.
//!
//!   budzet-server [serve]         uruchom serwer (domyślnie)
//!   budzet-server hash-password   wczytaj hasło ze stdin i wypisz hash argon2id do .env
//!   budzet-server backup [opis]   zrób ręczną kopię bazy (np. przed wdrożeniem)
//!   budzet-server healthcheck     sprawdź, czy serwer odpowiada (dla Dockera)

mod api;
mod auth;
mod backup;
mod commands;
mod db;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use auth::{AuthConfig, Throttle, User};

struct Config {
    data_dir: PathBuf,
    static_dir: PathBuf,
    bind: String,
    port: u16,
    backup_keep: usize,
    backup_hour: u32,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.into())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> Result<T, String> {
    match std::env::var(key).ok().filter(|v| !v.trim().is_empty()) {
        Some(v) => v.trim().parse().map_err(|_| format!("{key}: nieprawidłowa wartość „{v}”")),
        None => Ok(default),
    }
}

fn load_config() -> Result<Config, String> {
    Ok(Config {
        data_dir: env_or("BUDZET_DATA_DIR", "./data").into(),
        static_dir: env_or("BUDZET_STATIC_DIR", "../build").into(),
        bind: env_or("BUDZET_BIND", "0.0.0.0"),
        port: env_parse("BUDZET_PORT", 8420)?,
        backup_keep: env_parse("BUDZET_BACKUP_KEEP", 14)?,
        backup_hour: env_parse("BUDZET_BACKUP_HOUR", 3)?,
    })
}

/// Konta: BUDZET_USER1_NAME + BUDZET_USER1_PASSWORD_HASH, BUDZET_USER2_…
fn load_auth() -> Result<AuthConfig, String> {
    let mut users = Vec::new();
    for i in 1..=9 {
        let name = env_or(&format!("BUDZET_USER{i}_NAME"), "");
        let hash = env_or(&format!("BUDZET_USER{i}_PASSWORD_HASH"), "");
        match (name.is_empty(), hash.is_empty()) {
            (true, true) => continue,
            (false, false) => {
                auth::check_hash_format(&hash)
                    .map_err(|e| format!("BUDZET_USER{i}_PASSWORD_HASH: {e}"))?;
                users.push(User { name: name.trim().to_string(), password_hash: hash });
            }
            _ => return Err(format!("BUDZET_USER{i}: podaj zarówno NAME, jak i PASSWORD_HASH")),
        }
    }
    if users.is_empty() {
        return Err("Brak kont. Ustaw BUDZET_USER1_NAME i BUDZET_USER1_PASSWORD_HASH w pliku .env".into());
    }
    Ok(AuthConfig {
        users,
        session_days: env_parse("BUDZET_SESSION_DAYS", 30)?,
        cookie_secure: env_parse("BUDZET_COOKIE_SECURE", false)?,
    })
}

fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("serve") => serve(),
        Some("hash-password") => hash_password_cmd(),
        Some("backup") => backup_cmd(args.get(1).map(String::as_str)),
        Some("healthcheck") => healthcheck(),
        Some(other) => Err(format!("Nieznane polecenie: {other}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Błąd: {e}");
            ExitCode::FAILURE
        }
    }
}

fn hash_password_cmd() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).map_err(|e| e.to_string())?;
    let password = input.trim_end_matches(['\r', '\n']);
    if password.chars().count() < 8 {
        return Err("Hasło musi mieć co najmniej 8 znaków".into());
    }
    println!("{}", auth::hash_password(password)?);
    Ok(())
}

fn backup_cmd(tag: Option<&str>) -> Result<(), String> {
    let cfg = load_config()?;
    let db_path = cfg.data_dir.join("budzet.db");
    if !db_path.exists() {
        return Err(format!("Nie ma bazy {}", db_path.display()));
    }
    let conn = db::open(&db_path).map_err(|e| e.to_string())?;
    let path = backup::manual(&conn, &cfg.data_dir.join("backups"), tag)?;
    println!("{}", path.display());
    Ok(())
}

fn healthcheck() -> Result<(), String> {
    let port: u16 = env_parse("BUDZET_PORT", 8420)?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(3)).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(Duration::from_secs(3))).ok();
    s.write_all(b"GET /api/health HTTP/1.0\r\nHost: localhost\r\n\r\n").map_err(|e| e.to_string())?;
    let mut buf = [0u8; 64];
    let n = s.read(&mut buf).map_err(|e| e.to_string())?;
    let head = String::from_utf8_lossy(&buf[..n]);
    if head.starts_with("HTTP/1.1 200") || head.starts_with("HTTP/1.0 200") {
        Ok(())
    } else {
        Err(format!("Nieoczekiwana odpowiedź: {head}"))
    }
}

fn serve() -> Result<(), String> {
    let cfg = load_config()?;
    let auth = load_auth()?;
    std::fs::create_dir_all(&cfg.data_dir)
        .map_err(|e| format!("Nie mogę utworzyć {}: {e}", cfg.data_dir.display()))?;
    if !cfg.static_dir.join("index.html").exists() {
        return Err(format!("Brak zbudowanego frontendu w {}", cfg.static_dir.display()));
    }

    let conn = db::open(&cfg.data_dir.join("budzet.db")).map_err(|e| e.to_string())?;
    let removed = auth::purge_expired(&conn)?;
    tracing::info!(
        "baza: {}, konta: {}, wygasłe sesje usunięte: {removed}",
        cfg.data_dir.join("budzet.db").display(),
        auth.users.iter().map(|u| u.name.as_str()).collect::<Vec<_>>().join(", ")
    );

    let state = api::AppState {
        db: Arc::new(Mutex::new(conn)),
        auth: Arc::new(auth),
        throttle: Arc::new(Throttle::default()),
        data_dir: cfg.data_dir.clone(),
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2) // Raspberry Pi – dwa wątki w zupełności wystarczą
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(async move {
        spawn_backup_task(&state, &cfg);
        let app = api::router(state.clone(), cfg.static_dir.clone());
        let addr = format!("{}:{}", cfg.bind, cfg.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Nie mogę nasłuchiwać na {addr}: {e}"))?;
        tracing::info!("Budżet Domowy działa na http://{addr}");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| e.to_string())?;

        // Przy zamknięciu zapisujemy wszystko z dziennika WAL do głównego pliku bazy.
        if let Ok(conn) = state.db.lock() {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
        tracing::info!("zatrzymano");
        Ok(())
    })
}

/// Co 10 minut: dzisiejsza kopia (jeśli już pora i jeszcze jej nie ma) + sprzątanie sesji.
fn spawn_backup_task(state: &api::AppState, cfg: &Config) {
    let db = state.db.clone();
    let dir = cfg.data_dir.join("backups");
    let (keep, hour) = (cfg.backup_keep, cfg.backup_hour);
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(600));
        loop {
            tick.tick().await;
            let (db, dir) = (db.clone(), dir.clone());
            let res = tokio::task::spawn_blocking(move || {
                let conn = db.lock().map_err(|e| e.to_string())?;
                let _ = auth::purge_expired(&conn);
                backup::daily_if_due(&conn, &dir, keep, hour)
            })
            .await;
            match res {
                Ok(Ok(Some(path))) => tracing::info!("kopia zapasowa: {}", path.display()),
                Ok(Ok(None)) => {}
                Ok(Err(e)) => tracing::error!("kopia zapasowa nie powiodła się: {e}"),
                Err(e) => tracing::error!("zadanie kopii zapasowej: {e}"),
            }
        }
    });
}

/// `docker stop` wysyła SIGTERM – kończymy spokojnie, zamiast urywać zapis w połowie.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
}
