use tauri::command;
use crate::models::{EcuInfo, MonitorStatus, PidValue, RiskLevel};
use crate::obd::demo::DemoConnection;
use crate::obd::safety::SafetyGuard;
use crate::obd::anomaly;
use crate::obd::ecu_profiles;
use crate::obd::advanced_ops;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};
use crate::commands::ecu_scan::{
    get_ecu_probes, probe_ecu_alive, read_ecu_dids, build_ecu_info
};

/// Get user's language from the global setting
fn get_user_lang() -> String {
    super::connection::get_lang()
}

/// Map operation IDs to real UDS hex commands
fn resolve_operation_command(op_id: &str) -> Option<(&str, &str)> {
    match op_id {
        "reset_service" => Some(("752", "2E 2282 00")),
        "set_service_threshold" => Some(("752", "2E 2282")),
        "write_config" => Some(("752", "2E 2100")),
        "force_regen" => Some(("7E0", "31 01 0060")),
        "test_injectors" => Some(("7E0", "30 01")),
        "test_relays" => Some(("7E0", "30 02")),
        _ => None,
    }
}


/// Scan all ECUs — probes standard OBD-II + manufacturer addresses with multi-method discovery
#[command]
pub async fn scan_ecus() -> Vec<EcuInfo> {
    tokio::task::spawn_blocking(move || {
        if is_demo() {
            dev_log::log_info("ecu", "Demo mode: returning simulated ECUs");
            let lang = get_user_lang();
            return DemoConnection::get_ecus(&lang);
        }

        dev_log::log_info("ecu", "Real mode: starting ECU scan with multi-method discovery");

        let probes = get_ecu_probes();
        let mut ecus = Vec::new();
        let mut found_addresses = std::collections::HashSet::new();
        let scan_start = std::time::Instant::now();
        let max_scan_duration = std::time::Duration::from_secs(60);

        for probe in &probes {
            if scan_start.elapsed() > max_scan_duration {
                dev_log::log_warn("ecu", "ECU scan timeout (60s) — returning partial results");
                break;
            }
            // Skip broadcast address for individual ECU detection
            if probe.tx_addr == "7DF" {
                continue;
            }

            // Skip if we already found this address (avoids duplicates from overlapping ranges)
            if found_addresses.contains(probe.tx_addr) {
                continue;
            }

            dev_log::log_debug("ecu", &format!("Probing ECU at {}", probe.tx_addr));

            // Set header to target this ECU
            if with_real_connection(|conn| conn.set_ecu_header(probe.tx_addr)).is_err() {
                continue;
            }

            if !probe_ecu_alive(probe) {
                continue;
            }

            found_addresses.insert(probe.tx_addr);

            // ECU is alive — now read DIDs to gather info
            let (dids, dids_read) = read_ecu_dids(probe);
            dev_log::log_info("ecu", &format!("ECU at {}: {} DIDs read", probe.tx_addr, dids_read));

            ecus.push(build_ecu_info(probe, &dids));
        }

        // Reset headers to broadcast
        let _ = with_real_connection(|conn| conn.reset_headers());

        if ecus.is_empty() {
            dev_log::log_warn("ecu", "No ECUs found during scan — vehicle may need ignition on");
            tracing::warn!("No ECUs found during real scan");
        } else {
            dev_log::log_info("ecu", &format!("ECU scan complete: {} ECUs found", ecus.len()));
            tracing::info!("Found {} ECUs", ecus.len());
        }

        ecus
    }).await.unwrap_or_default()
}

/// Read DID from ECU — UDS Service 0x22 with improved error handling
#[command]
pub async fn read_did(ecu_address: String, did: String) -> Result<String, String> {
    // Validate DID format: 1-4 hex characters only (prevents injection attacks)
    if did.len() > 4 || did.is_empty() || !did.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid DID format: must be 1-4 hex characters".to_string());
    }

    let risk = SafetyGuard::check_command(&format!("22 {}", did));
    dev_log::log_info("ecu", &format!("Read DID safety check: {:?}", risk));
    if risk == RiskLevel::Blocked {
        dev_log::log_warn("ecu", "Read DID blocked by safety guard");
        return Err(super::connection::err_msg("BLOQUÉ — commande bloquée par la sécurité", "BLOCKED — command blocked by safety system"));
    }

    if is_demo() {
        dev_log::log_debug("ecu", &format!("Demo mode: reading DID {} from {}", did, ecu_address));
        return Ok(format!("[DEMO] 62 {} 56 46 33 4C 43 42", did));
    }

    tokio::task::spawn_blocking(move || {
        let cmd = format!("22{}", did.replace(" ", ""));
        dev_log::log_info("ecu", &format!("Reading DID {} from ECU {}", did, ecu_address));
        let addr = ecu_address.replace("0x", "");

        // Set ECU header
        let _ = with_real_connection(|conn| conn.set_ecu_header(&addr));

        // Send TesterPresent to wake up ECU before DID read
        let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });

        // Read DID with extended timeout (DIDs can be slow on some ECUs)
        let result = with_real_connection(|conn| conn.send_command_timeout(&cmd, 5000));

        // Reset headers
        let _ = with_real_connection(|conn| conn.reset_headers());

        match result {
            Ok(r) => {
                if r.contains("NO DATA") {
                    Err(format!("DID {} not supported by ECU {}", did, ecu_address))
                } else if r.contains("7F") {
                    // Parse negative response code
                    let nrc = parse_negative_response(&r);
                    Err(format!("DID {} error: {}", did, nrc))
                } else {
                    Ok(r)
                }
            }
            Err(e) => Err(e),
        }
    }).await.map_err(|e| format!("Task error: {}", e))?
}

/// Helper to get bilingual NRC description
fn nrc_description(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
}

/// Parse UDS negative response code into human-readable string
fn parse_negative_response(response: &str) -> String {
    let lang = get_user_lang();
    let bytes: Vec<u8> = response
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if bytes.len() >= 3 && bytes[0] == 0x7F {
        let nrc = bytes[2];
        match nrc {
            0x10 => nrc_description(&lang, "Rejet général", "General reject"),
            0x11 => nrc_description(&lang, "Service non supporté", "Service not supported"),
            0x12 => nrc_description(&lang, "Sous-fonction non supportée", "Sub-function not supported"),
            0x13 => nrc_description(&lang, "Longueur de message invalide", "Invalid message length"),
            0x14 => nrc_description(&lang, "Réponse trop longue", "Response too long"),
            0x22 => nrc_description(&lang, "Conditions non remplies", "Conditions not correct"),
            0x24 => nrc_description(&lang, "Erreur de séquence de requête", "Request sequence error"),
            0x25 => nrc_description(&lang, "Pas de réponse du sous-réseau", "No response from sub-net"),
            0x26 => nrc_description(&lang, "Échec empêchant l'exécution", "Failure prevents execution"),
            0x31 => nrc_description(&lang, "Requête hors limites", "Request out of range"),
            0x33 => nrc_description(&lang, "Accès sécurité refusé", "Security access denied"),
            0x35 => nrc_description(&lang, "Clé invalide", "Invalid key"),
            0x36 => nrc_description(&lang, "Nombre de tentatives dépassé", "Exceeded number of attempts"),
            0x37 => nrc_description(&lang, "Délai requis non écoulé", "Required time delay not expired"),
            0x70 => nrc_description(&lang, "Upload/download non accepté", "Upload/download not accepted"),
            0x71 => nrc_description(&lang, "Transfert de données suspendu", "Transfer data suspended"),
            0x72 => nrc_description(&lang, "Échec de programmation général", "General programming failure"),
            0x73 => nrc_description(&lang, "Compteur de séquence de bloc incorrect", "Wrong block sequence counter"),
            0x78 => nrc_description(&lang, "Réponse en attente (traitement en cours)", "Response pending (still processing)"),
            0x7E => nrc_description(&lang, "Sous-fonction non supportée dans la session active", "Sub-function not supported in active session"),
            0x7F => nrc_description(&lang, "Service non supporté dans la session active", "Service not supported in active session"),
            _ => format!("NRC 0x{:02X}", nrc),
        }
    } else {
        response.to_string()
    }
}

/// Get OBD monitor statuses — Mode 01 PID 01, with retry and wake-up
#[command]
pub async fn get_monitors() -> Vec<MonitorStatus> {
    tokio::task::spawn_blocking(|| {
        if is_demo() {
            dev_log::log_debug("ecu", "Demo mode: returning simulated monitor statuses");
            return DemoConnection::get_monitors();
        }

        dev_log::log_info("ecu", "Real mode: reading Mode 01 PID 01 for monitor statuses");

        // Try up to 2 times — first attempt may fail if ECU is asleep
        let response = match with_real_connection(|conn| conn.query_pid(0x01, 0x01)) {
            Ok(bytes) if bytes.len() >= 4 => {
                dev_log::log_rx("0101", &format!("{:02X?}", bytes));
                bytes
            },
            _ => {
                // Retry after wake-up
                dev_log::log_warn("ecu", "PID 01 failed, trying wake-up + retry...");
                let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
                // Small delay for ECU to wake up (acceptable in sync command running on thread pool)
                std::thread::sleep(std::time::Duration::from_millis(300));

                match with_real_connection(|conn| conn.query_pid(0x01, 0x01)) {
                    Ok(bytes) if bytes.len() >= 4 => bytes,
                    _ => {
                        dev_log::log_warn("ecu", "Mode 01 PID 01 read failed after retry");
                        return Vec::new();
                    }
                }
            }
        };

        let b = response[1];
        let c = response[2];
        let d = response[3];

        let mut monitors = Vec::new();

        // Continuous monitors (byte B)
        for (bit_sup, bit_comp, key, desc, spec) in [
            (0x01, 0x10, "monitors.misfire", "monitors.misfireDesc", "monitors.misfireSpec"),
            (0x02, 0x20, "monitors.fuelSystem", "monitors.fuelSystemDesc", "monitors.fuelSystemSpec"),
            (0x04, 0x40, "monitors.components", "monitors.componentsDesc", "monitors.componentsSpec"),
        ] {
            monitors.push(MonitorStatus {
                name_key: key.into(), available: b & bit_sup != 0, complete: b & bit_comp == 0,
                description_key: Some(desc.into()), specification_key: Some(spec.into()),
            });
        }

        // Non-continuous monitors (bytes C and D)
        for (bit, key, desc, spec) in [
            (0x01, "monitors.catalystB1", "monitors.catalystB1Desc", "monitors.catalystB1Spec"),
            (0x02, "monitors.o2HeaterB1S1", "monitors.o2HeaterB1S1Desc", "monitors.o2HeaterB1S1Spec"),
            (0x04, "monitors.evap", "monitors.evapDesc", "monitors.evapSpec"),
            (0x08, "monitors.secondaryAir", "monitors.secondaryAirDesc", "monitors.secondaryAirSpec"),
            (0x10, "monitors.ac", "monitors.acDesc", "monitors.acSpec"),
            (0x20, "monitors.o2B1S1", "monitors.o2B1S1Desc", "monitors.o2B1S1Spec"),
            (0x40, "monitors.egrVvt", "monitors.egrVvtDesc", "monitors.egrVvtSpec"),
            (0x80, "monitors.catalystB2", "monitors.catalystB2Desc", "monitors.catalystB2Spec"),
        ] {
            monitors.push(MonitorStatus {
                name_key: key.into(), available: c & bit != 0, complete: d & bit == 0,
                description_key: Some(desc.into()), specification_key: Some(spec.into()),
            });
        }

        monitors
    }).await.unwrap_or_default()
}

/// Execute a UDS command against an ECU: set header → tester_present → send → log_rx → reset headers
fn execute_uds_command(addr: &str, hex_cmd: &str) -> Result<String, String> {
    let _ = with_real_connection(|conn| conn.set_ecu_header(addr));
    let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
    let result = with_real_connection(|conn| conn.send_command_timeout(hex_cmd, 8000));
    dev_log::log_rx(hex_cmd, result.as_deref().unwrap_or("(error)"));
    let _ = with_real_connection(|conn| conn.reset_headers());
    result
}

/// Send raw UDS command or named operation (Advanced mode — uses elevated safety)
#[command]
pub fn send_raw_command(ecu_address: String, command: String, confirmed: Option<bool>) -> Result<String, String> {
    if let Some((addr, hex_cmd)) = resolve_operation_command(&command) {
        dev_log::log_info("ecu", &format!("Operation resolved: {} → {}", command, hex_cmd));
        let risk = SafetyGuard::check_command_advanced(hex_cmd);
        dev_log::log_debug("ecu", &format!("Safety check result: {:?}", risk));
        if risk == RiskLevel::Blocked {
            dev_log::log_warn("ecu", "Operation blocked by safety guard");
            return Err(super::connection::err_msg("BLOQUÉ — commande bloquée par la sécurité", "BLOCKED — command blocked by safety system"));
        }
        if risk == RiskLevel::Dangerous {
            dev_log::log_warn("ecu", "Operation blocked: dangerous command");
            return Err(super::connection::err_msg("DANGEREUX — commande trop risquée", "DANGEROUS — command too risky"));
        }
        if risk == RiskLevel::Caution && confirmed != Some(true) {
            dev_log::log_info("ecu", "Command requires confirmation");
            return Err("CONFIRM_REQUIRED".to_string());
        }

        if is_demo() {
            dev_log::log_debug("ecu", &format!("Demo mode: simulating {} → {}", addr, hex_cmd));
            return Ok(format!("[DEMO] OK — {} → {}", addr, hex_cmd));
        }

        dev_log::log_tx(hex_cmd);
        return execute_uds_command(addr, hex_cmd);
    }

    SafetyGuard::validate_hex(&command)?;
    let risk = SafetyGuard::check_command_advanced(&command);
    dev_log::log_debug("ecu", &format!("Safety check for raw hex: {:?}", risk));
    if risk == RiskLevel::Blocked {
        dev_log::log_warn("ecu", "Raw command blocked by safety guard");
        return Err(super::connection::err_msg("BLOQUÉ — commande bloquée par la sécurité", "BLOCKED — command blocked by safety system"));
    }
    if risk == RiskLevel::Dangerous {
        dev_log::log_warn("ecu", "Raw command blocked: dangerous");
        return Err(super::connection::err_msg("DANGEREUX — commande trop risquée", "DANGEROUS — command too risky"));
    }
    if risk == RiskLevel::Caution && confirmed != Some(true) {
        dev_log::log_info("ecu", "Command requires confirmation");
        return Err("CONFIRM_REQUIRED".to_string());
    }

    if is_demo() {
        dev_log::log_debug("ecu", &format!("Demo mode: simulating {} → {}", ecu_address, command));
        return Ok(format!("[DEMO] OK — {} → {}", ecu_address, command));
    }

    dev_log::log_info("ecu", &format!("Sending raw command to ECU {}: {}", ecu_address, command));
    dev_log::log_tx(&command);
    let addr = ecu_address.replace("0x", "");
    execute_uds_command(&addr, &command)
}

#[command]
pub fn check_anomalies(pid_data: Vec<PidValue>) -> Vec<anomaly::Anomaly> {
    let anomalies = anomaly::check_anomalies(&pid_data);
    dev_log::log_info("ecu", &format!("Anomaly check: {} anomalies found", anomalies.len()));
    anomalies
}

#[command]
pub fn get_generic_ecus() -> Vec<ecu_profiles::GenericEcu> {
    dev_log::log_debug("ecu", "get_generic_ecus");
    ecu_profiles::get_generic_ecus()
}

#[command]
pub fn get_manufacturer_dids(manufacturer: String) -> Vec<(String, String)> {
    dev_log::log_debug("ecu", &format!("get_manufacturer_dids: manufacturer='{}'", manufacturer));
    ecu_profiles::get_dids_for_manufacturer(&manufacturer)
}

#[command]
pub fn get_all_manufacturer_dids() -> std::collections::HashMap<String, Vec<(String, String)>> {
    dev_log::log_debug("ecu", "get_all_manufacturer_dids");
    ecu_profiles::get_all_manufacturer_dids()
}

#[command]
pub fn get_advanced_categories() -> Vec<advanced_ops::Category> {
    dev_log::log_debug("ecu", "get_advanced_categories");
    advanced_ops::get_categories()
}

#[command]
pub fn get_advanced_manufacturer_groups() -> std::collections::HashMap<String, advanced_ops::ManufacturerGroup> {
    dev_log::log_debug("ecu", "get_advanced_manufacturer_groups");
    advanced_ops::get_manufacturer_groups()
}

/// Get extended vehicle information (CalID, CVN, ECU Name) via Mode 09 PIDs
#[command]
pub async fn get_vehicle_info_extended() -> Result<crate::models::VehicleInfoExtended, String> {
    tokio::task::spawn_blocking(move || {
        let mut result = crate::models::VehicleInfoExtended { calid: None, cvn: None, ecu_name: None };

        if is_demo() {
            result.calid = Some("DEMO_CALID_001".to_string());
            result.cvn = Some("A1B2C3D4".to_string());
            result.ecu_name = Some("Demo ECU".to_string());
            return Ok(result);
        }

        // Mode 09 PID 04 - Calibration ID
        if let Ok(resp) = with_real_connection(|conn| {
            conn.send_command_timeout("0904", 3000)
        }) {
            let cleaned = resp.lines()
                .filter(|l| !l.starts_with("SEARCHING") && !l.contains("NO DATA"))
                .collect::<Vec<_>>().join(" ");
            let bytes: Vec<u8> = cleaned.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            // Skip response header bytes (49 04 XX)
            let data_bytes: Vec<u8> = bytes.iter().copied()
                .skip_while(|&b| b != 0x49).skip(3)
                .filter(|&b| b >= 0x20 && b <= 0x7E)
                .collect();
            if !data_bytes.is_empty() {
                result.calid = Some(String::from_utf8_lossy(&data_bytes).trim().to_string());
            }
        }

        // Mode 09 PID 06 - CVN
        if let Ok(resp) = with_real_connection(|conn| {
            conn.send_command_timeout("0906", 3000)
        }) {
            let bytes: Vec<u8> = resp.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            let hex: Vec<String> = bytes.iter().skip(3).map(|b| format!("{:02X}", b)).collect();
            if !hex.is_empty() {
                result.cvn = Some(hex.join(""));
            }
        }

        // Mode 09 PID 0A - ECU Name
        if let Ok(resp) = with_real_connection(|conn| {
            conn.send_command_timeout("090A", 3000)
        }) {
            let bytes: Vec<u8> = resp.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            let data_bytes: Vec<u8> = bytes.iter().copied()
                .skip(3)
                .filter(|&b| b >= 0x20 && b <= 0x7E)
                .collect();
            if !data_bytes.is_empty() {
                result.ecu_name = Some(String::from_utf8_lossy(&data_bytes).trim().to_string());
            }
        }

        dev_log::log_info("ecu", &format!("Vehicle info extended: calid={:?}, cvn={:?}, ecu_name={:?}", result.calid, result.cvn, result.ecu_name));
        Ok(result)
    }).await.unwrap_or_else(|_| Ok(crate::models::VehicleInfoExtended { calid: None, cvn: None, ecu_name: None }))
}
