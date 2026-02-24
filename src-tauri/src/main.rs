#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;

use database::{Database, NewTransaction, Transaction, Container};
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
fn add_transaction(
    amount: i64,
    description: Option<String>,
    category: Option<String>,
    container_id: i64,
    db: tauri::State<Arc<Database>>,
) -> Result<Transaction, String> {
    let new_transaction = NewTransaction {
        amount,
        description,
        category,
        container_id,
    };
    
    db.add_transaction(new_transaction)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transactions(
    container_id: i64,
    limit: Option<i64>,
    db: tauri::State<Arc<Database>>,
) -> Result<Vec<Transaction>, String> {
    db.get_transactions(container_id, limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_monthly_balance(container_id: i64, db: tauri::State<Arc<Database>>) -> Result<i64, String> {
    db.get_monthly_balance(container_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_time_balance(container_id: i64, db: tauri::State<Arc<Database>>) -> Result<i64, String> {
    db.get_all_time_balance(container_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_csv(container_id: i64, db: tauri::State<Arc<Database>>) -> Result<String, String> {
    db.export_transactions_csv(container_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_transaction(id: i64, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.delete_transaction(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_category_totals(container_id: i64, db: tauri::State<Arc<Database>>) -> Result<Vec<(String, i64)>, String> {
    db.get_category_totals(container_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_categories(db: tauri::State<Arc<Database>>) -> Result<Vec<String>, String> {
    db.get_categories().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_category(name: String, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.add_category(name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_category(name: String, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.delete_category(name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_available_months(container_id: i64, db: tauri::State<Arc<Database>>) -> Result<Vec<String>, String> {
    db.get_available_months(container_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_balance_for_month(container_id: i64, month: String, db: tauri::State<Arc<Database>>) -> Result<i64, String> {
    db.get_balance_for_month(container_id, month).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transactions_for_month(
    container_id: i64,
    month: String,
    limit: Option<i64>,
    db: tauri::State<Arc<Database>>,
) -> Result<Vec<Transaction>, String> {
    db.get_transactions_for_month(container_id, month, limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_category_totals_for_month(container_id: i64, month: String, db: tauri::State<Arc<Database>>) -> Result<Vec<(String, i64)>, String> {
    db.get_category_totals_for_month(container_id, month).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_transaction(
    id: i64,
    amount: i64,
    description: String,
    category: String,
    db: tauri::State<Arc<Database>>,
) -> Result<Transaction, String> {
    db.update_transaction(id, amount, description, category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_containers(db: tauri::State<Arc<Database>>) -> Result<Vec<Container>, String> {
    db.get_containers().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_container(name: String, db: tauri::State<Arc<Database>>) -> Result<Container, String> {
    db.add_container(name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_container(id: i64, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.delete_container(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_container(id: i64, name: String, db: tauri::State<Arc<Database>>) -> Result<Container, String> {
    db.update_container(id, name).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn import_csv(
    csv_content: String,
    container_id: i64,
    amount_column: usize,
    description_column: usize,
    category_column: usize,
    date_column: usize,
    skip_header: bool,
    db: tauri::State<Arc<Database>>,
) -> Result<serde_json::Value, String> {
    let result = db.import_transactions_from_csv(
        csv_content,
        container_id,
        amount_column,
        description_column,
        category_column,
        date_column,
        skip_header,
    ).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success_count": result.success_count,
        "error_count": result.error_count,
        "errors": result.errors,
    }))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
            
            let db_path = app_dir.join("spent.db");
            let database = Arc::new(Database::new(db_path).expect("Failed to initialize database"));
            
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_transaction,
            get_transactions,
            get_monthly_balance,
            get_all_time_balance,
            delete_transaction,
            get_category_totals,
            get_categories,
            add_category,
            delete_category,
            export_csv,
            get_available_months,
            get_balance_for_month,
            get_transactions_for_month,
            get_category_totals_for_month,
            update_transaction,
            get_containers,
            add_container,
            delete_container,
            update_container,
            import_csv
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
