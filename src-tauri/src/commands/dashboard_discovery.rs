use tauri::command;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};
use std::sync::Mutex;

// Discovered PIDs/DIDs — populated once by discover_vehicle_params, then used for polling
pub static DISCOVERED_PIDS: Mutex<Option<Vec<u8>>> = Mutex::new(None);
pub static DISCOVERED_DIDS: Mutex<Option<Vec<(String, String)>>> = Mutex::new(None); // (hex_id, name)

// Discovery progress tracking (current, total, phase)
static DISCOVERY_PROGRESS: Mutex<(u32, u32, String)> = Mutex::new((0, 0, String::new()));

/// Update discovery progress
#[allow(dead_code)]
fn update_progress(current: u32, total: u32, phase: &str) {
    let mut guard = DISCOVERY_PROGRESS.lock().unwrap_or_else(|e| e.into_inner());
    *guard = (current, total, phase.to_string());
}

/// Discover all supported parameters for the connected vehicle (run ONCE after connection)
/// Returns { standardPids: count, manufacturerDids: count }
#[command]
pub async fn discover_vehicle_params(manufacturer: String) -> serde_json::Value {
    tokio::task::spawn_blocking(move || {
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
            let mut last_keepalive = std::time::Instant::now();
            for pid in &common_pids {
                if last_keepalive.elapsed() > std::time::Duration::from_secs(4) {
                    let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
                    last_keepalive = std::time::Instant::now();
                }
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
            let mut last_keepalive = std::time::Instant::now();
            for (did_hex, did_name) in &all_dids {
                if consecutive_failures >= 15 {
                    dev_log::log_warn("dashboard", "15 consecutive DID failures — stopping discovery");
                    break;
                }

                if last_keepalive.elapsed() > std::time::Duration::from_secs(4) {
                    let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
                    last_keepalive = std::time::Instant::now();
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
    }).await.unwrap_or_else(|e| {
        dev_log::log_error("dashboard", &format!("discover_vehicle_params task failed: {}", e));
        serde_json::json!({})
    })
}

/// Reset discovered parameters (call from connection when clearing cache)
/// Lock ordering: DISCOVERED_DIDS → DISCOVERED_PIDS (matches clear_pid_history to prevent deadlocks)
pub fn reset_discovered_params_inner() {
    let mut dids_guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
    *dids_guard = None;

    let mut pids_guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
    *pids_guard = None;

    dev_log::log_info("dashboard", "Discovered parameters reset");
}

/// Get current discovery progress
#[command]
pub fn get_discovery_progress() -> serde_json::Value {
    let guard = DISCOVERY_PROGRESS.lock().unwrap_or_else(|e| e.into_inner());
    let (current, total, phase) = guard.clone();
    serde_json::json!({
        "current": current,
        "total": total,
        "phase": phase,
    })
}
