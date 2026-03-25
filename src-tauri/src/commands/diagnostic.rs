use tauri::command;
use crate::models::{Mode06Result, FreezeFrameData, PidValue};
use crate::obd::{demo::DemoConnection, pid, dtc, dev_log};
use crate::commands::connection::{is_demo, with_real_connection};

/// Helper for bilingual names
fn pn(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
}

/// Get bilingual Mode 06 test name and unit
fn get_mode06_name(tid: u8, mid: u8, lang: &str) -> (String, String) {
    match (tid, mid) {
        (0x01, _) => (pn(lang, "Raté d'allumage Cyl.1", "Misfire Cylinder 1"), "count".into()),
        (0x02, _) => (pn(lang, "Raté d'allumage Cyl.2", "Misfire Cylinder 2"), "count".into()),
        (0x03, _) => (pn(lang, "Raté d'allumage Cyl.3", "Misfire Cylinder 3"), "count".into()),
        (0x04, _) => (pn(lang, "Raté d'allumage Cyl.4", "Misfire Cylinder 4"), "count".into()),
        (0x05, _) => (pn(lang, "Raté d'allumage Cyl.5", "Misfire Cylinder 5"), "count".into()),
        (0x06, _) => (pn(lang, "Raté d'allumage Cyl.6", "Misfire Cylinder 6"), "count".into()),
        (0x21, 0x01) => (pn(lang, "Efficacité catalyseur B1", "Catalyst Efficiency B1"), "ratio".into()),
        (0x21, 0x02) => (pn(lang, "Efficacité catalyseur B2", "Catalyst Efficiency B2"), "ratio".into()),
        (0x29, 0x11) => (pn(lang, "Temps réponse O2 B1S1", "O2 Response Time B1S1"), "ms".into()),
        (0x29, 0x21) => (pn(lang, "Temps réponse O2 B1S2", "O2 Response Time B1S2"), "ms".into()),
        (0x2A, 0x11) => (pn(lang, "Amplitude O2 B1S1", "O2 Amplitude B1S1"), "V".into()),
        (0x31, _) => (pn(lang, "Fuite EVAP (grande)", "EVAP Large Leak"), "Pa".into()),
        (0x32, _) => (pn(lang, "Fuite EVAP (petite)", "EVAP Small Leak"), "Pa".into()),
        (0x35, _) => (pn(lang, "Purge EVAP", "EVAP Purge Flow"), "%".into()),
        (0x3C, _) => (pn(lang, "Débit EGR", "EGR Flow Rate"), "%".into()),
        (0x3D, _) => (pn(lang, "Erreur EGR", "EGR Error"), "%".into()),
        _ => (format!("Test {:02X}/{:02X}", tid, mid), "raw".into()),
    }
}

/// Parse Mode 06 availability bitmap from "46 00 XX XX XX XX" response
fn parse_mode06_bitmap(response: &str) -> Vec<u8> {
    let bytes: Vec<u8> = response.split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    // Find "46 00" prefix
    let start = bytes.windows(2).position(|w| w[0] == 0x46 && w[1] == 0x00);
    let start = match start {
        Some(s) => s + 2,
        None => return Vec::new(),
    };

    let bitmap_bytes = &bytes[start..];
    let mut supported = Vec::new();

    for (byte_idx, &byte) in bitmap_bytes.iter().enumerate() {
        for bit in 0..8 {
            if byte & (0x80 >> bit) != 0 {
                let tid = (byte_idx * 8 + bit + 1) as u8;
                if tid <= 0xA0 {
                    supported.push(tid);
                }
            }
        }
    }

    supported
}

/// Parse a single Mode 06 response line into results
fn parse_mode06_results(response: &str, lang: &str) -> Vec<Mode06Result> {
    let mut results = Vec::new();

    for line in response.lines() {
        let bytes: Vec<u8> = line.split_whitespace()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();

        // Need at least: 46 TID MID TV_hi TV_lo MIN_hi MIN_lo MAX_hi MAX_lo = 9 bytes
        // Find "46" marker
        let pos = match bytes.iter().position(|&b| b == 0x46) {
            Some(p) => p,
            None => continue,
        };

        if pos + 8 >= bytes.len() { continue; }

        let tid = bytes[pos + 1];
        if tid == 0x00 { continue; } // Skip bitmap response

        let mid = bytes[pos + 2];
        let test_value = (bytes[pos + 3] as f64) * 256.0 + (bytes[pos + 4] as f64);
        let min_limit = (bytes[pos + 5] as f64) * 256.0 + (bytes[pos + 6] as f64);
        let max_limit = (bytes[pos + 7] as f64) * 256.0 + (bytes[pos + 8] as f64);

        let (name, unit) = get_mode06_name(tid, mid, lang);
        let passed = test_value >= min_limit && test_value <= max_limit;

        results.push(Mode06Result {
            tid, mid, name, unit, test_value, min_limit, max_limit, passed,
        });
    }

    results
}

/// Get Mode 06 On-Board Monitoring Test Results
#[command]
pub fn get_mode06_results(lang: Option<String>) -> Vec<Mode06Result> {
    let lang = lang.as_deref().unwrap_or("en");

    if is_demo() {
        dev_log::log_info("diagnostic", "Demo mode: returning simulated Mode 06 results");
        return DemoConnection::get_mode06_results(lang);
    }

    dev_log::log_info("diagnostic", "Real mode: reading Mode 06 test results");

    // Step 1: Query supported TIDs via "0600"
    let bitmap_response = match with_real_connection(|conn| conn.send_command_timeout("0600", 5000)) {
        Ok(r) => r,
        Err(e) => {
            dev_log::log_warn("diagnostic", &format!("Mode 06 bitmap query failed: {}", e));
            return Vec::new();
        }
    };

    if bitmap_response.contains("NO DATA") || bitmap_response.is_empty() {
        dev_log::log_warn("diagnostic", "Mode 06 not supported by this vehicle");
        return Vec::new();
    }

    let supported_tids = parse_mode06_bitmap(&bitmap_response);
    dev_log::log_info("diagnostic", &format!("Mode 06: {} supported TIDs found", supported_tids.len()));

    if supported_tids.is_empty() {
        return Vec::new();
    }

    // Step 2: Query each supported TID
    let mut all_results = Vec::new();

    for tid in &supported_tids {
        let cmd = format!("06{:02X}", tid);
        match with_real_connection(|conn| conn.send_command_timeout(&cmd, 5000)) {
            Ok(response) => {
                if response.contains("NO DATA") || response.is_empty() {
                    continue;
                }
                let results = parse_mode06_results(&response, lang);
                if !results.is_empty() {
                    dev_log::log_debug("diagnostic", &format!("TID {:02X}: {} results", tid, results.len()));
                    all_results.extend(results);
                }
            }
            Err(e) => {
                dev_log::log_debug("diagnostic", &format!("TID {:02X} failed: {}", tid, e));
                continue;
            }
        }

        // Small delay between TID queries to avoid buffer overflow on clone adapters
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    dev_log::log_info("diagnostic", &format!("Mode 06 complete: {} test results", all_results.len()));
    all_results
}

/// Get Mode 02 Freeze Frame data
#[command]
pub fn get_freeze_frame(lang: Option<String>) -> Option<FreezeFrameData> {
    let lang = lang.as_deref().unwrap_or("en");

    if is_demo() {
        dev_log::log_info("diagnostic", "Demo mode: returning simulated freeze frame");
        return DemoConnection::get_freeze_frame(lang);
    }

    dev_log::log_info("diagnostic", "Real mode: reading Mode 02 freeze frame");

    // Step 1: Read the DTC that triggered freeze frame (PID 02)
    let dtc_response = match with_real_connection(|conn| conn.send_command_timeout("020200", 3000)) {
        Ok(r) => r,
        Err(e) => {
            dev_log::log_warn("diagnostic", &format!("Freeze frame DTC query failed: {}", e));
            return None;
        }
    };

    if dtc_response.contains("NO DATA") || dtc_response.is_empty() {
        dev_log::log_info("diagnostic", "No freeze frame stored (no active DTC)");
        return None;
    }

    // Parse DTC from "42 02 XX XX" response
    let dtc_bytes: Vec<u8> = dtc_response.split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    // Find "42 02" prefix
    let dtc_code = if let Some(pos) = dtc_bytes.windows(2).position(|w| w[0] == 0x42 && w[1] == 0x02) {
        if pos + 4 <= dtc_bytes.len() {
            dtc::decode_dtc_bytes(dtc_bytes[pos + 2], dtc_bytes[pos + 3])
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    dev_log::log_info("diagnostic", &format!("Freeze frame triggered by DTC: {}", dtc_code));

    // Step 2: Read key PIDs from freeze frame
    // Only read the most relevant PIDs to keep scan time reasonable (~20 PIDs)
    let key_pids: Vec<u16> = vec![
        0x04, 0x05, 0x06, 0x07, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        0x11, 0x2F, 0x33, 0x42, 0x44, 0x46, 0x5C, 0x5E,
    ];

    let definitions = pid::get_pid_definitions(lang);
    let mut pids = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    for def in &definitions {
        if !key_pids.contains(&def.pid) {
            continue;
        }

        let pid_u8 = def.pid as u8;
        let cmd = format!("02{:02X}00", pid_u8);

        match with_real_connection(|conn| conn.send_command_timeout(&cmd, 2000)) {
            Ok(response) => {
                if response.contains("NO DATA") || response.is_empty() {
                    continue;
                }

                // Parse response: find "42 XX" where XX = our PID
                let tokens: Vec<&str> = response.split_whitespace().collect();
                let prefix_42 = format!("{:02X}", 0x42);
                let prefix_pid = format!("{:02X}", pid_u8);

                if let Some(pos) = tokens.windows(2).position(|w| w[0].eq_ignore_ascii_case(&prefix_42) && w[1].eq_ignore_ascii_case(&prefix_pid)) {
                    let data_bytes: Vec<u8> = tokens[pos+2..]
                        .iter()
                        .filter_map(|s| u8::from_str_radix(s, 16).ok())
                        .collect();

                    if let Some(value) = pid::decode_pid(def.pid, &data_bytes) {
                        pids.push(PidValue {
                            pid: def.pid,
                            name: def.name.clone(),
                            value,
                            unit: def.unit.clone(),
                            min: def.min,
                            max: def.max,
                            history: vec![],
                            timestamp: now,
                        });
                    }
                }
            }
            Err(_) => continue,
        }
    }

    dev_log::log_info("diagnostic", &format!("Freeze frame: {} PIDs read for DTC {}", pids.len(), dtc_code));

    if pids.is_empty() && dtc_code == "Unknown" {
        return None;
    }

    Some(FreezeFrameData {
        dtc_code,
        frame_number: 0,
        pids,
    })
}

/// Discover which manufacturer DIDs the vehicle actually supports
/// Scans the given DID list and returns only the ones that respond
#[command]
pub fn discover_vehicle_dids(manufacturer: String) -> Vec<(String, String)> {
    if is_demo() {
        dev_log::log_info("diagnostic", "Demo mode: returning full manufacturer DID list as supported");
        let dids = crate::obd::ecu_profiles::get_dids_for_manufacturer(&manufacturer);
        // In demo, return a representative subset
        return dids.into_iter().take(30).collect();
    }

    dev_log::log_info("diagnostic", &format!("Discovering supported DIDs for {}", manufacturer));

    let all_dids = crate::obd::ecu_profiles::get_dids_for_manufacturer(&manufacturer);
    if all_dids.is_empty() {
        dev_log::log_warn("diagnostic", "No known DIDs for this manufacturer");
        return Vec::new();
    }

    dev_log::log_info("diagnostic", &format!("Testing {} candidate DIDs", all_dids.len()));

    let mut supported: Vec<(String, String)> = Vec::new();
    let mut consecutive_failures = 0;

    for (did_hex, did_name) in &all_dids {
        // If too many consecutive failures, the ECU likely doesn't support extended DIDs
        if consecutive_failures >= 10 {
            dev_log::log_warn("diagnostic", "10 consecutive DID failures — stopping discovery");
            break;
        }

        let cmd = format!("22{}", did_hex);
        match with_real_connection(|conn| conn.send_command_timeout(&cmd, 2000)) {
            Ok(response) => {
                if response.contains("62") && !response.contains("7F") && !response.contains("NO DATA") {
                    supported.push((did_hex.clone(), did_name.clone()));
                    consecutive_failures = 0;
                    dev_log::log_debug("diagnostic", &format!("DID {} supported: {}", did_hex, did_name));
                } else {
                    consecutive_failures += 1;
                }
            }
            Err(_) => {
                consecutive_failures += 1;
            }
        }

        // Small delay to avoid overwhelming the adapter
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    dev_log::log_info("diagnostic", &format!("DID discovery complete: {}/{} supported", supported.len(), all_dids.len()));
    supported
}
