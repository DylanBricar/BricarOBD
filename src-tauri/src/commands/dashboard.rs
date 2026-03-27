use tauri::command;
use crate::models::PidValue;
use crate::obd::demo::DemoConnection;
use crate::obd::pid;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};
use super::dashboard_did::decode_did_value;
use super::dashboard_discovery::{DISCOVERED_PIDS, DISCOVERED_DIDS};

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

static DEMO: Mutex<Option<DemoConnection>> = Mutex::new(None);

// History buffer for real mode PIDs — VecDeque for O(1) pop_front
static PID_HISTORY: Mutex<Option<HashMap<u16, VecDeque<f64>>>> = Mutex::new(None);

// Track which PIDs consistently fail — skip them after N failures to speed up polling
static PID_FAIL_COUNT: Mutex<Option<HashMap<u16, u32>>> = Mutex::new(None);
const MAX_PID_FAILURES: u32 = 3; // Skip PID after 3 consecutive failures

// Cache for DID info from SQLite DB — populated once per session, avoids per-poll DB queries
// Key: DID hex string (e.g. "2282"), Value: (name_en, name_fr, ecu_name)
static DID_INFO_CACHE: Mutex<Option<HashMap<String, (String, String, String)>>> = Mutex::new(None);

/// Step 1: Lock PID_FAIL_COUNT briefly, clone snapshot, return
fn snapshot_fail_counts() -> HashMap<u16, u32> {
    let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
    fail_guard.get_or_insert_with(HashMap::new).clone()
}

/// Step 2: Query PIDs with bus recovery; return (raw_results, fail_updates, fail_count, skip_count)
fn query_all_pids(
    definitions: &[crate::models::PidDefinition],
    fail_snapshot: &HashMap<u16, u32>,
    supported_pids: &[u8],
) -> (Vec<(u16, String, String, Vec<u8>)>, Vec<(u16, bool)>, usize, usize) {
    let has_pid_bitmap = !supported_pids.is_empty();
    let mut raw_results: Vec<(u16, String, String, Vec<u8>)> = Vec::new();
    let mut fail_updates: Vec<(u16, bool)> = Vec::new();
    let mut skip_count = 0;
    let mut fail_count = 0;
    // Acquire CONNECTION mutex ONCE for entire batch
    let result: Result<Vec<(u16, bool, Vec<u8>)>, String> = with_real_connection(|conn| {
        let mut batch_results: Vec<(u16, bool, Vec<u8>)> = Vec::new();
        let mut batch_timeouts = 0;

        for def in definitions {
            let pid_u8 = def.pid as u8;

            if has_pid_bitmap && !supported_pids.contains(&pid_u8) {
                skip_count += 1;
                continue;
            }

            let pid_fails = fail_snapshot.get(&def.pid).copied().unwrap_or(0);
            if pid_fails >= MAX_PID_FAILURES {
                skip_count += 1;
                continue;
            }

            if batch_timeouts >= 5 {
                let _ = conn.attempt_recovery();
                batch_timeouts = 0;
            }

            match conn.query_pid(0x01, pid_u8) {
                Ok(bytes) => {
                    batch_timeouts = 0;
                    batch_results.push((def.pid, true, bytes));
                }
                Err(e) => {
                    if e.contains("Timeout") { batch_timeouts += 1; }
                    batch_results.push((def.pid, false, vec![]));
                }
            }
        }
        Ok(batch_results)
    });

    if let Ok(batch) = result {
        for (pid, success, bytes) in batch {
            fail_updates.push((pid, success));
            if success {
                if let Some(def) = definitions.iter().find(|d| d.pid == pid) {
                    raw_results.push((pid, def.name.clone(), def.unit.clone(), bytes));
                }
            } else {
                fail_count += 1;
            }
        }
    }

    (raw_results, fail_updates, fail_count, skip_count)
}

/// Step 3: Lock PID_FAIL_COUNT and apply success/failure updates
fn update_fail_counts(fail_updates: &[(u16, bool)]) {
    let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
    let fail_counts = fail_guard.get_or_insert_with(HashMap::new);
    for (pid, success) in fail_updates {
        if *success {
            fail_counts.remove(pid);
        } else {
            *fail_counts.entry(*pid).or_insert(0) += 1;
        }
    }
}

/// Step 4: Lock PID_HISTORY, decode raw bytes, record history, return PidValue results
fn decode_and_record_history(
    raw_results: &[(u16, String, String, Vec<u8>)],
    now: u64,
) -> Vec<PidValue> {
    let mut results = Vec::new();
    let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
    let history = history_guard.get_or_insert_with(HashMap::new);

    for (pid, name, unit, bytes) in raw_results {
        if let Some(value) = pid::decode_pid(*pid, bytes) {
            let hist = history.entry(*pid).or_insert_with(VecDeque::new);
            hist.push_back(value);
            if hist.len() > 120 { hist.pop_front(); }

            let min = hist.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = hist.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            results.push(PidValue {
                pid: *pid,
                name: name.clone(),
                value,
                unit: unit.clone(),
                min,
                max,
                history: {
                    let len = hist.len();
                    let skip = len.saturating_sub(30);
                    hist.iter().skip(skip).cloned().collect()
                },
                timestamp: now,
            });
        }
    }

    results
}

/// Get current PID data — real or demo
#[command]
pub async fn get_pid_data() -> Vec<PidValue> {
    match tokio::task::spawn_blocking(|| {
        get_pid_data_inner()
    }).await {
        Ok(data) => data,
        Err(e) => {
            dev_log::log_error("dashboard", &format!("get_pid_data task failed: {}", e));
            Vec::new()
        }
    }
}

fn get_pid_data_inner() -> Vec<PidValue> {
    if is_demo() {
        dev_log::log_debug("dashboard", "Demo mode: returning simulated PID data");
        let mut demo = DEMO.lock().unwrap_or_else(|e| e.into_inner());
        if demo.is_none() {
            *demo = Some(DemoConnection::new());
        }
        return demo.as_mut().map(|d| { d.refresh_lang(); d.get_pid_data() }).unwrap_or_default();
    }

    // Check if OBD is busy with another operation
    if super::connection::is_obd_busy() {
        dev_log::log_debug("dashboard", "OBD is busy, skipping PID poll");
        return Vec::new();
    }

    dev_log::log_debug("dashboard", "Real mode: querying live PID data");

    let lang = super::connection::get_lang();
    let definitions = pid::get_pid_definitions(&lang);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let fail_snapshot = snapshot_fail_counts();

    // Use discovered PIDs if available (from discover_vehicle_params), else fallback to bitmap
    let supported_pids: Vec<u8> = {
        let guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
        guard.clone().unwrap_or_default()
    };
    let supported_pids = if supported_pids.is_empty() {
        // Fallback: use connection bitmap
        with_real_connection(|conn| Ok(conn.supported_pids.clone())).unwrap_or_default()
    } else {
        supported_pids
    };

    let (raw_results, fail_updates, fail_count, skip_count) =
        query_all_pids(&definitions, &fail_snapshot, &supported_pids);

    update_fail_counts(&fail_updates);

    let results = decode_and_record_history(&raw_results, now);

    dev_log::log_debug("dashboard", &format!(
        "PID poll: {} ok, {} failed, {} skipped (bitmap/blacklist)",
        raw_results.len(), fail_count, skip_count
    ));
    results
}

/// Get all supported PIDs definitions
#[command]
pub fn get_all_pids() -> Vec<crate::models::PidDefinition> {
    let lang = super::connection::get_lang();
    let pids = pid::get_pid_definitions(&lang);
    dev_log::log_debug("dashboard", &format!("Retrieved {} PID definitions", pids.len()));
    pids
}

/// Get extended PID data including manufacturer-specific DIDs
#[command]
pub async fn get_pid_data_extended(manufacturer: String) -> Vec<PidValue> {
    match tokio::task::spawn_blocking(move || {
        get_pid_data_extended_inner(manufacturer)
    }).await {
        Ok(data) => data,
        Err(e) => {
            dev_log::log_error("dashboard", &format!("get_pid_data_extended task failed: {}", e));
            Vec::new()
        }
    }
}

fn get_pid_data_extended_inner(manufacturer: String) -> Vec<PidValue> {
    // Start with standard OBD-II PIDs
    let mut results = get_pid_data_inner();

    if is_demo() || manufacturer.is_empty() {
        dev_log::log_debug("dashboard", "Extended polling skipped: demo mode or empty manufacturer");
        return results;
    }

    dev_log::log_info("dashboard", &format!("Extended polling for manufacturer: {}", manufacturer));

    // Use discovered DIDs if available, else fallback to full manufacturer list
    let dids = {
        let guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
        guard.clone()
    };
    let dids = dids.unwrap_or_else(|| crate::obd::ecu_profiles::get_dids_for_manufacturer(&manufacturer));
    dev_log::log_debug("dashboard", &format!("Polling {} DIDs for {}", dids.len(), manufacturer));

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Step 1: Snapshot fail counts (short lock, then release)
    let fail_snapshot: HashMap<u16, u32>;
    {
        let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
        fail_snapshot = fail_guard.get_or_insert_with(HashMap::new).clone();
    }

    // Step 2: Query DIDs — only CONNECTION lock held per query
    let mut did_results: Vec<(u16, String, Vec<u8>)> = Vec::new();
    let mut fail_updates: Vec<(u16, bool)> = Vec::new();

    for (did_hex, did_name) in &dids {
        let did_id = match u16::from_str_radix(did_hex, 16).or_else(|_| u16::from_str_radix(did_hex, 10)) {
            Ok(id) => id,
            Err(_) => continue,
        };

        if did_id < 0x100 { continue; }

        let did_fails = fail_snapshot.get(&did_id).copied().unwrap_or(0);
        if did_fails >= MAX_PID_FAILURES { continue; }

        match with_real_connection(|conn| conn.query_did(did_id)) {
            Ok(bytes) => {
                fail_updates.push((did_id, true));
                did_results.push((did_id, did_name.clone(), bytes));
            }
            Err(_) => {
                fail_updates.push((did_id, false));
            }
        }
    }

    // Step 3: Update fail counts (short lock)
    {
        let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
        let fail_counts = fail_guard.get_or_insert_with(HashMap::new);
        for (did_id, success) in &fail_updates {
            if *success {
                fail_counts.remove(did_id);
            } else {
                *fail_counts.entry(*did_id).or_insert(0) += 1;
            }
        }
    }

    // Step 4: Build DID info cache (populated once, reused across poll cycles)
    let lang = super::connection::get_lang();
    {
        let mut cache_guard = DID_INFO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if cache_guard.is_none() {
            let mut cache = HashMap::new();
            for (did_id, _did_name, _) in &did_results {
                let did_hex = format!("{:04X}", did_id);
                if let Some(info) = super::database::get_did_info_sync(&did_hex, &manufacturer) {
                    cache.insert(did_hex, info);
                }
            }
            dev_log::log_info("dashboard", &format!("DID info cache populated: {} entries from DB", cache.len()));
            *cache_guard = Some(cache);
        }
    }

    // Step 5: Decode values + update history (short lock, no DB queries)
    let did_cache = {
        let cache_guard = DID_INFO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        cache_guard.clone().unwrap_or_default()
    };
    let mut success_count = 0;
    {
        let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
        let history = history_guard.get_or_insert_with(HashMap::new);

        for (did_id, did_name, response_bytes) in &did_results {
            // Look up from cache (no DB query in hot loop)
            let did_hex = format!("{:04X}", did_id);
            let db_info = did_cache.get(&did_hex);

            // Choose best name: DB name (localized) > ecu_profiles name > fallback
            let display_name = if let Some((name_en, name_fr, _ecu)) = db_info {
                let name = if lang == "fr" && !name_fr.is_empty() { name_fr } else { name_en };
                if !name.is_empty() { name.clone() } else { did_name.clone() }
            } else {
                did_name.clone()
            };

            // Decode value with heuristic based on name
            let (value, unit) = decode_did_value(&response_bytes, &display_name);

            let hist = history.entry(*did_id).or_insert_with(VecDeque::new);
            hist.push_back(value);
            if hist.len() > 120 { hist.pop_front(); }

            let min = hist.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = hist.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            results.push(PidValue {
                pid: *did_id,
                name: display_name,
                value,
                unit,
                min,
                max,
                history: hist.iter().cloned().collect(),
                timestamp: now,
            });
            success_count += 1;
        }
    }

    let standard_count = results.len() - success_count.min(results.len());
    dev_log::log_info("dashboard", &format!("Extended polling: {} standard PIDs + {} manufacturer DIDs", standard_count, success_count));

    results
}

// discover_vehicle_params, get_discovery_progress, and discovery statics moved to dashboard_discovery.rs

/// Reset the PID failure blacklist (call when reconnecting or switching vehicles)
#[command]
pub fn reset_pid_blacklist() {
    let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
    *fail_guard = Some(HashMap::new());
    dev_log::log_info("dashboard", "PID failure blacklist reset");
}

/// Clear both PID_HISTORY and PID_FAIL_COUNT statics (call on disconnect)
/// Lock ordering: DID_INFO_CACHE → DISCOVERED_DIDS → DISCOVERED_PIDS → PID_FAIL_COUNT → PID_HISTORY
/// This order matches get_pid_data_extended_inner to prevent deadlocks.
pub fn clear_pid_history() {
    // Release each mutex in its own scope block
    {
        let mut cache_guard = DID_INFO_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        *cache_guard = None;
    }

    {
        let mut dids_guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
        *dids_guard = None;
    }

    {
        let mut pids_guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
        *pids_guard = None;
    }

    {
        let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
        *fail_guard = None;
    }

    {
        let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
        *history_guard = None;
    }

    {
        let mut demo_guard = DEMO.lock().unwrap_or_else(|e| e.into_inner());
        *demo_guard = None;
    }

    dev_log::log_info("connection", "PID history and discovery data cleared");
}

/// Get battery voltage from adapter
#[command]
pub async fn get_battery_voltage() -> Option<f64> {
    if is_demo() {
        return Some(14.1);
    }
    if super::connection::is_obd_busy() {
        return None;
    }
    tokio::task::spawn_blocking(|| {
        with_real_connection(|conn| {
            Ok(conn.get_voltage())
        }).unwrap_or(None)
    }).await.unwrap_or(None)
}

// get_discovery_progress and reset_discovered_params_inner moved to dashboard_discovery.rs

