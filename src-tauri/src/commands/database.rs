use std::path::PathBuf;
use std::sync::Mutex;
use tauri::command;

use crate::db::Database;

static DB: Mutex<Option<Database>> = Mutex::new(None);

fn with_db<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Database) -> Result<R, String>,
{
    let guard = DB.lock().map_err(|e| format!("DB lock: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    f(db)
}

/// Internal init (called from setup, not from frontend)
pub fn init_database_internal(path: &std::path::Path) -> Result<(u64, u64, u64), String> {
    crate::obd::dev_log::log_info("db", &format!("Initializing database from path: {:?}", path));
    let db = Database::open(path)?;
    let stats = db.get_stats();
    crate::obd::dev_log::log_info("db", &format!("Database stats — Operations: {}, Profiles: {}, ECUs: {}", stats.0, stats.1, stats.2));
    let mut guard = DB.lock().map_err(|e| format!("Lock: {}", e))?;
    *guard = Some(db);
    Ok(stats)
}

/// Initialize database (can also be called from frontend)
#[command]
pub fn init_database(db_path: String) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&db_path);
    let db = Database::open(&path)?;
    let (ops, profiles, ecus) = db.get_stats();

    let mut guard = DB.lock().map_err(|e| format!("Lock: {}", e))?;
    *guard = Some(db);

    Ok(serde_json::json!({
        "operations": ops,
        "profiles": profiles,
        "ecus": ecus,
    }))
}

/// Get database stats
#[command]
pub fn get_database_stats() -> Result<serde_json::Value, String> {
    with_db(|db| {
        let (ops, profiles, ecus) = db.get_stats();
        crate::obd::dev_log::log_debug("db", &format!("Stats requested — Operations: {}, Profiles: {}, ECUs: {}", ops, profiles, ecus));
        Ok(serde_json::json!({ "operations": ops, "profiles": profiles, "ecus": ecus }))
    })
}

/// Search operations by keyword
#[command]
pub fn search_operations(query: String, limit: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
    let limit_val = limit.unwrap_or(100);
    crate::obd::dev_log::log_debug("db", &format!("Searching operations: query='{}', limit={}", query, limit_val));
    with_db(|db| {
        let results = db.search_operations(&query, limit_val)?;
        crate::obd::dev_log::log_info("db", &format!("Search returned {} results", results.len()));
        Ok(results)
    })
}

/// Get operations for a specific vehicle make
#[command]
pub fn get_vehicle_operations(vehicle: String, limit: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
    with_db(|db| db.get_operations_for_vehicle(&vehicle, limit.unwrap_or(500)))
}

/// Get read operations for ECU + vehicle (Live Data / Dashboard)
#[command]
pub fn get_read_operations(ecu_name: String, vehicle: String) -> Result<Vec<serde_json::Value>, String> {
    with_db(|db| db.get_read_operations(&ecu_name, &vehicle))
}

/// Get write operations for ECU + vehicle (Advanced page)
#[command]
pub fn get_write_operations(ecu_name: String, vehicle: String) -> Result<Vec<serde_json::Value>, String> {
    with_db(|db| db.get_write_operations(&ecu_name, &vehicle))
}

/// Get all vehicle profiles
#[command]
pub fn get_vehicle_profiles() -> Result<Vec<String>, String> {
    with_db(|db| db.get_vehicle_profiles())
}

/// Get ECUs for a vehicle profile
#[command]
pub fn get_profile_ecus(profile_name: String) -> Result<Vec<serde_json::Value>, String> {
    with_db(|db| db.get_profile_ecus(&profile_name))
}

/// Search ECU catalog
#[command]
pub fn search_ecu_catalog(query: String, limit: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
    with_db(|db| db.search_ecu_catalog(&query, limit.unwrap_or(50)))
}
