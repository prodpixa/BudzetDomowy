//! Serwer HTTP: API w JSON pod /api, logowanie i pliki frontendu.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::auth::{self, AuthConfig, Throttle};
use crate::backup;
use crate::commands::{self, Res};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub auth: Arc<AuthConfig>,
    pub throttle: Arc<Throttle>,
    pub data_dir: PathBuf,
}

impl AppState {
    /// Operacje na SQLite są blokujące – wykonujemy je poza wątkami obsługującymi sieć.
    async fn with_db<T, F>(&self, f: F) -> Result<T, ApiError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Res<T> + Send + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.lock().map_err(|_| "Baza danych jest niedostępna".to_string())?;
            f(&mut conn)
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::bad_request)
    }
}

// ---------- błędy ----------

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: String) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message }
    }
    fn internal(message: String) -> Self {
        tracing::error!("{message}");
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: "Błąd serwera".into() }
    }
    fn unauthorized() -> Self {
        Self { status: StatusCode::UNAUTHORIZED, message: "Zaloguj się".into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

// ---------- komendy ----------

fn to_json<T: serde::Serialize>(v: T) -> Res<Value> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

/// Dla każdej komendy: struktura argumentów (jak wysyła frontend) + wywołanie funkcji z `commands`.
macro_rules! dispatch {
    ($conn:ident, $name:expr, $args:expr; $( $cmd:ident ( $($arg:ident : $ty:ty),* ) ),* $(,)?) => {
        match $name {
            $( stringify!($cmd) => {
                #[derive(Deserialize)]
                struct Args { $($arg: $ty),* }
                let a: Args = serde_json::from_value($args)
                    .map_err(|e| format!("Nieprawidłowe dane: {e}"))?;
                let _ = &a;
                to_json(commands::$cmd($conn, $(a.$arg),*)?)
            } )*
            other => Err(format!("Nieznana operacja: {other}")),
        }
    };
}

fn run_command(conn: &Connection, name: &str, args: Value) -> Res<Value> {
    use commands::{Category, Expense, Income, Planned, Source, Voucher};
    dispatch!(conn, name, args;
        list_categories(),
        save_category(item: Category),
        delete_category(id: i64),
        list_sources(),
        save_source(item: Source),
        delete_source(id: i64),
        list_expenses(month: String),
        save_expense(item: Expense),
        delete_expense(id: i64),
        list_incomes(month: String),
        save_income(item: Income),
        delete_income(id: i64),
        list_planned(month: String, kind: String),
        save_planned(item: Planned),
        set_planned_done(id: i64, done: bool),
        delete_planned(id: i64),
        copy_planned_from_previous(month: String, kind: String),
        list_vouchers(month: String),
        save_voucher(item: Voucher),
        delete_voucher(id: i64),
        voucher_summary(month: String),
        month_summary(month: String),
        month_trend(month: String, count: u32),
        get_opening_balance(),
        set_opening_balance(amount: i64),
    )
}

async fn command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(args): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let args = if args.is_null() { json!({}) } else { args };
    state.with_db(move |conn| run_command(conn, &name, args)).await.map(Json)
}

// ---------- logowanie ----------

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> Response {
    if let Some(secs) = state.throttle.locked_for(&req.username) {
        let msg = format!("Za dużo nieudanych prób. Spróbuj ponownie za {secs} s.");
        return (StatusCode::TOO_MANY_REQUESTS, Json(json!({ "error": msg }))).into_response();
    }

    // argon2 celowo zajmuje procesor na ~0,1 s – nie blokujemy nim obsługi sieci.
    let cfg = state.auth.clone();
    let (u, p) = (req.username.clone(), req.password);
    let user = tokio::task::spawn_blocking(move || cfg.authenticate(&u, &p)).await.ok().flatten();

    let Some(user) = user else {
        state.throttle.failed(&req.username);
        tracing::warn!("nieudane logowanie: {:?}", req.username);
        let msg = "Nieprawidłowy login lub hasło";
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": msg }))).into_response();
    };
    state.throttle.succeeded(&req.username);

    let days = state.auth.session_days;
    let name = user.clone();
    match state.with_db(move |conn| auth::create_session(conn, &name, days)).await {
        Ok(token) => {
            tracing::info!("zalogowano: {user}");
            let cookie = auth::session_cookie(&token, state.auth.cookie_secure);
            ([(header::SET_COOKIE, cookie)], Json(json!({ "username": user }))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = cookie_token(&headers) {
        let _ = state.with_db(move |conn| auth::delete_session(conn, &token)).await;
    }
    let cookie = auth::clear_cookie(state.auth.cookie_secure);
    ([(header::SET_COOKIE, cookie)], Json(json!({}))).into_response()
}

#[derive(Clone)]
struct CurrentUser(String);

async fn me(axum::Extension(user): axum::Extension<CurrentUser>) -> Json<Value> {
    Json(json!({ "username": user.0 }))
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| auth::token_from_cookie(v).map(str::to_string))
}

/// Wpuszcza dalej tylko zalogowanych; nazwę użytkownika dokłada do żądania.
async fn require_session(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = cookie_token(req.headers()).ok_or_else(ApiError::unauthorized)?;
    let days = state.auth.session_days;
    let user = state
        .with_db(move |conn| auth::session_user(conn, &token, days))
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    req.extensions_mut().insert(CurrentUser(user));
    Ok(next.run(req).await)
}

// ---------- pliki: eksport, kopia, przywracanie ----------

fn download(bytes: Vec<u8>, content_type: &'static str, filename: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
                    .unwrap_or(HeaderValue::from_static("attachment")),
            ),
        ],
        bytes,
    )
        .into_response()
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

async fn export_zip(State(state): State<AppState>) -> Result<Response, ApiError> {
    let bytes = state.with_db(|conn| commands::export_zip(conn)).await?;
    Ok(download(bytes, "application/zip", format!("budzet-eksport-{}.zip", today())))
}

async fn backup_download(State(state): State<AppState>) -> Result<Response, ApiError> {
    let tmp = state.data_dir.join(format!("pobranie-{}.db", std::process::id()));
    let path = tmp.clone();
    state.with_db(move |conn| backup::backup_to(conn, &path)).await?;
    let bytes = tokio::fs::read(&tmp).await.map_err(|e| ApiError::internal(e.to_string()))?;
    let _ = tokio::fs::remove_file(&tmp).await;
    Ok(download(bytes, "application/vnd.sqlite3", format!("budzet-kopia-{}.db", today())))
}

async fn restore_upload(State(state): State<AppState>, body: Bytes) -> Result<Json<Value>, ApiError> {
    let upload = state.data_dir.join(format!("wgrana-{}.db", std::process::id()));
    tokio::fs::write(&upload, &body).await.map_err(|e| ApiError::internal(e.to_string()))?;
    let backups = state.data_dir.join("backups");
    let path = upload.clone();
    let result = state
        .with_db(move |conn| {
            commands::validate_backup_file(&path)?;
            // Zanim nadpiszemy dane – kopia obecnego stanu (na wypadek pomyłki).
            backup::manual(conn, &backups, Some("przed-przywroceniem"))?;
            commands::restore_from(conn, &path)
        })
        .await;
    let _ = tokio::fs::remove_file(&upload).await;
    result?;
    tracing::info!("przywrócono bazę z wgranej kopii");
    Ok(Json(json!({})))
}

// ---------- pliki frontendu ----------

/// Pliki z /_app/immutable mają hash w nazwie – można je trzymać w cache „na zawsze”.
/// index.html zawsze sprawdzamy od nowa, żeby po aktualizacji od razu była nowa wersja.
async fn cache_headers(req: Request, next: Next) -> Response {
    let immutable = req.uri().path().starts_with("/_app/immutable/");
    let mut res = next.run(req).await;
    let value = if immutable { "public, max-age=31536000, immutable" } else { "no-cache" };
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    res
}

async fn health() -> &'static str {
    "ok"
}

pub fn router(state: AppState, static_dir: PathBuf) -> Router {
    let protected = Router::new()
        .route("/api/auth/me", get(me))
        .route("/api/export.zip", get(export_zip))
        .route("/api/backup.db", get(backup_download))
        .route(
            "/api/restore",
            post(restore_upload).layer(DefaultBodyLimit::max(64 * 1024 * 1024)),
        )
        .route("/api/{name}", post(command))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_session));

    let static_files = ServeDir::new(&static_dir)
        .fallback(ServeFile::new(static_dir.join("index.html")));

    // Podstawowe nagłówki bezpieczeństwa dla każdej odpowiedzi.
    let sec = |name: &'static str, value: &'static str| {
        SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static(name),
            HeaderValue::from_static(value),
        )
    };

    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .merge(protected)
        .fallback_service(Router::new().fallback_service(static_files).layer(middleware::from_fn(cache_headers)))
        .layer(CompressionLayer::new())
        .layer(sec("x-content-type-options", "nosniff"))
        .layer(sec("x-frame-options", "DENY"))
        .layer(sec("referrer-policy", "no-referrer"))
        .layer(sec(
            "content-security-policy",
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'",
        ))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{hash_password, User};
    use crate::db;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        let dir = std::env::temp_dir().join(format!("budzet-api-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), "<html>app</html>").unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        let state = AppState {
            db: Arc::new(Mutex::new(conn)),
            auth: Arc::new(AuthConfig {
                users: vec![User { name: "Ola".into(), password_hash: hash_password("haslo1234").unwrap() }],
                session_days: 30,
                cookie_secure: false,
            }),
            throttle: Arc::new(Throttle::default()),
            data_dir: dir.clone(),
        };
        router(state, dir)
    }

    async fn call(app: &Router, method: &str, uri: &str, body: Value, cookie: Option<&str>) -> (StatusCode, HeaderMap, Value) {
        let mut req = HttpRequest::builder().method(method).uri(uri).header("content-type", "application/json");
        if let Some(c) = cookie {
            req = req.header("cookie", c);
        }
        let res = app.clone().oneshot(req.body(Body::from(body.to_string())).unwrap()).await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, headers, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    #[tokio::test]
    async fn login_flow_and_protection() {
        let app = app();

        // bez logowania – brak dostępu do danych
        let (s, _, _) = call(&app, "POST", "/api/list_categories", json!({}), None).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
        let (s, _, _) = call(&app, "GET", "/api/backup.db", Value::Null, None).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);

        // złe hasło
        let (s, _, _) = call(&app, "POST", "/api/auth/login", json!({"username": "ola", "password": "zle"}), None).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);

        // dobre hasło → ciasteczko HttpOnly + SameSite=Strict
        let (s, h, body) = call(&app, "POST", "/api/auth/login", json!({"username": "ola", "password": "haslo1234"}), None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body["username"], "Ola");
        let set_cookie = h.get("set-cookie").unwrap().to_str().unwrap();
        assert!(set_cookie.contains("HttpOnly") && set_cookie.contains("SameSite=Strict"));
        let cookie = set_cookie.split(';').next().unwrap().to_string();

        // z sesją – komendy działają tak jak w Tauri
        let (s, _, cats) = call(&app, "POST", "/api/list_categories", json!({}), Some(&cookie)).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(cats.as_array().unwrap().len(), 7);
        let item = json!({"item": {"id": null, "name": "Chleb", "amount": 650, "date": "2026-09-30", "categoryId": 2, "note": ""}});
        let (s, _, _) = call(&app, "POST", "/api/save_expense", item, Some(&cookie)).await;
        assert_eq!(s, StatusCode::OK);
        let (_, _, sum) = call(&app, "POST", "/api/month_summary", json!({"month": "2026-09"}), Some(&cookie)).await;
        assert_eq!(sum["expenseTotal"], 650);

        // błąd walidacji → 400 z komunikatem po polsku
        let (s, _, e) = call(&app, "POST", "/api/list_expenses", json!({"month": "zly"}), Some(&cookie)).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        assert!(e["error"].as_str().unwrap().contains("miesiąc"));

        // wylogowanie unieważnia sesję
        let (s, _, _) = call(&app, "POST", "/api/auth/logout", json!({}), Some(&cookie)).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _, _) = call(&app, "POST", "/api/list_categories", json!({}), Some(&cookie)).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn serves_frontend_with_security_headers() {
        let app = app();
        let res = app
            .oneshot(HttpRequest::builder().uri("/wydatki").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK); // SPA: nieznana ścieżka → index.html
        assert_eq!(res.headers()["x-frame-options"], "DENY");
        assert_eq!(res.headers()["cache-control"], "no-cache");
    }
}
