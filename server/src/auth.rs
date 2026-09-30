//! Logowanie: konta z pliku .env (hash argon2id), sesje w bazie, ochrona przed zgadywaniem haseł.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::commands::{err, Res};

pub const COOKIE_NAME: &str = "budzet_session";

/// Po tylu nieudanych próbach konto jest chwilowo blokowane.
const MAX_FAILURES: u32 = 5;
const LOCKOUT: Duration = Duration::from_secs(60);

pub struct User {
    pub name: String,
    pub password_hash: String,
}

pub struct AuthConfig {
    pub users: Vec<User>,
    pub session_days: i64,
    /// `Secure` w ciasteczku – tylko gdy aplikacja działa za HTTPS.
    pub cookie_secure: bool,
}

pub fn hash_password(password: &str) -> Res<String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(err)
}

pub fn check_hash_format(hash: &str) -> Res<()> {
    PasswordHash::new(hash).map(|_| ()).map_err(|e| format!("Nieprawidłowy hash hasła: {e}"))
}

fn verify(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

impl AuthConfig {
    /// Zwraca kanoniczną nazwę użytkownika, jeśli login i hasło są poprawne.
    /// Dla nieistniejącego loginu i tak liczymy hash, żeby czas odpowiedzi nie zdradzał,
    /// które loginy istnieją.
    pub fn authenticate(&self, username: &str, password: &str) -> Option<String> {
        let user = self.users.iter().find(|u| u.name.eq_ignore_ascii_case(username.trim()));
        match user {
            Some(u) => verify(&u.password_hash, password).then(|| u.name.clone()),
            None => {
                let _ = verify(&self.users[0].password_hash, "\u{0}nie-istniejący-użytkownik");
                None
            }
        }
    }
}

// ---------- ochrona przed zgadywaniem ----------

#[derive(Default)]
pub struct Throttle {
    failures: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Throttle {
    /// Ile sekund trzeba jeszcze poczekać (None = można próbować).
    pub fn locked_for(&self, username: &str) -> Option<u64> {
        let map = self.failures.lock().unwrap();
        let (count, last) = map.get(&username.trim().to_lowercase())?;
        if *count >= MAX_FAILURES && last.elapsed() < LOCKOUT {
            Some((LOCKOUT - last.elapsed()).as_secs().max(1))
        } else {
            None
        }
    }

    pub fn failed(&self, username: &str) {
        let mut map = self.failures.lock().unwrap();
        if map.len() > 1000 {
            map.clear(); // zabezpieczenie przed zapchaniem pamięci losowymi loginami
        }
        let entry = map.entry(username.trim().to_lowercase()).or_insert((0, Instant::now()));
        if entry.1.elapsed() >= LOCKOUT {
            entry.0 = 0; // po odczekaniu liczymy od nowa
        }
        entry.0 += 1;
        entry.1 = Instant::now();
    }

    pub fn succeeded(&self, username: &str) {
        self.failures.lock().unwrap().remove(&username.trim().to_lowercase());
    }
}

// ---------- sesje ----------

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn token_hash(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

/// Tworzy sesję i zwraca token do ciasteczka (w bazie ląduje tylko jego skrót).
pub fn create_session(conn: &Connection, username: &str, days: i64) -> Res<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(err)?;
    let token = to_hex(&bytes);
    let t = now();
    conn.execute(
        "INSERT INTO sessions (token_hash, username, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
        params![token_hash(&token), username, t, t + days * 86400],
    )
    .map_err(err)?;
    Ok(token)
}

/// Zwraca użytkownika sesji. Sesja jest „przesuwana”: wygasa po `days` dniach
/// od ostatniego użycia (przedłużamy najwyżej raz na dobę, żeby nie pisać przy każdym żądaniu).
pub fn session_user(conn: &Connection, token: &str, days: i64) -> Res<Option<String>> {
    let hash = token_hash(token);
    let t = now();
    let row: Option<(String, i64)> = conn
        .query_row(
            "SELECT username, expires_at FROM sessions WHERE token_hash = ?1 AND expires_at > ?2",
            params![hash, t],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((username, expires_at)) = row else { return Ok(None) };
    let full = t + days * 86400;
    if full - expires_at > 86400 {
        conn.execute("UPDATE sessions SET expires_at = ?1 WHERE token_hash = ?2", params![full, hash])
            .map_err(err)?;
    }
    Ok(Some(username))
}

pub fn delete_session(conn: &Connection, token: &str) -> Res<()> {
    conn.execute("DELETE FROM sessions WHERE token_hash = ?1", [token_hash(token)])
        .map_err(err)?;
    Ok(())
}

pub fn purge_expired(conn: &Connection) -> Res<usize> {
    conn.execute("DELETE FROM sessions WHERE expires_at <= ?1", [now()]).map_err(err)
}

/// Wyciąga token sesji z nagłówka Cookie.
pub fn token_from_cookie(header: &str) -> Option<&str> {
    header
        .split(';')
        .filter_map(|p| p.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE_NAME)
        .map(|(_, v)| v)
        .filter(|v| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Nagłówek Set-Cookie. HttpOnly: JavaScript nie odczyta tokenu (ochrona przed XSS).
/// SameSite=Strict: przeglądarka nie wyśle ciasteczka z cudzej strony (ochrona przed CSRF).
pub fn session_cookie(token: &str, secure: bool) -> String {
    format!(
        "{COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=34560000{}",
        if secure { "; Secure" } else { "" }
    )
}

pub fn clear_cookie(secure: bool) -> String {
    format!(
        "{COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
        if secure { "; Secure" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn config() -> AuthConfig {
        AuthConfig {
            users: vec![
                User { name: "Daniel".into(), password_hash: hash_password("tajne1").unwrap() },
                User { name: "Ola".into(), password_hash: hash_password("tajne2").unwrap() },
            ],
            session_days: 30,
            cookie_secure: false,
        }
    }

    #[test]
    fn login_checks_password() {
        let c = config();
        assert_eq!(c.authenticate("daniel", "tajne1").as_deref(), Some("Daniel"));
        assert_eq!(c.authenticate("Ola", "tajne2").as_deref(), Some("Ola"));
        assert_eq!(c.authenticate("Ola", "tajne1"), None);
        assert_eq!(c.authenticate("ktoś", "tajne1"), None);
    }

    #[test]
    fn sessions_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        let token = create_session(&conn, "Ola", 30).unwrap();
        assert_eq!(session_user(&conn, &token, 30).unwrap().as_deref(), Some("Ola"));
        // w bazie nie ma jawnego tokenu
        let stored: String =
            conn.query_row("SELECT token_hash FROM sessions", [], |r| r.get(0)).unwrap();
        assert_ne!(stored, token);
        delete_session(&conn, &token).unwrap();
        assert_eq!(session_user(&conn, &token, 30).unwrap(), None);
    }

    #[test]
    fn throttle_locks_after_failures() {
        let t = Throttle::default();
        for _ in 0..MAX_FAILURES {
            assert!(t.locked_for("Ola").is_none());
            t.failed("Ola");
        }
        assert!(t.locked_for("ola").is_some());
        t.succeeded("OLA");
        assert!(t.locked_for("Ola").is_none());
    }

    #[test]
    fn cookie_parsing() {
        let tok = "a".repeat(64);
        assert_eq!(token_from_cookie(&format!("x=1; {COOKIE_NAME}={tok}")), Some(tok.as_str()));
        assert_eq!(token_from_cookie(&format!("{COOKIE_NAME}=zly")), None);
    }
}
