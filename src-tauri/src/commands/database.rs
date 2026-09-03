use std::sync::Mutex;
use tauri::command;

use crate::db::Database;

static DB: Mutex<Option<Database>> = Mutex::new(None);

pub fn is_database_initialized() -> bool {
    DB.lock()
        .unwrap_or_else(|error| error.into_inner())
        .is_some()
}

pub fn database_stats() -> Option<(u64, u64, u64)> {
    DB.lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .map(Database::get_stats)
}

pub fn with_db<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Database) -> Result<R, String>,
{
    let guard = DB.lock().unwrap_or_else(|e| e.into_inner());
    let db = guard.as_ref().ok_or("Database not initialized")?;
    f(db)
}

/// Internal init (called from setup, not from frontend)
pub fn init_database_internal(
    catalog_path: &std::path::Path,
    user_path: &std::path::Path,
) -> Result<(u64, u64, u64), String> {
    crate::obd::dev_log::log_info(
        "db",
        &format!(
            "Initializing catalog database from path: {:?}",
            catalog_path
        ),
    );
    let db = Database::open(catalog_path, user_path)?;
    let stats = db.get_stats();
    if stats.0 < 3_000_000 || stats.1 < 80 || stats.2 < 4_000 {
        return Err(format!(
            "Operations database is incomplete: {} operations, {} profiles, {} ECUs",
            stats.0, stats.1, stats.2
        ));
    }
    crate::obd::dev_log::log_info(
        "db",
        &format!(
            "Database stats — Operations: {}, Profiles: {}, ECUs: {}",
            stats.0, stats.1, stats.2
        ),
    );
    let mut guard = DB.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(db);
    Ok(stats)
}

/// Get database stats
#[command]
pub fn get_database_stats() -> Result<serde_json::Value, String> {
    with_db(|db| {
        let (ops, profiles, ecus) = db.get_stats();
        crate::obd::dev_log::log_debug(
            "db",
            &format!(
                "Stats requested — Operations: {}, Profiles: {}, ECUs: {}",
                ops, profiles, ecus
            ),
        );
        Ok(serde_json::json!({ "operations": ops, "profiles": profiles, "ecus": ecus }))
    })
}

/// Search operations by keyword
#[command]
pub fn search_operations(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    let limit_val = limit.unwrap_or(100).clamp(1, 500);
    crate::obd::dev_log::log_debug(
        "db",
        &format!(
            "Searching operations: query='{}', limit={}",
            query, limit_val
        ),
    );
    with_db(|db| {
        let results = db.search_operations(&query, limit_val)?;
        crate::obd::dev_log::log_info("db", &format!("Search returned {} results", results.len()));
        Ok(results)
    })
}

/// Get operations for a specific vehicle make
#[command]
pub fn get_vehicle_operations(
    vehicle: String,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    let limit = limit.unwrap_or(500).clamp(1, 1_000);
    crate::obd::dev_log::log_debug(
        "db",
        &format!(
            "get_vehicle_operations: vehicle='{}', limit={}",
            vehicle, limit
        ),
    );
    with_db(|db| db.get_operations_for_vehicle(&vehicle, limit))
}

/// Get read operations for ECU + vehicle (Live Data / Dashboard)
#[command]
pub fn get_read_operations(
    ecu_name: String,
    vehicle: String,
) -> Result<Vec<serde_json::Value>, String> {
    crate::obd::dev_log::log_debug(
        "db",
        &format!(
            "get_read_operations: ecu='{}', vehicle='{}'",
            ecu_name, vehicle
        ),
    );
    with_db(|db| db.get_read_operations(&ecu_name, &vehicle))
}

/// Get write operations for ECU + vehicle (Advanced page)
#[command]
pub fn get_write_operations(
    ecu_name: String,
    vehicle: String,
) -> Result<Vec<serde_json::Value>, String> {
    crate::obd::dev_log::log_debug(
        "db",
        &format!(
            "get_write_operations: ecu='{}', vehicle='{}'",
            ecu_name, vehicle
        ),
    );
    with_db(|db| db.get_write_operations(&ecu_name, &vehicle))
}

/// Get all vehicle profiles
#[command]
pub fn get_vehicle_profiles() -> Result<Vec<String>, String> {
    crate::obd::dev_log::log_debug("db", "get_vehicle_profiles");
    with_db(|db| db.get_vehicle_profiles())
}

/// Get ECUs for a vehicle profile
#[command]
pub fn get_profile_ecus(profile_name: String) -> Result<Vec<serde_json::Value>, String> {
    crate::obd::dev_log::log_debug(
        "db",
        &format!("get_profile_ecus: profile='{}'", profile_name),
    );
    with_db(|db| db.get_profile_ecus(&profile_name))
}

/// Search ECU catalog
#[command]
pub fn search_ecu_catalog(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    let limit = limit.unwrap_or(50).clamp(1, 500);
    crate::obd::dev_log::log_debug(
        "db",
        &format!("search_ecu_catalog: query='{}', limit={}", query, limit),
    );
    with_db(|db| db.search_ecu_catalog(&query, limit))
}

/// Get all sessions
#[command]
pub fn get_sessions_cmd() -> Result<Vec<serde_json::Value>, String> {
    crate::obd::dev_log::log_debug("db", "get_sessions_cmd");
    with_db(|db| db.get_sessions())
}

/// Delete a session
#[command]
pub fn delete_session_cmd(id: i64) -> Result<(), String> {
    crate::obd::dev_log::log_info("db", &format!("delete_session_cmd: id={}", id));
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
    data: String,
) -> Result<i64, String> {
    if vin.len() > 17
        || make.len() > 100
        || model.len() > 100
        || notes.len() > 64 * 1024
        || data.len() > 5 * 1024 * 1024
    {
        return Err("Session data exceeds allowed limits".to_string());
    }
    crate::obd::dev_log::log_info(
        "db",
        &format!(
            "save_session_cmd: vin='{}', make='{}', model='{}', dtc_count={}",
            vin, make, model, dtc_count
        ),
    );
    with_db(|db| db.save_session(&vin, &make, &model, dtc_count, &notes, &data))
}

/// Try to find a vehicle model name from the database
#[command]
pub fn find_vehicle_model(make: String) -> Option<String> {
    crate::obd::dev_log::log_debug("db", &format!("find_vehicle_model: make='{}'", make));
    with_db(|db| Ok(db.find_vehicle_model(&make)))
        .ok()
        .flatten()
}

/// Sync helper for finding vehicle model (used from connection.rs inside spawn_blocking)
pub fn find_vehicle_model_sync(make: &str) -> Option<String> {
    with_db(|db| Ok(db.find_vehicle_model(make))).ok().flatten()
}

pub fn get_did_info_batch_sync(
    dids: &[String],
    vehicle: &str,
) -> std::collections::HashMap<String, (String, String, String)> {
    with_db(|db| Ok(db.get_did_info_batch(dids, vehicle))).unwrap_or_default()
}

/// Sync helper for saving DTCs (used from dtc.rs)
pub fn save_dtc_sync(
    code: &str,
    desc: &str,
    status: &str,
    source: &str,
    vin: &str,
) -> Result<(), String> {
    with_db(|db| db.save_dtc(code, desc, status, source, vin))
}

/// Get distinct ECU names that have DTC operations for a vehicle
pub fn get_dtc_ecu_names_sync(vehicle: &str) -> Vec<String> {
    with_db(|db| {
        let results = db.search_dtc_context(vehicle);
        Ok(results
            .into_iter()
            .map(|(_, _, ecu)| ecu)
            .filter(|e| !e.is_empty())
            .collect::<Vec<_>>())
    })
    .unwrap_or_default()
}
