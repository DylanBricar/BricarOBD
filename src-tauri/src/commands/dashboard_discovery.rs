use tauri::command;
use crate::obd::dev_log;
use crate::obd::vin_cache::{self, VinCache};
use crate::commands::connection::{is_demo, with_real_connection};
use std::sync::Mutex;

// Discovered PIDs/DIDs — populated once by discover_vehicle_params, then used for polling
pub static DISCOVERED_PIDS: Mutex<Option<Vec<u8>>> = Mutex::new(None);
pub static DISCOVERED_DIDS: Mutex<Option<Vec<(String, String)>>> = Mutex::new(None); // (hex_id, name)

// Discovery progress tracking (current, total, phase)
static DISCOVERY_PROGRESS: Mutex<(u32, u32, &'static str)> = Mutex::new((0, 0, ""));

/// Update discovery progress
fn update_progress(current: u32, total: u32, phase: &'static str) {
    let mut guard = DISCOVERY_PROGRESS.lock().unwrap_or_else(|e| e.into_inner());
    *guard = (current, total, phase);
}

/// Discover all supported parameters for the connected vehicle (run ONCE after connection)
/// Uses VIN-based cache to skip re-discovery on same vehicle within 30 days
/// Returns { standardPids: count, manufacturerDids: count, fromCache: bool }
#[command]
pub async fn discover_vehicle_params(manufacturer: String, vin: String) -> serde_json::Value {
    tokio::task::spawn_blocking(move || {
        if is_demo() {
            let demo_pids: Vec<u8> = vec![0x0C, 0x0D, 0x05, 0x04, 0x0F, 0x10, 0x11, 0x0B, 0x0E, 0x2F, 0x42, 0x46, 0x33, 0x06, 0x07, 0x1F];
            {
                let mut guard = DISCOVERED_PIDS.lock().unwrap_or_else(|e| e.into_inner());
                *guard = Some(demo_pids.clone());
            }
            {
                let mut guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
                *guard = Some(Vec::new());
            }
            dev_log::log_info("dashboard", &format!("Demo discovery: {} PIDs", demo_pids.len()));
            return serde_json::json!({ "standardPids": demo_pids.len(), "manufacturerDids": 0, "fromCache": false });
        }

        // === Wait for OBD bus to be free (e.g. ECU scan / DTC scan in progress) ===
        {
            let start = std::time::Instant::now();
            let max_wait = std::time::Duration::from_secs(90);
            while super::connection_helpers::is_obd_busy() {
                if start.elapsed() > max_wait {
                    dev_log::log_warn("dashboard", "Discovery: OBD busy timeout after 90s, proceeding anyway");
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            if start.elapsed() > std::time::Duration::from_millis(500) {
                dev_log::log_info("dashboard", &format!("Discovery waited {:.1}s for OBD bus", start.elapsed().as_secs_f64()));
            }
        }

        // === Load cached failure lists (if available) ===
        let cached = if !vin.is_empty() { vin_cache::load_cache(&vin) } else { None };
        let mut failed_pids: Vec<u8> = cached.as_ref().map(|c| c.failed_pids.clone()).unwrap_or_default();
        let mut failed_dids: Vec<String> = cached.as_ref().map(|c| c.failed_dids.clone()).unwrap_or_default();
        let has_cache = cached.is_some();

        if has_cache {
            dev_log::log_info("dashboard", &format!(
                "VIN cache hit for {}: {} failed PIDs, {} failed DIDs to skip",
                vin, failed_pids.len(), failed_dids.len()
            ));
        } else {
            dev_log::log_info("dashboard", &format!("Starting full vehicle parameter discovery for '{}'", manufacturer));
        }

        // === Phase 1: Discover standard OBD-II PIDs via bitmap ===
        update_progress(0, 100, "scanning_pids");
        let supported_pids: Vec<u8> = with_real_connection(|conn| {
            Ok(conn.supported_pids.clone())
        }).unwrap_or_default();

        let standard_pids = if supported_pids.is_empty() {
            dev_log::log_info("dashboard", "No PID bitmap — probing common PIDs manually");
            let common_pids: Vec<u8> = vec![
                0x04, 0x05, 0x06, 0x07, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
                0x11, 0x1F, 0x2F, 0x33, 0x42, 0x46, 0x5C,
            ];
            let mut found = Vec::new();
            let mut last_keepalive = std::time::Instant::now();
            for pid in &common_pids {
                // Skip PIDs known to fail from cache
                if failed_pids.contains(pid) {
                    continue;
                }
                if last_keepalive.elapsed() > std::time::Duration::from_secs(4) {
                    let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
                    last_keepalive = std::time::Instant::now();
                }
                match with_real_connection(|conn| conn.query_pid(0x01, *pid)) {
                    Ok(_) => { found.push(*pid); }
                    Err(_) => { failed_pids.push(*pid); }
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
        update_progress(50, 100, "scanning_dids");

        // === Phase 2: Discover manufacturer-specific DIDs ===
        let mut discovered_dids: Vec<(String, String)> = Vec::new();

        if !manufacturer.is_empty() {
            let all_dids = crate::obd::ecu_profiles::get_dids_for_manufacturer(&manufacturer);
            // Filter out known-failed DIDs, keep new + previously-successful candidates
            let candidates: Vec<_> = all_dids.iter()
                .filter(|(hex, _)| !failed_dids.contains(hex))
                .collect();
            let skipped = all_dids.len() - candidates.len();

            dev_log::log_info("dashboard", &format!(
                "Probing {} candidate DIDs ({} skipped from fail cache)",
                candidates.len(), skipped
            ));

            let mut consecutive_failures = 0;
            let mut last_keepalive = std::time::Instant::now();
            for (did_hex, did_name) in &candidates {
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
                            discovered_dids.push((did_hex.to_string(), did_name.to_string()));
                            consecutive_failures = 0;
                        } else {
                            failed_dids.push(did_hex.to_string());
                            consecutive_failures += 1;
                        }
                    }
                    Err(_) => {
                        failed_dids.push(did_hex.to_string());
                        consecutive_failures += 1;
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(30));
            }

            // Also include previously-discovered DIDs from cache that we skipped testing
            if let Some(ref cache) = cached {
                for (hex, name) in &cache.supported_dids {
                    if !discovered_dids.iter().any(|(h, _)| h == hex) {
                        discovered_dids.push((hex.clone(), name.clone()));
                    }
                }
            }
        }

        dev_log::log_info("dashboard", &format!("Discovered {} manufacturer DIDs", discovered_dids.len()));
        let did_count = discovered_dids.len();
        update_progress(100, 100, "complete");

        // === Save to VIN cache: supportés + échoués ===
        if !vin.is_empty() {
            let mut cache = VinCache::new(vin.clone());
            cache.supported_pids = standard_pids.clone();
            cache.supported_dids = discovered_dids.clone();
            cache.failed_pids = failed_pids;
            cache.failed_dids = failed_dids;
            if let Err(e) = vin_cache::save_cache(&cache) {
                dev_log::log_warn("dashboard", &format!("Failed to save VIN cache: {}", e));
            } else {
                dev_log::log_info("dashboard", &format!("VIN cache saved for {}", vin));
            }
        }

        {
            let mut guard = DISCOVERED_DIDS.lock().unwrap_or_else(|e| e.into_inner());
            *guard = Some(discovered_dids);
        }

        serde_json::json!({
            "standardPids": standard_pids.len(),
            "manufacturerDids": did_count,
            "fromCache": has_cache
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
    let (current, total, phase) = *guard;
    serde_json::json!({
        "current": current,
        "total": total,
        "phase": phase,
    })
}
