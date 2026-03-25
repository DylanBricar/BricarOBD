use tauri::command;
use crate::models::PidValue;
use crate::obd::demo::DemoConnection;
use crate::obd::pid;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

static DEMO: Mutex<Option<DemoConnection>> = Mutex::new(None);

// History buffer for real mode PIDs — VecDeque for O(1) pop_front
static PID_HISTORY: Mutex<Option<HashMap<u16, VecDeque<f64>>>> = Mutex::new(None);

// Track which PIDs consistently fail — skip them after N failures to speed up polling
static PID_FAIL_COUNT: Mutex<Option<HashMap<u16, u32>>> = Mutex::new(None);
const MAX_PID_FAILURES: u32 = 3; // Skip PID after 3 consecutive failures

// Discovered PIDs/DIDs — populated once by discover_vehicle_params, then used for polling
static DISCOVERED_PIDS: Mutex<Option<Vec<u8>>> = Mutex::new(None);
static DISCOVERED_DIDS: Mutex<Option<Vec<(String, String)>>> = Mutex::new(None); // (hex_id, name)

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
    let mut consecutive_timeouts = 0;

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

        if consecutive_timeouts >= 5 {
            dev_log::log_warn("dashboard", "5 consecutive timeouts — attempting bus recovery");
            let _ = with_real_connection(|conn| conn.attempt_recovery());
            consecutive_timeouts = 0;
        }

        match with_real_connection(|conn| conn.query_pid(0x01, pid_u8)) {
            Ok(bytes) => {
                fail_updates.push((def.pid, true));
                consecutive_timeouts = 0;
                raw_results.push((def.pid, def.name.clone(), def.unit.clone(), bytes));
            }
            Err(e) => {
                fail_updates.push((def.pid, false));
                if e.contains("Timeout") { consecutive_timeouts += 1; }
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
                history: hist.iter().cloned().collect(),
                timestamp: now,
            });
        }
    }

    results
}

/// Get current PID data — real or demo
#[command]
pub fn get_pid_data() -> Vec<PidValue> {
    if is_demo() {
        dev_log::log_debug("dashboard", "Demo mode: returning simulated PID data");
        let mut demo = DEMO.lock().unwrap_or_else(|e| e.into_inner());
        if demo.is_none() {
            *demo = Some(DemoConnection::new());
        }
        return demo.as_mut().map(|d| { d.refresh_lang(); d.get_pid_data() }).unwrap_or_default();
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

    dev_log::log_info("dashboard", &format!(
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
pub fn get_pid_data_extended(manufacturer: String) -> Vec<PidValue> {
    // Start with standard OBD-II PIDs
    let mut results = get_pid_data();

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

    // Step 4: Decode values + update history (short lock)
    let mut success_count = 0;
    {
        let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
        let history = history_guard.get_or_insert_with(HashMap::new);

        for (did_id, did_name, response_bytes) in &did_results {
            let value = if response_bytes.len() >= 2 {
                (response_bytes[0] as f64) * 256.0 + (response_bytes[1] as f64)
            } else if response_bytes.len() == 1 {
                response_bytes[0] as f64
            } else {
                continue;
            };

            let hist = history.entry(*did_id).or_insert_with(VecDeque::new);
            hist.push_back(value);
            if hist.len() > 120 { hist.pop_front(); }

            let min = hist.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = hist.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            results.push(PidValue {
                pid: *did_id,
                name: did_name.clone(),
                value,
                unit: String::new(),
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

/// Discover all supported parameters for the connected vehicle (run ONCE after connection)
/// Returns { standardPids: count, manufacturerDids: count }
#[command]
pub fn discover_vehicle_params(manufacturer: String) -> serde_json::Value {
    if is_demo() {
        // In demo, "discover" all demo PIDs + a subset of manufacturer DIDs
        let demo_pids: Vec<u8> = vec![0x0C, 0x0D, 0x05, 0x04, 0x0F, 0x10, 0x11, 0x0B, 0x0E, 0x2F, 0x42, 0x46, 0x33, 0x06, 0x07, 0x1F];
        {
            let mut guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
            *guard = Some(demo_pids.clone());
        }
        let demo_dids: Vec<(String, String)> = Vec::new();
        {
            let mut guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
            *guard = Some(demo_dids);
        }
        dev_log::log_info("dashboard", &format!("Demo discovery: {} PIDs", demo_pids.len()));
        return serde_json::json!({ "standardPids": demo_pids.len(), "manufacturerDids": 0 });
    }

    dev_log::log_info("dashboard", &format!("Starting vehicle parameter discovery for '{}'", manufacturer));

    // === Phase 1: Discover standard OBD-II PIDs via bitmap ===
    let supported_pids: Vec<u8> = with_real_connection(|conn| {
        Ok(conn.supported_pids.clone())
    }).unwrap_or_default();

    // If bitmap is empty, do a manual probe of common PIDs
    let standard_pids = if supported_pids.is_empty() {
        dev_log::log_info("dashboard", "No PID bitmap — probing common PIDs manually");
        let common_pids: Vec<u8> = vec![
            0x04, 0x05, 0x06, 0x07, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
            0x11, 0x1F, 0x2F, 0x33, 0x42, 0x46, 0x5C,
        ];
        let mut found = Vec::new();
        for pid in &common_pids {
            match with_real_connection(|conn| conn.query_pid(0x01, *pid)) {
                Ok(_) => { found.push(*pid); }
                Err(_) => {}
            }
        }
        found
    } else {
        supported_pids
    };

    dev_log::log_info("dashboard", &format!("Discovered {} standard PIDs", standard_pids.len()));
    {
        let mut guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(standard_pids.clone());
    }

    // === Phase 2: Discover manufacturer-specific DIDs ===
    let mut discovered_dids: Vec<(String, String)> = Vec::new();

    if !manufacturer.is_empty() {
        let all_dids = crate::obd::ecu_profiles::get_dids_for_manufacturer(&manufacturer);
        dev_log::log_info("dashboard", &format!("Probing {} candidate manufacturer DIDs", all_dids.len()));

        let mut consecutive_failures = 0;
        for (did_hex, did_name) in &all_dids {
            if consecutive_failures >= 15 {
                dev_log::log_warn("dashboard", "15 consecutive DID failures — stopping discovery");
                break;
            }

            let did_id = match u16::from_str_radix(did_hex, 16).or_else(|_| u16::from_str_radix(did_hex, 10)) {
                Ok(id) if id >= 0x100 => id,
                _ => continue,
            };

            let cmd = format!("22{:04X}", did_id);
            match with_real_connection(|conn| conn.send_command_timeout(&cmd, 2000)) {
                Ok(response) => {
                    if response.contains("62") && !response.contains("7F") && !response.contains("NO DATA") {
                        discovered_dids.push((did_hex.clone(), did_name.clone()));
                        consecutive_failures = 0;
                    } else {
                        consecutive_failures += 1;
                    }
                }
                Err(_) => { consecutive_failures += 1; }
            }

            std::thread::sleep(std::time::Duration::from_millis(30));
        }
    }

    dev_log::log_info("dashboard", &format!("Discovered {} manufacturer DIDs", discovered_dids.len()));
    let did_count = discovered_dids.len();
    {
        let mut guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(discovered_dids);
    }

    serde_json::json!({
        "standardPids": standard_pids.len(),
        "manufacturerDids": did_count
    })
}

/// Reset the PID failure blacklist (call when reconnecting or switching vehicles)
#[command]
pub fn reset_pid_blacklist() {
    let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
    *fail_guard = Some(HashMap::new());
    dev_log::log_info("dashboard", "PID failure blacklist reset");
}

/// Clear both PID_HISTORY and PID_FAIL_COUNT statics (call on disconnect)
pub fn clear_pid_history() {
    let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
    *history_guard = None;

    let mut fail_guard = PID_FAIL_COUNT.lock().unwrap_or_else(|e| e.into_inner());
    *fail_guard = None;

    let mut pids_guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
    *pids_guard = None;

    let mut dids_guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
    *dids_guard = None;

    dev_log::log_info("connection", "PID history and discovery data cleared");
}

/// Get battery voltage from adapter
#[command]
pub fn get_battery_voltage() -> Option<f64> {
    if is_demo() {
        return Some(14.1);
    }
    with_real_connection(|conn| {
        Ok(conn.get_voltage())
    }).unwrap_or(None)
}

/// Start CSV recording (returns filename)
#[command]
pub fn start_recording() -> Result<String, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("recording_{}.csv", timestamp);
    dev_log::log_info("dashboard", &format!("CSV recording started: {}", filename));
    Ok(filename)
}

/// Stop CSV recording
#[command]
pub fn stop_recording() -> Result<(), String> {
    dev_log::log_info("dashboard", "CSV recording stopped");
    Ok(())
}
