use crate::models::EcuInfo;
use crate::obd::dev_log;
use crate::commands::connection::{with_real_connection, get_lang};
use std::collections::HashMap;

/// ECU scan address definitions with discovery methods
pub struct EcuProbe {
    pub tx_addr: &'static str,
    pub name_fr: &'static str,
    pub name_en: &'static str,
    /// DIDs to attempt reading (in order of priority)
    pub dids: &'static [(&'static str, &'static str)],
}

/// Get all ECU addresses to probe — standard + manufacturer-specific
pub fn get_ecu_probes() -> Vec<EcuProbe> {
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

/// Helper to get bilingual ECU name
pub fn ecu_name(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
}

/// Check if a response indicates the ECU is alive (not NO DATA, not empty, not error)
pub fn is_valid_ecu_response(response: &str) -> bool {
    !response.is_empty()
        && !response.contains("NO DATA")
        && !response.contains("ERROR")
        && !response.contains("UNABLE")
        && !response.contains("?")
        && !response.trim().is_empty()
}

/// Probe whether an ECU is alive using 3-method discovery (header must already be set)
pub fn probe_ecu_alive(probe: &EcuProbe) -> bool {
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
pub fn read_ecu_dids(probe: &EcuProbe) -> (HashMap<String, String>, usize) {
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

/// Build and return EcuInfo from a probe after successful discovery
pub fn build_ecu_info(probe: &EcuProbe, dids: &HashMap<String, String>) -> EcuInfo {
    let lang = get_lang();
    let ecu_protocol = with_real_connection(|conn| Ok(conn.protocol.clone()))
        .unwrap_or_else(|_| "Unknown".to_string());

    EcuInfo {
        name: ecu_name(&lang, probe.name_fr, probe.name_en),
        address: format!("0x{}", probe.tx_addr),
        protocol: ecu_protocol,
        dids: dids.clone(),
    }
}
