use tauri::command;
use crate::models::{Mode06Result, FreezeFrameData, PidValue};
use crate::obd::{demo::DemoConnection, pid, dtc, dev_log};
use crate::commands::connection::{is_demo, with_real_connection};

/// Helper for bilingual names
fn pn(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
}

/// Get bilingual Mode 06 test name and unit — comprehensive TID/MID coverage
fn get_mode06_name(tid: u8, mid: u8, lang: &str) -> (String, String) {
    match (tid, mid) {
        // === Misfire Monitoring (TID 0x01-0x0A) ===
        (0x01, _) => (pn(lang, "Raté d'allumage Cyl.1", "Misfire Cylinder 1"), "count".into()),
        (0x02, _) => (pn(lang, "Raté d'allumage Cyl.2", "Misfire Cylinder 2"), "count".into()),
        (0x03, _) => (pn(lang, "Raté d'allumage Cyl.3", "Misfire Cylinder 3"), "count".into()),
        (0x04, _) => (pn(lang, "Raté d'allumage Cyl.4", "Misfire Cylinder 4"), "count".into()),
        (0x05, _) => (pn(lang, "Raté d'allumage Cyl.5", "Misfire Cylinder 5"), "count".into()),
        (0x06, _) => (pn(lang, "Raté d'allumage Cyl.6", "Misfire Cylinder 6"), "count".into()),
        (0x07, _) => (pn(lang, "Raté d'allumage Cyl.7", "Misfire Cylinder 7"), "count".into()),
        (0x08, _) => (pn(lang, "Raté d'allumage Cyl.8", "Misfire Cylinder 8"), "count".into()),
        (0x09, _) => (pn(lang, "Raté d'allumage général", "General Misfire"), "count".into()),
        (0x0A, _) => (pn(lang, "Raté d'allumage aléatoire", "Random Misfire"), "count".into()),

        // === Fuel System Monitoring (TID 0x11-0x16) ===
        (0x11, _) => (pn(lang, "Correction carburant court terme B1", "Short Term Fuel Trim B1"), "%".into()),
        (0x12, _) => (pn(lang, "Correction carburant long terme B1", "Long Term Fuel Trim B1"), "%".into()),
        (0x13, _) => (pn(lang, "Correction carburant court terme B2", "Short Term Fuel Trim B2"), "%".into()),
        (0x14, _) => (pn(lang, "Correction carburant long terme B2", "Long Term Fuel Trim B2"), "%".into()),
        (0x15, _) => (pn(lang, "Pression système carburant", "Fuel System Pressure"), "kPa".into()),
        (0x16, _) => (pn(lang, "Pression rampe carburant", "Fuel Rail Pressure"), "kPa".into()),

        // === Catalyst Monitoring (TID 0x21-0x24) ===
        (0x21, 0x01) => (pn(lang, "Efficacité catalyseur B1", "Catalyst Efficiency B1"), "ratio".into()),
        (0x21, 0x02) => (pn(lang, "Efficacité catalyseur B2", "Catalyst Efficiency B2"), "ratio".into()),
        (0x21, _) => (pn(lang, "Efficacité catalyseur", "Catalyst Efficiency"), "ratio".into()),
        (0x22, 0x01) => (pn(lang, "Vieillissement catalyseur B1", "Catalyst Aging B1"), "ratio".into()),
        (0x22, 0x02) => (pn(lang, "Vieillissement catalyseur B2", "Catalyst Aging B2"), "ratio".into()),
        (0x22, _) => (pn(lang, "Vieillissement catalyseur", "Catalyst Aging"), "ratio".into()),
        (0x23, _) => (pn(lang, "Chauffage catalyseur B1", "Catalyst Heater B1"), "s".into()),
        (0x24, _) => (pn(lang, "Chauffage catalyseur B2", "Catalyst Heater B2"), "s".into()),

        // === O2 Sensor Monitoring (TID 0x29-0x2C) ===
        (0x29, 0x11) => (pn(lang, "Temps réponse O2 B1S1", "O2 Response Time B1S1"), "ms".into()),
        (0x29, 0x21) => (pn(lang, "Temps réponse O2 B1S2", "O2 Response Time B1S2"), "ms".into()),
        (0x29, _) => (pn(lang, "Temps réponse O2", "O2 Response Time"), "ms".into()),
        (0x2A, 0x11) => (pn(lang, "Amplitude O2 B1S1", "O2 Amplitude B1S1"), "V".into()),
        (0x2A, 0x21) => (pn(lang, "Amplitude O2 B1S2", "O2 Amplitude B1S2"), "V".into()),
        (0x2A, _) => (pn(lang, "Amplitude O2", "O2 Amplitude"), "V".into()),
        (0x2B, 0x11) => (pn(lang, "Temps réponse O2 B2S1", "O2 Response Time B2S1"), "ms".into()),
        (0x2B, 0x21) => (pn(lang, "Temps réponse O2 B2S2", "O2 Response Time B2S2"), "ms".into()),
        (0x2B, _) => (pn(lang, "Temps réponse O2 B2", "O2 Response Time B2"), "ms".into()),
        (0x2C, 0x11) => (pn(lang, "Amplitude O2 B2S1", "O2 Amplitude B2S1"), "V".into()),
        (0x2C, 0x21) => (pn(lang, "Amplitude O2 B2S2", "O2 Amplitude B2S2"), "V".into()),
        (0x2C, _) => (pn(lang, "Amplitude O2 B2", "O2 Amplitude B2"), "V".into()),

        // === EVAP System Monitoring (TID 0x31-0x35) ===
        (0x31, _) => (pn(lang, "Fuite EVAP (grande)", "EVAP Large Leak"), "Pa".into()),
        (0x32, _) => (pn(lang, "Fuite EVAP (petite)", "EVAP Small Leak"), "Pa".into()),
        (0x33, _) => (pn(lang, "Étanchéité canister EVAP", "EVAP Canister Close"), "Pa".into()),
        (0x34, _) => (pn(lang, "Pression EVAP", "EVAP System Pressure"), "Pa".into()),
        (0x35, _) => (pn(lang, "Purge EVAP", "EVAP Purge Flow"), "%".into()),

        // === EGR Monitoring (TID 0x3C-0x3D) ===
        (0x3C, _) => (pn(lang, "Débit EGR", "EGR Flow Rate"), "%".into()),
        (0x3D, _) => (pn(lang, "Erreur EGR", "EGR Error"), "%".into()),

        // === Secondary Air Injection (TID 0x41-0x42) ===
        (0x41, _) => (pn(lang, "Injection air secondaire B1", "Secondary Air Injection B1"), "g/s".into()),
        (0x42, _) => (pn(lang, "Injection air secondaire B2", "Secondary Air Injection B2"), "g/s".into()),

        // === A/C System (TID 0x51-0x52) ===
        (0x51, _) => (pn(lang, "Système A/C réfrigérant", "A/C Refrigerant"), "g".into()),
        (0x52, _) => (pn(lang, "Pression A/C", "A/C Pressure"), "kPa".into()),

        // === Heated O2 Sensor Heater (TID 0x61-0x66) ===
        (0x61, _) => (pn(lang, "Chauffage sonde O2 B1S1", "O2 Heater B1S1"), "s".into()),
        (0x62, _) => (pn(lang, "Chauffage sonde O2 B1S2", "O2 Heater B1S2"), "s".into()),
        (0x63, _) => (pn(lang, "Chauffage sonde O2 B2S1", "O2 Heater B2S1"), "s".into()),
        (0x64, _) => (pn(lang, "Chauffage sonde O2 B2S2", "O2 Heater B2S2"), "s".into()),
        (0x65, _) => (pn(lang, "Résistance chauffage O2 B1", "O2 Heater Resistance B1"), "Ω".into()),
        (0x66, _) => (pn(lang, "Résistance chauffage O2 B2", "O2 Heater Resistance B2"), "Ω".into()),

        // === VVT (Variable Valve Timing) (TID 0x71-0x74) ===
        (0x71, _) => (pn(lang, "Distribution variable admission B1", "VVT Intake B1"), "°".into()),
        (0x72, _) => (pn(lang, "Distribution variable échappement B1", "VVT Exhaust B1"), "°".into()),
        (0x73, _) => (pn(lang, "Distribution variable admission B2", "VVT Intake B2"), "°".into()),
        (0x74, _) => (pn(lang, "Distribution variable échappement B2", "VVT Exhaust B2"), "°".into()),

        // === Thermostat (TID 0x81) ===
        (0x81, _) => (pn(lang, "Thermostat", "Thermostat"), "°C".into()),

        // === Cold Start Emissions (TID 0x91) ===
        (0x91, _) => (pn(lang, "Réduction émissions démarrage à froid", "Cold Start Emission Reduction"), "s".into()),

        // === DPF / GPF (TID 0xA1-0xA3) ===
        (0xA1, _) => (pn(lang, "Pression différentielle FAP", "DPF Differential Pressure"), "kPa".into()),
        (0xA2, _) => (pn(lang, "Température entrée FAP", "DPF Inlet Temperature"), "°C".into()),
        (0xA3, _) => (pn(lang, "Régénération FAP", "DPF Regeneration"), "count".into()),

        // === Fallback ===
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
pub async fn get_mode06_results(lang: Option<String>) -> Vec<Mode06Result> {
    tokio::task::spawn_blocking(move || {
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
    }).await.unwrap_or_default()
}

/// Get Mode 02 Freeze Frame data — reads frames 0 through 3
#[command]
pub async fn get_freeze_frame(lang: Option<String>) -> Vec<FreezeFrameData> {
    tokio::task::spawn_blocking(move || {
        let lang = lang.as_deref().unwrap_or("en");

        if is_demo() {
            dev_log::log_info("diagnostic", "Demo mode: returning simulated freeze frame");
            return DemoConnection::get_freeze_frame(lang).into_iter().collect();
        }

        dev_log::log_info("diagnostic", "Real mode: reading Mode 02 freeze frames (0-3)");

        let definitions = pid::get_pid_definitions(lang);
        let key_pids: Vec<u16> = vec![
            0x04, 0x05, 0x06, 0x07, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
            0x11, 0x2F, 0x33, 0x42, 0x44, 0x46, 0x5C, 0x5E,
        ];
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut frames = Vec::new();
        let scan_start = std::time::Instant::now();
        const MAX_SCAN_DURATION: std::time::Duration = std::time::Duration::from_secs(30);

        for frame_num in 0u8..4 {
            // Overall timeout guard — abort after 30s to prevent UI hang
            if scan_start.elapsed() > MAX_SCAN_DURATION {
                dev_log::log_warn("diagnostic", &format!("Freeze frame scan timeout after {}s — returning {} frames", scan_start.elapsed().as_secs(), frames.len()));
                break;
            }

            // Step 1: Read the DTC that triggered this frame
            let dtc_cmd = format!("0202{:02X}", frame_num);
            let dtc_response = match with_real_connection(|conn| conn.send_command_timeout(&dtc_cmd, 3000)) {
                Ok(r) => r,
                Err(e) => {
                    dev_log::log_debug("diagnostic", &format!("Frame {} DTC query failed: {}", frame_num, e));
                    continue;
                }
            };

            if dtc_response.contains("NO DATA") || dtc_response.is_empty() {
                dev_log::log_debug("diagnostic", &format!("No freeze frame stored for frame {}", frame_num));
                continue;
            }

            // Parse DTC from "42 02 XX XX" response
            let dtc_bytes: Vec<u8> = dtc_response.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();

            let dtc_code = if let Some(pos) = dtc_bytes.windows(2).position(|w| w[0] == 0x42 && w[1] == 0x02) {
                if pos + 4 <= dtc_bytes.len() {
                    dtc::decode_dtc_bytes(dtc_bytes[pos + 2], dtc_bytes[pos + 3])
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            dev_log::log_info("diagnostic", &format!("Frame {}: triggered by DTC {}", frame_num, dtc_code));

            // Step 2: Read key PIDs for this frame
            let mut pids = Vec::new();

            for def in &definitions {
                if scan_start.elapsed() > MAX_SCAN_DURATION { break; }
                if !key_pids.contains(&def.pid) {
                    continue;
                }

                let pid_u8 = def.pid as u8;
                let cmd = format!("02{:02X}{:02X}", pid_u8, frame_num);

                match with_real_connection(|conn| conn.send_command_timeout(&cmd, 2000)) {
                    Ok(response) => {
                        if response.contains("NO DATA") || response.is_empty() {
                            continue;
                        }

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

            if !pids.is_empty() || dtc_code != "Unknown" {
                dev_log::log_info("diagnostic", &format!("Frame {}: {} PIDs read for DTC {}", frame_num, pids.len(), dtc_code));
                frames.push(FreezeFrameData {
                    dtc_code,
                    frame_number: frame_num,
                    pids,
                });
            }
        }

        dev_log::log_info("diagnostic", &format!("Freeze frame scan complete: {} frames found", frames.len()));
        frames
    }).await.unwrap_or_default()
}

