mod commands;
mod db;

use std::sync::Mutex;

use tauri::Manager;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("budzet.db"))?;
            app.manage(AppState { conn: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_categories,
            commands::save_category,
            commands::delete_category,
            commands::list_sources,
            commands::save_source,
            commands::delete_source,
            commands::list_expenses,
            commands::save_expense,
            commands::delete_expense,
            commands::list_incomes,
            commands::save_income,
            commands::delete_income,
            commands::list_planned,
            commands::save_planned,
            commands::set_planned_done,
            commands::delete_planned,
            commands::copy_planned_from_previous,
            commands::list_vouchers,
            commands::save_voucher,
            commands::delete_voucher,
            commands::voucher_summary,
            commands::month_summary,
            commands::month_trend,
            commands::get_opening_balance,
            commands::set_opening_balance,
            commands::export_csv,
            commands::backup_db,
            commands::restore_db,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
