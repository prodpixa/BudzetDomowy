//! Logika aplikacji: każda funkcja dostaje połączenie z bazą i zwraca dane gotowe do JSON.
//! Warstwa HTTP (`api.rs`) tylko wywołuje te funkcje po nazwie.

use std::fmt::Display;
use std::io::Write;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db;

pub type Res<T> = Result<T, String>;

pub fn err<E: Display>(e: E) -> String {
    e.to_string()
}

// ---------- daty ----------

fn parse_month(month: &str) -> Res<(i32, u32)> {
    let bad = || format!("Nieprawidłowy miesiąc: {month}");
    let (y, m) = month.split_once('-').ok_or_else(bad)?;
    let y: i32 = y.parse().map_err(|_| bad())?;
    let m: u32 = m.parse().map_err(|_| bad())?;
    if !(1..=12).contains(&m) || y < 1900 {
        return Err(bad());
    }
    Ok((y, m))
}

fn fmt_month(y: i32, m: u32) -> String {
    format!("{y:04}-{m:02}")
}

fn shift_month(month: &str, delta: i32) -> Res<String> {
    let (y, m) = parse_month(month)?;
    let idx = y * 12 + (m as i32 - 1) + delta;
    Ok(fmt_month(idx.div_euclid(12), (idx.rem_euclid(12) + 1) as u32))
}

/// Zwraca (pierwszy dzień miesiąca, pierwszy dzień następnego) jako YYYY-MM-DD.
fn month_bounds(month: &str) -> Res<(String, String)> {
    parse_month(month)?;
    Ok((format!("{month}-01"), format!("{}-01", shift_month(month, 1)?)))
}

fn validate_date(date: &str) -> Res<()> {
    let ok = date.len() == 10
        && date.as_bytes()[4] == b'-'
        && date.as_bytes()[7] == b'-'
        && parse_month(&date[..7]).is_ok()
        && date[8..].parse::<u32>().map(|d| (1..=31).contains(&d)).unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(format!("Nieprawidłowa data: {date}"))
    }
}

fn validate_entry(name: &str, amount: i64) -> Res<()> {
    if name.trim().is_empty() {
        return Err("Nazwa nie może być pusta".into());
    }
    if amount <= 0 {
        return Err("Kwota musi być większa od zera".into());
    }
    Ok(())
}

// ---------- kategorie ----------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
    pub color: i64,
    pub icon: String,
}

pub fn list_categories(conn: &Connection) -> Res<Vec<Category>> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, icon FROM categories ORDER BY sort_order, id")
        .map_err(err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category { id: r.get(0)?, name: r.get(1)?, color: r.get(2)?, icon: r.get(3)? })
        })
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

pub fn save_category(conn: &Connection, item: Category) -> Res<i64> {
    if item.name.trim().is_empty() {
        return Err("Nazwa kategorii nie może być pusta".into());
    }
    match item.id {
        Some(id) => {
            conn.execute(
                "UPDATE categories SET name = ?1, color = ?2, icon = ?3 WHERE id = ?4",
                params![item.name.trim(), item.color, item.icon, id],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO categories (name, color, icon, sort_order)
                 VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM categories))",
                params![item.name.trim(), item.color, item.icon],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_category(conn: &Connection, id: i64) -> Res<()> {
    conn.execute("DELETE FROM categories WHERE id = ?1", [id]).map_err(err)?;
    Ok(())
}

// ---------- źródła przychodów ----------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: Option<i64>,
    pub name: String,
}

pub fn list_sources(conn: &Connection) -> Res<Vec<Source>> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM income_sources ORDER BY sort_order, id")
        .map_err(err)?;
    let rows = stmt
        .query_map([], |r| Ok(Source { id: r.get(0)?, name: r.get(1)? }))
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

pub fn save_source(conn: &Connection, item: Source) -> Res<i64> {
    if item.name.trim().is_empty() {
        return Err("Nazwa źródła nie może być pusta".into());
    }
    match item.id {
        Some(id) => {
            conn.execute(
                "UPDATE income_sources SET name = ?1 WHERE id = ?2",
                params![item.name.trim(), id],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO income_sources (name, sort_order)
                 VALUES (?1, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM income_sources))",
                params![item.name.trim()],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_source(conn: &Connection, id: i64) -> Res<()> {
    conn.execute("DELETE FROM income_sources WHERE id = ?1", [id]).map_err(err)?;
    Ok(())
}

// ---------- wydatki ----------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub id: Option<i64>,
    pub name: String,
    pub amount: i64,
    pub date: String,
    pub category_id: Option<i64>,
    pub note: String,
}

pub fn list_expenses(conn: &Connection, month: String) -> Res<Vec<Expense>> {
    let (start, end) = month_bounds(&month)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, amount, date, category_id, note FROM expenses
             WHERE date >= ?1 AND date < ?2 ORDER BY date DESC, id DESC",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map([start, end], |r| {
            Ok(Expense {
                id: r.get(0)?,
                name: r.get(1)?,
                amount: r.get(2)?,
                date: r.get(3)?,
                category_id: r.get(4)?,
                note: r.get(5)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

pub fn save_expense(conn: &Connection, item: Expense) -> Res<i64> {
    validate_entry(&item.name, item.amount)?;
    validate_date(&item.date)?;
    match item.id {
        Some(id) => {
            conn.execute(
                "UPDATE expenses SET name = ?1, amount = ?2, date = ?3, category_id = ?4, note = ?5
                 WHERE id = ?6",
                params![item.name.trim(), item.amount, item.date, item.category_id, item.note, id],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO expenses (name, amount, date, category_id, note)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![item.name.trim(), item.amount, item.date, item.category_id, item.note],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_expense(conn: &Connection, id: i64) -> Res<()> {
    conn.execute("DELETE FROM expenses WHERE id = ?1", [id]).map_err(err)?;
    Ok(())
}

// ---------- przychody ----------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Income {
    pub id: Option<i64>,
    pub name: String,
    pub amount: i64,
    pub date: String,
    pub source_id: Option<i64>,
    pub note: String,
}

pub fn list_incomes(conn: &Connection, month: String) -> Res<Vec<Income>> {
    let (start, end) = month_bounds(&month)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, amount, date, source_id, note FROM incomes
             WHERE date >= ?1 AND date < ?2 ORDER BY date DESC, id DESC",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map([start, end], |r| {
            Ok(Income {
                id: r.get(0)?,
                name: r.get(1)?,
                amount: r.get(2)?,
                date: r.get(3)?,
                source_id: r.get(4)?,
                note: r.get(5)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

pub fn save_income(conn: &Connection, item: Income) -> Res<i64> {
    validate_entry(&item.name, item.amount)?;
    validate_date(&item.date)?;
    match item.id {
        Some(id) => {
            conn.execute(
                "UPDATE incomes SET name = ?1, amount = ?2, date = ?3, source_id = ?4, note = ?5
                 WHERE id = ?6",
                params![item.name.trim(), item.amount, item.date, item.source_id, item.note, id],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO incomes (name, amount, date, source_id, note)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![item.name.trim(), item.amount, item.date, item.source_id, item.note],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_income(conn: &Connection, id: i64) -> Res<()> {
    conn.execute("DELETE FROM incomes WHERE id = ?1", [id]).map_err(err)?;
    Ok(())
}

// ---------- planowane (ToDo) ----------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Planned {
    pub id: Option<i64>,
    pub kind: String,
    pub month: String,
    pub name: String,
    pub amount: i64,
    pub note: String,
    pub done: bool,
}

fn validate_kind(kind: &str) -> Res<()> {
    match kind {
        "expense" | "income" => Ok(()),
        _ => Err(format!("Nieznany rodzaj pozycji: {kind}")),
    }
}

pub fn list_planned(conn: &Connection, month: String, kind: String) -> Res<Vec<Planned>> {
    parse_month(&month)?;
    validate_kind(&kind)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, month, name, amount, note, done FROM planned_items
             WHERE month = ?1 AND kind = ?2 ORDER BY done, sort_order, id",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map([month, kind], |r| {
            Ok(Planned {
                id: r.get(0)?,
                kind: r.get(1)?,
                month: r.get(2)?,
                name: r.get(3)?,
                amount: r.get(4)?,
                note: r.get(5)?,
                done: r.get(6)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

pub fn save_planned(conn: &Connection, item: Planned) -> Res<i64> {
    validate_entry(&item.name, item.amount)?;
    validate_kind(&item.kind)?;
    parse_month(&item.month)?;
    match item.id {
        Some(id) => {
            conn.execute(
                "UPDATE planned_items SET name = ?1, amount = ?2, note = ?3, done = ?4 WHERE id = ?5",
                params![item.name.trim(), item.amount, item.note, item.done, id],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO planned_items (kind, month, name, amount, note, done, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                   (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM planned_items
                    WHERE month = ?2 AND kind = ?1))",
                params![item.kind, item.month, item.name.trim(), item.amount, item.note, item.done],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn set_planned_done(conn: &Connection, id: i64, done: bool) -> Res<()> {
    conn.execute("UPDATE planned_items SET done = ?1 WHERE id = ?2", params![done, id])
        .map_err(err)?;
    Ok(())
}

pub fn delete_planned(conn: &Connection, id: i64) -> Res<()> {
    conn.execute("DELETE FROM planned_items WHERE id = ?1", [id]).map_err(err)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    /// Miesiąc, z którego skopiowano (najbliższy wcześniejszy z pozycjami).
    pub from_month: Option<String>,
    pub copied: usize,
    pub skipped: usize,
}

/// Kopiuje pozycje z najbliższego wcześniejszego miesiąca, który ma jakieś pozycje.
/// Pozycje o nazwie już obecnej w bieżącym miesiącu są pomijane.
pub fn copy_planned_from_previous(
    conn: &Connection,
    month: String,
    kind: String,
) -> Res<CopyResult> {
    parse_month(&month)?;
    validate_kind(&kind)?;
    let tx = conn.unchecked_transaction().map_err(err)?;

    let from_month: Option<String> = tx
        .query_row(
            "SELECT MAX(month) FROM planned_items WHERE month < ?1 AND kind = ?2",
            [&month, &kind],
            |r| r.get(0),
        )
        .map_err(err)?;
    let Some(from) = from_month else {
        return Ok(CopyResult { from_month: None, copied: 0, skipped: 0 });
    };

    let copied = tx
        .execute(
            "INSERT INTO planned_items (kind, month, name, amount, note, done, sort_order)
             SELECT kind, ?1, name, amount, note, 0, sort_order FROM planned_items p
             WHERE p.month = ?2 AND p.kind = ?3
               AND NOT EXISTS (SELECT 1 FROM planned_items q
                               WHERE q.month = ?1 AND q.kind = ?3
                                 AND lower(q.name) = lower(p.name))",
            params![month, from, kind],
        )
        .map_err(err)?;
    let total: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM planned_items WHERE month = ?1 AND kind = ?2",
            [&from, &kind],
            |r| r.get(0),
        )
        .map_err(err)?;
    tx.commit().map_err(err)?;
    Ok(CopyResult { from_month: Some(from), copied, skipped: (total as usize).saturating_sub(copied) })
}

// ---------- bony żywnościowe ----------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Voucher {
    pub id: Option<i64>,
    pub kind: String,
    pub place: String,
    pub amount: i64,
    pub date: String,
    pub note: String,
}

pub fn list_vouchers(conn: &Connection, month: String) -> Res<Vec<Voucher>> {
    let (start, end) = month_bounds(&month)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, place, amount, date, note FROM voucher_entries
             WHERE date >= ?1 AND date < ?2 ORDER BY date DESC, id DESC",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map([start, end], |r| {
            Ok(Voucher {
                id: r.get(0)?,
                kind: r.get(1)?,
                place: r.get(2)?,
                amount: r.get(3)?,
                date: r.get(4)?,
                note: r.get(5)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

pub fn save_voucher(conn: &Connection, item: Voucher) -> Res<i64> {
    validate_entry(&item.place, item.amount)?;
    validate_date(&item.date)?;
    if item.kind != "topup" && item.kind != "spend" {
        return Err(format!("Nieznany rodzaj wpisu: {}", item.kind));
    }
    match item.id {
        Some(id) => {
            conn.execute(
                "UPDATE voucher_entries SET kind = ?1, place = ?2, amount = ?3, date = ?4, note = ?5
                 WHERE id = ?6",
                params![item.kind, item.place.trim(), item.amount, item.date, item.note, id],
            )
            .map_err(err)?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO voucher_entries (kind, place, amount, date, note)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![item.kind, item.place.trim(), item.amount, item.date, item.note],
            )
            .map_err(err)?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_voucher(conn: &Connection, id: i64) -> Res<()> {
    conn.execute("DELETE FROM voucher_entries WHERE id = ?1", [id]).map_err(err)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoucherSummary {
    /// Saldo na karcie ze wszystkich wpisów do końca wybranego miesiąca.
    pub balance: i64,
    /// Saldo na początek miesiąca.
    pub carried_over: i64,
    pub month_topups: i64,
    pub month_spends: i64,
}

pub fn voucher_summary(conn: &Connection, month: String) -> Res<VoucherSummary> {
    let (start, end) = month_bounds(&month)?;
    let sum = |kind: &str, from: &str, to: &str| -> Res<i64> {
        conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM voucher_entries
             WHERE kind = ?1 AND date >= ?2 AND date < ?3",
            [kind, from, to],
            |r| r.get(0),
        )
        .map_err(err)
    };
    let carried_over = sum("topup", "", &start)? - sum("spend", "", &start)?;
    let month_topups = sum("topup", &start, &end)?;
    let month_spends = sum("spend", &start, &end)?;
    Ok(VoucherSummary {
        balance: carried_over + month_topups - month_spends,
        carried_over,
        month_topups,
        month_spends,
    })
}

// ---------- podsumowanie miesiąca ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupTotal {
    pub id: Option<i64>,
    pub name: String,
    pub color: i64,
    pub icon: String,
    pub total: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedTotals {
    pub total: i64,
    pub remaining: i64,
    pub count: i64,
    pub done_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthSummary {
    pub carried_over: i64,
    pub income_total: i64,
    pub expense_total: i64,
    /// Przychody - wydatki w tym miesiącu.
    pub balance: i64,
    /// Saldo przeniesione + bilans miesiąca.
    pub end_balance: i64,
    pub by_category: Vec<GroupTotal>,
    pub by_source: Vec<GroupTotal>,
    pub planned_expenses: PlannedTotals,
    pub planned_incomes: PlannedTotals,
}

fn opening_balance(conn: &Connection) -> Res<i64> {
    let v: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = 'opening_balance'", [], |r| r.get(0))
        .optional()
        .map_err(err)?;
    Ok(v.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn sum_between(conn: &Connection, table: &str, from: &str, to: &str) -> Res<i64> {
    conn.query_row(
        &format!("SELECT COALESCE(SUM(amount), 0) FROM {table} WHERE date >= ?1 AND date < ?2"),
        [from, to],
        |r| r.get(0),
    )
    .map_err(err)
}

fn planned_totals(conn: &Connection, month: &str, kind: &str) -> Res<PlannedTotals> {
    conn.query_row(
        "SELECT COALESCE(SUM(amount), 0),
                COALESCE(SUM(CASE WHEN done = 0 THEN amount END), 0),
                COUNT(*),
                COALESCE(SUM(done), 0)
         FROM planned_items WHERE month = ?1 AND kind = ?2",
        [month, kind],
        |r| {
            Ok(PlannedTotals {
                total: r.get(0)?,
                remaining: r.get(1)?,
                count: r.get(2)?,
                done_count: r.get(3)?,
            })
        },
    )
    .map_err(err)
}

pub fn month_summary(conn: &Connection, month: String) -> Res<MonthSummary> {
    let (start, end) = month_bounds(&month)?;

    let carried_over = opening_balance(&conn)? + sum_between(&conn, "incomes", "", &start)?
        - sum_between(&conn, "expenses", "", &start)?;
    let income_total = sum_between(&conn, "incomes", &start, &end)?;
    let expense_total = sum_between(&conn, "expenses", &start, &end)?;

    let mut stmt = conn
        .prepare(
            "SELECT c.id, COALESCE(c.name, 'Bez kategorii'), COALESCE(c.color, 0),
                    COALESCE(c.icon, 'circle-dashed'), SUM(e.amount) AS total
             FROM expenses e LEFT JOIN categories c ON c.id = e.category_id
             WHERE e.date >= ?1 AND e.date < ?2
             GROUP BY c.id ORDER BY total DESC",
        )
        .map_err(err)?;
    let by_category = stmt
        .query_map([&start, &end], |r| {
            Ok(GroupTotal {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                icon: r.get(3)?,
                total: r.get(4)?,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;

    let mut stmt = conn
        .prepare(
            "SELECT s.id, COALESCE(s.name, 'Bez źródła'), SUM(i.amount) AS total
             FROM incomes i LEFT JOIN income_sources s ON s.id = i.source_id
             WHERE i.date >= ?1 AND i.date < ?2
             GROUP BY s.id ORDER BY total DESC",
        )
        .map_err(err)?;
    let by_source = stmt
        .query_map([&start, &end], |r| {
            Ok(GroupTotal {
                id: r.get(0)?,
                name: r.get(1)?,
                color: 0,
                icon: String::new(),
                total: r.get(2)?,
            })
        })
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)?;

    let balance = income_total - expense_total;
    Ok(MonthSummary {
        carried_over,
        income_total,
        expense_total,
        balance,
        end_balance: carried_over + balance,
        by_category,
        by_source,
        planned_expenses: planned_totals(&conn, &month, "expense")?,
        planned_incomes: planned_totals(&conn, &month, "income")?,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub month: String,
    pub income: i64,
    pub expense: i64,
}

/// Przychody i wydatki z `count` miesięcy kończących się na `month`.
pub fn month_trend(conn: &Connection, month: String, count: u32) -> Res<Vec<TrendPoint>> {
    let count = count.clamp(1, 24) as i32;
    let mut out = Vec::with_capacity(count as usize);
    for i in (0..count).rev() {
        let m = shift_month(&month, -i)?;
        let (start, end) = month_bounds(&m)?;
        out.push(TrendPoint {
            income: sum_between(&conn, "incomes", &start, &end)?,
            expense: sum_between(&conn, "expenses", &start, &end)?,
            month: m,
        });
    }
    Ok(out)
}

// ---------- ustawienia ----------

pub fn get_opening_balance(conn: &Connection) -> Res<i64> {
    opening_balance(&conn)
}

pub fn set_opening_balance(conn: &Connection, amount: i64) -> Res<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('opening_balance', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [amount.to_string()],
    )
    .map_err(err)?;
    Ok(())
}

// ---------- eksport i przywracanie ----------

fn csv_field(s: &str) -> String {
    if s.contains([';', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn money(grosze: i64) -> String {
    let sign = if grosze < 0 { "-" } else { "" };
    format!("{sign}{},{:02}", grosze.abs() / 100, grosze.abs() % 100)
}

/// CSV: średnik + przecinek dziesiętny + BOM – otwiera się poprawnie w polskim Excelu.
fn build_csv(conn: &Connection, header: &str, sql: &str) -> Res<String> {
    let mut out = String::from("\u{feff}");
    out.push_str(header);
    out.push('\n');
    let mut stmt = conn.prepare(sql).map_err(err)?;
    let cols = stmt.column_count();
    let mut rows = stmt.query([]).map_err(err)?;
    while let Some(row) = rows.next().map_err(err)? {
        let mut fields = Vec::with_capacity(cols);
        for i in 0..cols {
            let name = row.as_ref().column_name(i).map_err(err)?;
            let field = if name == "amount" {
                money(row.get::<_, i64>(i).map_err(err)?)
            } else {
                row.get::<_, Option<String>>(i).map_err(err)?.unwrap_or_default()
            };
            fields.push(csv_field(&field));
        }
        out.push_str(&fields.join(";"));
        out.push('\n');
    }
    Ok(out)
}

/// Wszystkie dane jako archiwum ZIP z czterema plikami CSV.
pub fn export_zip(conn: &Connection) -> Res<Vec<u8>> {
    let files = [
        (
            "wydatki.csv",
            "Data;Nazwa;Kategoria;Kwota;Notatka",
            "SELECT e.date, e.name, c.name, e.amount AS amount, e.note FROM expenses e
             LEFT JOIN categories c ON c.id = e.category_id ORDER BY e.date, e.id",
        ),
        (
            "przychody.csv",
            "Data;Nazwa;Źródło;Kwota;Notatka",
            "SELECT i.date, i.name, s.name, i.amount AS amount, i.note FROM incomes i
             LEFT JOIN income_sources s ON s.id = i.source_id ORDER BY i.date, i.id",
        ),
        (
            "planowane.csv",
            "Miesiąc;Rodzaj;Nazwa;Kwota;Zrobione;Notatka",
            "SELECT month, CASE kind WHEN 'income' THEN 'Przychód' ELSE 'Wydatek' END,
                    name, amount AS amount, CASE done WHEN 1 THEN 'tak' ELSE 'nie' END, note
             FROM planned_items ORDER BY month, kind, sort_order, id",
        ),
        (
            "bony.csv",
            "Data;Rodzaj;Gdzie;Kwota;Notatka",
            "SELECT date, CASE kind WHEN 'topup' THEN 'Wpływ' ELSE 'Wydatek' END,
                    place, amount AS amount, note
             FROM voucher_entries ORDER BY date, id",
        ),
    ];
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (file, header, sql) in files {
        let csv = build_csv(conn, header, sql)?;
        zip.start_file(file, opts).map_err(err)?;
        zip.write_all(csv.as_bytes()).map_err(err)?;
    }
    Ok(zip.finish().map_err(err)?.into_inner())
}

/// Sprawdza, czy plik jest bazą tej aplikacji (zanim cokolwiek nadpiszemy).
pub fn validate_backup_file(path: &Path) -> Res<()> {
    let bad = || "Wybrany plik nie jest kopią bazy budżetu".to_string();
    let src = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| bad())?;
    let tables: i64 = src
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'
             AND name IN ('expenses', 'incomes', 'planned_items', 'voucher_entries', 'categories')",
            [],
            |r| r.get(0),
        )
        .map_err(|_| bad())?;
    if tables != 5 {
        return Err(bad());
    }
    Ok(())
}

/// Zastępuje całą bazę zawartością kopii. Aktywne sesje logowania są zachowywane,
/// żeby przywrócenie starej kopii nikogo nie wylogowało.
pub fn restore_from(conn: &mut Connection, path: &Path) -> Res<()> {
    validate_backup_file(path)?;
    let sessions: Vec<(String, String, i64, i64)> = {
        let mut stmt = conn
            .prepare("SELECT token_hash, username, created_at, expires_at FROM sessions")
            .map_err(err)?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)?
    };

    conn.restore(rusqlite::MAIN_DB, path, None::<fn(rusqlite::backup::Progress)>)
        .map_err(err)?;
    db::configure(conn).map_err(err)?;
    db::migrate(conn).map_err(err)?;

    let tx = conn.transaction().map_err(err)?;
    tx.execute("DELETE FROM sessions", []).map_err(err)?;
    for (token_hash, username, created_at, expires_at) in sessions {
        tx.execute(
            "INSERT INTO sessions (token_hash, username, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
            params![token_hash, username, created_at, expires_at],
        )
        .map_err(err)?;
    }
    tx.commit().map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_math() {
        assert_eq!(shift_month("2026-01", -1).unwrap(), "2025-12");
        assert_eq!(shift_month("2026-12", 1).unwrap(), "2027-01");
        assert_eq!(shift_month("2026-09", -13).unwrap(), "2025-08");
        assert_eq!(
            month_bounds("2026-12").unwrap(),
            ("2026-12-01".to_string(), "2027-01-01".to_string())
        );
        assert!(parse_month("2026-13").is_err());
        assert!(validate_date("2026-09-29").is_ok());
        assert!(validate_date("2026-9-29").is_err());
    }

    #[test]
    fn money_format() {
        assert_eq!(money(123456), "1234,56");
        assert_eq!(money(-5), "-0,05");
        assert_eq!(csv_field("a;b"), "\"a;b\"");
    }
}
