use tauri::command;
use crate::models::PidValue;
use crate::obd::demo::DemoConnection;
use crate::obd::pid;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};

use std::collections::HashMap;
use std::sync::Mutex;

static DEMO: Mutex<Option<DemoConnection>> = Mutex::new(None);

// History buffer for real mode PIDs
static PID_HISTORY: Mutex<Option<HashMap<u16, Vec<f64>>>> = Mutex::new(None);

/// Get current PID data — real or demo
#[command]
pub fn get_pid_data() -> Vec<PidValue> {
    if is_demo() {
        dev_log::log_debug("dashboard", "Demo mode: returning simulated PID data");
        let mut demo = DEMO.lock().unwrap_or_else(|e| e.into_inner());
        if demo.is_none() {
            *demo = Some(DemoConnection::new());
        }
        return demo.as_mut().map(|d| d.get_pid_data()).unwrap_or_default();
    }

    dev_log::log_debug("dashboard", "Real mode: querying live PID data");

    // Real vehicle: query supported PIDs
    let definitions = pid::get_pid_definitions();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut results = Vec::new();
    let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
    if history_guard.is_none() {
        *history_guard = Some(HashMap::new());
    }
    let history = history_guard.as_mut().unwrap();

    let mut success_count = 0;
    let mut fail_count = 0;

    for def in &definitions {
        // Query Mode 01 PID
        let response = match with_real_connection(|conn| {
            conn.query_pid(0x01, def.pid as u8)
        }) {
            Ok(bytes) => bytes,
            Err(_) => {
                fail_count += 1;
                continue; // PID not supported — skip silently
            }
        };

        // Decode value
        if let Some(value) = pid::decode_pid(def.pid, &response) {
            // Update history
            let hist = history.entry(def.pid).or_insert_with(Vec::new);
            hist.push(value);
            if hist.len() > 120 { hist.remove(0); } // Keep 2 minutes at 1Hz

            // Track min/max
            let min = hist.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = hist.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            results.push(PidValue {
                pid: def.pid,
                name: def.name.clone(),
                value,
                unit: def.unit.clone(),
                min,
                max,
                history: hist.clone(),
                timestamp: now,
            });
            success_count += 1;
        }
    }

    dev_log::log_info("dashboard", &format!("PID data: {} successful, {} failed", success_count, fail_count));
    results
}

/// Get all supported PIDs definitions
#[command]
pub fn get_all_pids() -> Vec<crate::models::PidDefinition> {
    let pids = pid::get_pid_definitions();
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

    // Add manufacturer-specific DIDs via UDS Service 0x22
    let dids = crate::obd::ecu_profiles::get_dids_for_manufacturer(&manufacturer);
    dev_log::log_debug("dashboard", &format!("Available DIDs for {}: {}", manufacturer, dids.len()));

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut history_guard = PID_HISTORY.lock().unwrap_or_else(|e| e.into_inner());
    let history = history_guard.get_or_insert_with(HashMap::new);

    let mut success_count = 0;
    for (did_hex, did_name) in &dids {
        // Parse DID hex string to u16
        let did_id = match u16::from_str_radix(did_hex, 16).or_else(|_| u16::from_str_radix(did_hex, 10)) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Skip if DID ID conflicts with standard PID range (0x00-0xFF)
        if did_id < 0x100 {
            continue;
        }

        // Send UDS 0x22 + DID
        let cmd = format!("22{:04X}", did_id);
        let response = match with_real_connection(|conn| conn.send_command(&cmd)) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Parse response: "62 XX XX YY YY ..."
        if response.contains("62") {
            let bytes: Vec<u8> = response
                .split_whitespace()
                .skip(3) // Skip "62 XX XX"
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();

            // Decode as numeric value (first 2 bytes as u16, or 1 byte)
            let value = if bytes.len() >= 2 {
                (bytes[0] as f64) * 256.0 + (bytes[1] as f64)
            } else if bytes.len() == 1 {
                bytes[0] as f64
            } else {
                continue;
            };

            let hist = history.entry(did_id).or_insert_with(Vec::new);
            hist.push(value);
            if hist.len() > 120 { hist.remove(0); }

            let min = hist.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = hist.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            results.push(PidValue {
                pid: did_id,
                name: did_name.clone(),
                value,
                unit: String::new(), // DID units not available in the profile data
                min,
                max,
                history: hist.clone(),
                timestamp: now,
            });
            success_count += 1;
        }
    }

    let standard_count = results.len() - success_count.min(results.len());
    dev_log::log_info("dashboard", &format!("Extended polling: {} standard PIDs + {} manufacturer DIDs", standard_count, success_count));

    results
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
