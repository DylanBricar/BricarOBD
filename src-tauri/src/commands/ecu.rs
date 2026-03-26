use tauri::command;
use crate::models::{EcuInfo, MonitorStatus, PidValue, RiskLevel};
use crate::obd::demo::DemoConnection;
use crate::obd::safety::SafetyGuard;
use crate::obd::anomaly;
use crate::obd::ecu_profiles;
use crate::obd::advanced_ops;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};

use std::collections::HashMap;

/// Get user's language from the global setting
fn get_user_lang() -> String {
    super::connection::get_lang()
}

/// Helper to get bilingual ECU name
fn ecu_name(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
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

/// ECU scan address definitions with discovery methods
struct EcuProbe {
    tx_addr: &'static str,
    name_fr: &'static str,
    name_en: &'static str,
    /// DIDs to attempt reading (in order of priority)
    dids: &'static [(&'static str, &'static str)],
}

/// Get all ECU addresses to probe — standard + manufacturer-specific
fn get_ecu_probes() -> Vec<EcuProbe> {
    vec![
        // Standard OBD-II
        EcuProbe { tx_addr: "7E0", name_fr: "Moteur (ECM)", name_en: "Engine (ECM)", dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191"), ("22F194", "F194"), ("22F18C", "F18C")] },
        EcuProbe { tx_addr: "7E1", name_fr: "Transmission (TCM)", name_en: "Transmission (TCM)", dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")] },
        EcuProbe { tx_addr: "7E2", name_fr: "ABS/ESP", name_en: "ABS/ESP", dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")] },
        EcuProbe { tx_addr: "7E3", name_fr: "Airbag (SRS)", name_en: "Airbag (SRS)", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "7E4", name_fr: "Contrôle carrosserie (BCM)", name_en: "Body Control (BCM)", dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")] },
        EcuProbe { tx_addr: "7E5", name_fr: "Tableau de bord", name_en: "Instrument Cluster", dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F18C", "F18C")] },
        EcuProbe { tx_addr: "7E6", name_fr: "Climatisation (HVAC)", name_en: "HVAC", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "7E7", name_fr: "Contrôleur hybride/EV", name_en: "Hybrid/EV Controller", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        // PSA/Stellantis extended
        EcuProbe { tx_addr: "75D", name_fr: "BSI (Boîtier Servitudes Intelligent)", name_en: "BSI (Body Systems Interface)", dids: &[("22F190", "F190"), ("22F18C", "F18C"), ("22F195", "F195"), ("22F191", "F191")] },
        EcuProbe { tx_addr: "6A8", name_fr: "Injection/Moteur (PSA)", name_en: "Injection/Engine (PSA)", dids: &[("22F190", "F190"), ("22F194", "F194"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "6AD", name_fr: "ABS/ESP (PSA)", name_en: "ABS/ESP (PSA)", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "76D", name_fr: "Climatisation (PSA)", name_en: "Climate Control (PSA)", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "772", name_fr: "Capteurs de stationnement", name_en: "Parking Sensors", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "734", name_fr: "Tableau de bord (PSA)", name_en: "Instrument Panel (PSA)", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "7A8", name_fr: "Radio/Audio", name_en: "Radio/Audio", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        EcuProbe { tx_addr: "752", name_fr: "Module de service", name_en: "Service Module", dids: &[("22F190", "F190"), ("22F195", "F195")] },
        // VAG extended
        EcuProbe { tx_addr: "714", name_fr: "Direction", name_en: "Steering", dids: &[("22F190", "F190"), ("22F191", "F191")] },
        EcuProbe { tx_addr: "710", name_fr: "Module confort", name_en: "Comfort Module", dids: &[("22F190", "F190")] },
        EcuProbe { tx_addr: "740", name_fr: "Électronique porte (conducteur)", name_en: "Door Electronics (Driver)", dids: &[("22F190", "F190")] },
        // BMW/Toyota/Honda
        EcuProbe { tx_addr: "7E8", name_fr: "Moteur (réponse)", name_en: "Engine (Response)", dids: &[("22F190", "F190")] },
        EcuProbe { tx_addr: "7DF", name_fr: "Broadcast", name_en: "Broadcast", dids: &[] }, // Broadcast probe
    ]
}

/// Probe whether an ECU is alive using 3-method discovery (header must already be set)
fn probe_ecu_alive(probe: &EcuProbe) -> bool {
    // === Method 1: TesterPresent (3E 00) — fastest, most reliable ===
    if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("3E00", 2000)) {
        if is_valid_ecu_response(&response) {
            dev_log::log_debug("ecu", &format!("ECU at {} responded to TesterPresent", probe.tx_addr));
            return true;
        }
    }

    // === Method 2: StartDiagnosticSession (10 01) — wakes up sleeping ECUs ===
    if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("1001", 3000)) {
        if is_valid_ecu_response(&response) {
            dev_log::log_debug("ecu", &format!("ECU at {} responded to DiagSession", probe.tx_addr));
            return true;
        }
    }

    // === Method 3: ReadDataByIdentifier F190 (VIN) — slower but universal ===
    if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("22F190", 3000)) {
        if is_valid_ecu_response(&response) {
            dev_log::log_debug("ecu", &format!("ECU at {} responded to ReadDID F190", probe.tx_addr));
            return true;
        }
    }

    false
}

/// Read all DIDs for a probe — returns (dids_map, count)
fn read_ecu_dids(probe: &EcuProbe) -> (HashMap<String, String>, usize) {
    let mut dids = HashMap::new();
    let mut dids_read = 0;

    for (did_cmd, did_key) in probe.dids {
        if let Ok(r) = with_real_connection(|conn| conn.send_command_timeout(did_cmd, 3000)) {
            if r.contains("62") {
                // Parse response robustly: find "62" position, skip "62" + 2 DID bytes
                let tokens: Vec<&str> = r.split_whitespace().collect();
                if let Some(pos) = tokens.iter().position(|t| *t == "62") {
                    // Ensure we have at least "62" + DID_high + DID_low
                    if pos + 3 <= tokens.len() {
                        let bytes: Vec<u8> = tokens[pos+3..]
                            .iter()
                            .filter_map(|s| u8::from_str_radix(s, 16).ok())
                            .collect();

                        // Try to decode as ASCII string first
                        if let Ok(val) = String::from_utf8(bytes.clone()) {
                            let clean: String = val.chars().filter(|c| c.is_ascii_graphic() || *c == ' ').collect();
                            if !clean.trim().is_empty() {
                                dids.insert(did_key.to_string(), clean.trim().to_string());
                                dids_read += 1;
                                continue;
                            }
                        }

                        // If not valid ASCII, store as hex
                        if !bytes.is_empty() {
                            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                            dids.insert(did_key.to_string(), hex);
                            dids_read += 1;
                        }
                    } else {
                        dev_log::log_warn("ecu", &format!("DID {} response too short at position {}", did_key, pos));
                    }
                } else {
                    dev_log::log_warn("ecu", &format!("DID {} response contains '62' but not as token", did_key));
                }
            } else if r.contains("7F") {
                // Negative response — DID not supported, skip silently
                continue;
            }
        }
    }

    (dids, dids_read)
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

        let lang = get_user_lang();
        let probes = get_ecu_probes();
        let mut ecus = Vec::new();
        let mut found_addresses = std::collections::HashSet::new();

        for probe in &probes {
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

            // Detect protocol for this ECU (might differ from main)
            let ecu_protocol = with_real_connection(|conn| Ok(conn.protocol.clone()))
                .unwrap_or_else(|_| "Unknown".to_string());

            let probe_name = ecu_name(&lang, probe.name_fr, probe.name_en);
            dev_log::log_info("ecu", &format!("ECU at {} ({}): {} DIDs read", probe.tx_addr, probe_name, dids_read));

            ecus.push(EcuInfo {
                name: probe_name,
                address: format!("0x{}", probe.tx_addr),
                protocol: ecu_protocol,
                dids,
            });
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

/// Check if a response indicates the ECU is alive (not NO DATA, not empty, not error)
fn is_valid_ecu_response(response: &str) -> bool {
    !response.is_empty()
        && !response.contains("NO DATA")
        && !response.contains("ERROR")
        && !response.contains("UNABLE")
        && !response.contains("?")
        && !response.trim().is_empty()
}

/// Read DID from ECU — UDS Service 0x22 with improved error handling
#[command]
pub fn read_did(ecu_address: String, did: String) -> Result<String, String> {
    // Validate DID format: 1-4 hex characters only (prevents injection attacks)
    if did.len() > 4 || did.is_empty() || !did.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid DID format: must be 1-4 hex characters".to_string());
    }

    let cmd = format!("22{}", did.replace(" ", ""));
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
pub fn get_monitors() -> Vec<MonitorStatus> {
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
    ecu_profiles::get_generic_ecus()
}

#[command]
pub fn get_manufacturer_dids(manufacturer: String) -> Vec<(String, String)> {
    ecu_profiles::get_dids_for_manufacturer(&manufacturer)
}

#[command]
pub fn get_all_manufacturer_dids() -> std::collections::HashMap<String, Vec<(String, String)>> {
    ecu_profiles::get_all_manufacturer_dids()
}

#[command]
pub fn get_advanced_categories() -> Vec<advanced_ops::Category> {
    advanced_ops::get_categories()
}

#[command]
pub fn get_advanced_manufacturer_groups() -> std::collections::HashMap<String, advanced_ops::ManufacturerGroup> {
    advanced_ops::get_manufacturer_groups()
}
