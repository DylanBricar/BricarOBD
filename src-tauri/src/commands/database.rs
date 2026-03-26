use std::sync::Mutex;
use tauri::command;

use crate::db::Database;

static DB: Mutex<Option<Database>> = Mutex::new(None);

pub fn with_db<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Database) -> Result<R, String>,
{
    let guard = DB.lock().unwrap_or_else(|e| e.into_inner());
    let db = guard.as_ref().ok_or("Database not initialized")?;
    f(db)
}

/// Internal init (called from setup, not from frontend)
pub fn init_database_internal(path: &std::path::Path) -> Result<(u64, u64, u64), String> {
    crate::obd::dev_log::log_info("db", &format!("Initializing database from path: {:?}", path));
    let db = Database::open(path)?;
    let stats = db.get_stats();
    crate::obd::dev_log::log_info("db", &format!("Database stats — Operations: {}, Profiles: {}, ECUs: {}", stats.0, stats.1, stats.2));
    let mut guard = DB.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(db);
    Ok(stats)
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

/// Get all sessions
#[command]
pub fn get_sessions_cmd() -> Result<Vec<serde_json::Value>, String> {
    with_db(|db| db.get_sessions())
}

/// Delete a session
#[command]
pub fn delete_session_cmd(id: i64) -> Result<(), String> {
    with_db(|db| db.delete_session(id))
}

/// Save a session
#[command]
pub fn save_session_cmd(
    vin: String,
    make: String,
    model: String,
    dtc_count: i32,
    notes: String,
) -> Result<i64, String> {
    with_db(|db| db.save_session(&vin, &make, &model, dtc_count, &notes, ""))
}

/// Try to find a vehicle model name from the database
#[command]
pub fn find_vehicle_model(make: String) -> Option<String> {
    with_db(|db| Ok(db.find_vehicle_model(&make))).ok().flatten()
}

/// Sync helper for finding vehicle model (used from connection.rs inside spawn_blocking)
pub fn find_vehicle_model_sync(make: &str) -> Option<String> {
    with_db(|db| Ok(db.find_vehicle_model(make))).ok().flatten()
}

/// Sync helper for DID info lookup (used from dashboard.rs)
pub fn get_did_info_sync(did: &str, vehicle: &str) -> Option<(String, String, String)> {
    with_db(|db| Ok(db.get_did_info(did, vehicle))).ok().flatten()
}

/// Get distinct ECU names that have DTC operations for a vehicle
pub fn get_dtc_ecu_names_sync(vehicle: &str) -> Vec<String> {
    with_db(|db| {
        let results = db.search_dtc_context(vehicle);
        Ok(results.into_iter().map(|(_, _, ecu)| ecu).filter(|e| !e.is_empty()).collect::<Vec<_>>())
    }).unwrap_or_default()
}
