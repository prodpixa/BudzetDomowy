//! Kopie zapasowe bazy: codzienna automatyczna (trzymamy N ostatnich) i ręczna (np. przed wdrożeniem).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, Timelike};
use rusqlite::Connection;

use crate::commands::{err, Res};

const DAILY_PREFIX: &str = "budzet-";
const MANUAL_PREFIX: &str = "reczna-";
const MANUAL_KEEP: usize = 10;

/// Spójna kopia działającej bazy (API backupu SQLite, a nie `cp`, które mogłoby
/// skopiować plik w połowie zapisu). Najpierw plik tymczasowy, potem zmiana nazwy.
pub fn backup_to(conn: &Connection, dest: &Path) -> Res<()> {
    let tmp = dest.with_extension("db.tmp");
    let _ = fs::remove_file(&tmp);
    conn.backup(rusqlite::MAIN_DB, &tmp, None).map_err(err)?;
    fs::rename(&tmp, dest).map_err(err)
}

fn is_daily_name(name: &str) -> bool {
    // budzet-RRRR-MM-DD.db
    name.len() == DAILY_PREFIX.len() + 13
        && name.starts_with(DAILY_PREFIX)
        && name.ends_with(".db")
        && name[DAILY_PREFIX.len()..name.len() - 3]
            .bytes()
            .enumerate()
            .all(|(i, b)| if i == 4 || i == 7 { b == b'-' } else { b.is_ascii_digit() })
}

/// Usuwa najstarsze pliki pasujące do `matches`, zostawiając `keep` najnowszych.
/// Nazwy zawierają datę, więc sortowanie alfabetyczne = chronologiczne.
fn prune(dir: &Path, keep: usize, matches: impl Fn(&str) -> bool) -> Res<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(err)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(&matches))
        .collect();
    files.sort();
    let excess = files.len().saturating_sub(keep);
    let removed: Vec<PathBuf> = files.drain(..excess).collect();
    for f in &removed {
        fs::remove_file(f).map_err(err)?;
    }
    Ok(removed)
}

/// Robi dzisiejszą kopię, jeśli jej jeszcze nie ma i minęła godzina `hour`.
/// Wywoływane co kilka minut – dzięki temu kopia powstanie także wtedy,
/// gdy serwer był wyłączony o zaplanowanej godzinie.
pub fn daily_if_due(conn: &Connection, dir: &Path, keep: usize, hour: u32) -> Res<Option<PathBuf>> {
    let now = Local::now();
    if now.hour() < hour {
        return Ok(None);
    }
    fs::create_dir_all(dir).map_err(err)?;
    let path = dir.join(format!("{DAILY_PREFIX}{}.db", now.format("%Y-%m-%d")));
    if path.exists() {
        return Ok(None);
    }
    backup_to(conn, &path)?;
    prune(dir, keep, is_daily_name)?;
    Ok(Some(path))
}

/// Kopia na żądanie, np. `reczna-2026-09-30_142501-przed-wdrozeniem.db`.
pub fn manual(conn: &Connection, dir: &Path, tag: Option<&str>) -> Res<PathBuf> {
    fs::create_dir_all(dir).map_err(err)?;
    let tag: String = tag
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(40)
        .collect();
    let stamp = Local::now().format("%Y-%m-%d_%H%M%S");
    let name = if tag.is_empty() {
        format!("{MANUAL_PREFIX}{stamp}.db")
    } else {
        format!("{MANUAL_PREFIX}{stamp}-{tag}.db")
    };
    let path = dir.join(name);
    backup_to(conn, &path)?;
    prune(dir, MANUAL_KEEP, |n| n.starts_with(MANUAL_PREFIX) && n.ends_with(".db"))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("budzet-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn daily_names() {
        assert!(is_daily_name("budzet-2026-09-30.db"));
        assert!(!is_daily_name("budzet-2026-09-30.db.tmp"));
        assert!(!is_daily_name("budzet.db"));
        assert!(!is_daily_name("reczna-2026-09-30_120000.db"));
    }

    #[test]
    fn keeps_only_newest() {
        let dir = temp_dir("prune");
        for d in 1..=20 {
            fs::write(dir.join(format!("budzet-2026-09-{d:02}.db")), b"x").unwrap();
        }
        fs::write(dir.join("budzet.db"), b"glowna baza").unwrap();
        let removed = prune(&dir, 14, is_daily_name).unwrap();
        assert_eq!(removed.len(), 6);
        assert!(!dir.join("budzet-2026-09-06.db").exists());
        assert!(dir.join("budzet-2026-09-07.db").exists());
        assert!(dir.join("budzet.db").exists(), "główna baza nie może zostać usunięta");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn backup_is_readable_copy() {
        let dir = temp_dir("copy");
        let conn = db::open(&dir.join("budzet.db")).unwrap();
        conn.execute(
            "INSERT INTO expenses (name, amount, date) VALUES ('Test', 1234, '2026-09-30')",
            [],
        )
        .unwrap();
        let path = manual(&conn, &dir.join("backups"), Some("test")).unwrap();
        let copy = Connection::open(&path).unwrap();
        let n: i64 = copy.query_row("SELECT amount FROM expenses", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1234);
        fs::remove_dir_all(dir).unwrap();
    }
}
