use std::path::Path;

use rusqlite::{params, Connection};

/// Kolejne migracje schematu. Indeks + 1 = numer wersji (PRAGMA user_version).
/// Nigdy nie edytuj istniejących wpisów – tylko dopisuj nowe na końcu.
const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE categories (
        id          INTEGER PRIMARY KEY,
        name        TEXT    NOT NULL,
        color       INTEGER NOT NULL,          -- slot palety 1..8
        icon        TEXT    NOT NULL,
        sort_order  INTEGER NOT NULL DEFAULT 0
    );

    CREATE TABLE income_sources (
        id          INTEGER PRIMARY KEY,
        name        TEXT    NOT NULL,
        sort_order  INTEGER NOT NULL DEFAULT 0
    );

    -- Kwoty przechowujemy w groszach (INTEGER), żeby uniknąć błędów zaokrągleń.
    CREATE TABLE expenses (
        id           INTEGER PRIMARY KEY,
        name         TEXT    NOT NULL,
        amount       INTEGER NOT NULL,
        date         TEXT    NOT NULL,         -- YYYY-MM-DD
        category_id  INTEGER REFERENCES categories(id) ON DELETE SET NULL,
        note         TEXT    NOT NULL DEFAULT ''
    );
    CREATE INDEX idx_expenses_date ON expenses(date);

    CREATE TABLE incomes (
        id          INTEGER PRIMARY KEY,
        name        TEXT    NOT NULL,
        amount      INTEGER NOT NULL,
        date        TEXT    NOT NULL,
        source_id   INTEGER REFERENCES income_sources(id) ON DELETE SET NULL,
        note        TEXT    NOT NULL DEFAULT ''
    );
    CREATE INDEX idx_incomes_date ON incomes(date);

    CREATE TABLE planned_items (
        id          INTEGER PRIMARY KEY,
        kind        TEXT    NOT NULL CHECK (kind IN ('expense', 'income')),
        month       TEXT    NOT NULL,          -- YYYY-MM
        name        TEXT    NOT NULL,
        amount      INTEGER NOT NULL,
        note        TEXT    NOT NULL DEFAULT '',
        done        INTEGER NOT NULL DEFAULT 0,
        sort_order  INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX idx_planned_month ON planned_items(month, kind);

    CREATE TABLE voucher_entries (
        id          INTEGER PRIMARY KEY,
        kind        TEXT    NOT NULL CHECK (kind IN ('topup', 'spend')),
        place       TEXT    NOT NULL,
        amount      INTEGER NOT NULL,
        date        TEXT    NOT NULL,
        note        TEXT    NOT NULL DEFAULT ''
    );
    CREATE INDEX idx_voucher_date ON voucher_entries(date);

    CREATE TABLE settings (
        key    TEXT PRIMARY KEY,
        value  TEXT NOT NULL
    );
"#,
    // 2: sesje logowania (wersja webowa). Trzymamy tylko skrót SHA-256 tokenu,
    // więc wyciek bazy lub kopii zapasowej nie pozwala przejąć sesji.
    r#"
    CREATE TABLE sessions (
        token_hash  TEXT    PRIMARY KEY,
        username    TEXT    NOT NULL,
        created_at  INTEGER NOT NULL,          -- sekundy od epoki (UTC)
        expires_at  INTEGER NOT NULL
    );
    CREATE INDEX idx_sessions_expires ON sessions(expires_at);
"#,
];

const DEFAULT_CATEGORIES: &[(&str, i64, &str)] = &[
    ("Dom", 1, "house"),
    ("Jedzenie", 2, "shopping-cart"),
    ("Gotowe jedzenie", 3, "utensils"),
    ("Transport", 4, "car"),
    ("Zdrowie", 5, "heart-pulse"),
    ("Subskrypcje", 6, "repeat"),
    ("Hobby", 7, "palette"),
];

const DEFAULT_SOURCES: &[&str] = &["Pensja", "Premia", "Zwroty", "Inne"];

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Ustawienia połączenia. WAL: czytanie nie blokuje zapisu (dwie osoby + backup w tle).
/// busy_timeout: przy chwilowej blokadzie czekaj zamiast od razu zwracać błąd.
pub fn configure(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;",
    )
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        if i == 0 {
            seed(&tx)?;
        }
        tx.pragma_update(None, "user_version", i as i64 + 1)?;
        tx.commit()?;
    }
    Ok(())
}

fn seed(conn: &Connection) -> rusqlite::Result<()> {
    for (i, (name, color, icon)) in DEFAULT_CATEGORIES.iter().enumerate() {
        conn.execute(
            "INSERT INTO categories (name, color, icon, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![name, color, icon, i as i64],
        )?;
    }
    for (i, name) in DEFAULT_SOURCES.iter().enumerate() {
        conn.execute(
            "INSERT INTO income_sources (name, sort_order) VALUES (?1, ?2)",
            params![name, i as i64],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_seeds_once() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // drugi raz nic nie robi
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, MIGRATIONS.len() as i64);
        let cats: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0)).unwrap();
        let srcs: i64 =
            conn.query_row("SELECT COUNT(*) FROM income_sources", [], |r| r.get(0)).unwrap();
        assert_eq!(cats, 7);
        assert_eq!(srcs, 4);
    }
}
