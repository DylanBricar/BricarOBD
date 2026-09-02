use crate::commands::connection::{get_lang, with_real_connection};
use crate::models::EcuInfo;
use crate::obd::dev_log;
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
        EcuProbe {
            tx_addr: "7E0",
            name_fr: "Moteur (ECM)",
            name_en: "Engine (ECM)",
            dids: &[
                ("22F190", "F190"),
                ("22F195", "F195"),
                ("22F191", "F191"),
                ("22F194", "F194"),
                ("22F18C", "F18C"),
            ],
        },
        EcuProbe {
            tx_addr: "7E1",
            name_fr: "Transmission (TCM)",
            name_en: "Transmission (TCM)",
            dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")],
        },
        EcuProbe {
            tx_addr: "7E2",
            name_fr: "ABS/ESP",
            name_en: "ABS/ESP",
            dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")],
        },
        EcuProbe {
            tx_addr: "7E3",
            name_fr: "Airbag (SRS)",
            name_en: "Airbag (SRS)",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "7E4",
            name_fr: "Contrôle carrosserie (BCM)",
            name_en: "Body Control (BCM)",
            dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")],
        },
        EcuProbe {
            tx_addr: "7E5",
            name_fr: "Tableau de bord",
            name_en: "Instrument Cluster",
            dids: &[("22F190", "F190"), ("22F195", "F195"), ("22F18C", "F18C")],
        },
        EcuProbe {
            tx_addr: "7E6",
            name_fr: "Climatisation (HVAC)",
            name_en: "HVAC",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "7E7",
            name_fr: "Contrôleur hybride/EV",
            name_en: "Hybrid/EV Controller",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        // PSA/Stellantis extended
        EcuProbe {
            tx_addr: "75D",
            name_fr: "BSI (Boîtier Servitudes Intelligent)",
            name_en: "BSI (Body Systems Interface)",
            dids: &[
                ("22F190", "F190"),
                ("22F18C", "F18C"),
                ("22F195", "F195"),
                ("22F191", "F191"),
            ],
        },
        EcuProbe {
            tx_addr: "6A8",
            name_fr: "Injection/Moteur (PSA)",
            name_en: "Injection/Engine (PSA)",
            dids: &[("22F190", "F190"), ("22F194", "F194"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "6AD",
            name_fr: "ABS/ESP (PSA)",
            name_en: "ABS/ESP (PSA)",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "76D",
            name_fr: "Climatisation (PSA)",
            name_en: "Climate Control (PSA)",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "772",
            name_fr: "Capteurs de stationnement",
            name_en: "Parking Sensors",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "734",
            name_fr: "Tableau de bord (PSA)",
            name_en: "Instrument Panel (PSA)",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "7A8",
            name_fr: "Radio/Audio",
            name_en: "Radio/Audio",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        EcuProbe {
            tx_addr: "752",
            name_fr: "Module de service",
            name_en: "Service Module",
            dids: &[("22F190", "F190"), ("22F195", "F195")],
        },
        // VAG extended
        EcuProbe {
            tx_addr: "714",
            name_fr: "Direction",
            name_en: "Steering",
            dids: &[("22F190", "F190"), ("22F191", "F191")],
        },
        EcuProbe {
            tx_addr: "710",
            name_fr: "Module confort",
            name_en: "Comfort Module",
            dids: &[("22F190", "F190")],
        },
        EcuProbe {
            tx_addr: "740",
            name_fr: "Électronique porte (conducteur)",
            name_en: "Door Electronics (Driver)",
            dids: &[("22F190", "F190")],
        },
        EcuProbe {
            tx_addr: "7DF",
            name_fr: "Broadcast",
            name_en: "Broadcast",
            dids: &[],
        }, // Broadcast probe
    ]
}

/// Put manufacturer-specific ECUs near the front so the bounded scan does not
/// expire on generic addresses before reaching the relevant controllers.
pub fn ecu_probe_priority(manufacturer: &str, tx_addr: &str) -> u8 {
    if tx_addr == "7E0" {
        return 0;
    }
    let manufacturer = manufacturer.to_uppercase();
    let psa = matches!(
        manufacturer.as_str(),
        "PEUGEOT" | "CITROËN" | "CITROEN" | "DS" | "DS AUTOMOBILES" | "OPEL" | "VAUXHALL"
    );
    if psa {
        return match tx_addr {
            "6A8" => 1,
            "75D" => 2,
            "6AD" | "76D" | "734" | "772" | "7A8" | "752" => 3,
            _ => 10,
        };
    }
    10
}

/// Helper to get bilingual ECU name
pub fn ecu_name(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" {
        fr.to_string()
    } else {
        en.to_string()
    }
}

/// Check if a response indicates the ECU is alive (not NO DATA, not empty, not error)
pub fn is_valid_ecu_response(response: &str) -> bool {
    if response.is_empty()
        || response.contains("NO DATA")
        || response.contains("ERROR")
        || response.contains("UNABLE")
        || response.contains('?')
    {
        return false;
    }

    response.lines().any(|line| {
        let (_, bytes) = crate::obd::Elm327Connection::parse_hex_response_line(line);
        let positive = bytes
            .iter()
            .position(|service| matches!(service, 0x7E | 0x62));
        let negative = bytes
            .windows(3)
            .position(|window| window[0] == 0x7F && matches!(window[1], 0x3E | 0x22));
        positive.is_some_and(|position| negative.is_none_or(|negative| position < negative))
    })
}

/// Probe whether an ECU is alive using 3-method discovery (header must already be set)
pub fn probe_ecu_alive(probe: &EcuProbe) -> bool {
    // === Method 1: TesterPresent (3E 00) — fastest, most reliable ===
    if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("3E00", 2000)) {
        if is_valid_ecu_response(&response) {
            dev_log::log_debug(
                "ecu",
                &format!("ECU at {} responded to TesterPresent", probe.tx_addr),
            );
            return true;
        }
    }

    // === Method 2: ReadDataByIdentifier F190 (VIN) — slower but read-only ===
    if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("22F190", 3000)) {
        if is_valid_ecu_response(&response) {
            dev_log::log_debug(
                "ecu",
                &format!("ECU at {} responded to ReadDID F190", probe.tx_addr),
            );
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
            let Some(did_id) = u16::from_str_radix(did_key, 16).ok() else {
                continue;
            };
            let Some(bytes) = crate::obd::Elm327Connection::parse_did_response(&r, did_id) else {
                continue;
            };

            // Try to decode identity DIDs as ASCII string first.
            if let Ok(val) = String::from_utf8(bytes.clone()) {
                let clean: String = val
                    .chars()
                    .filter(|c| c.is_ascii_graphic() || *c == ' ')
                    .collect();
                if !clean.trim().is_empty() {
                    dids.insert(did_key.to_string(), clean.trim().to_string());
                    dids_read += 1;
                    continue;
                }
            }

            let hex = bytes
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<Vec<_>>()
                .join(" ");
            dids.insert(did_key.to_string(), hex);
            dids_read += 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ecu_probes_count() {
        let probes = get_ecu_probes();
        assert!(probes.len() >= 20, "Should have at least 20 ECU probes");
    }

    #[test]
    fn test_get_ecu_probes_has_engine() {
        let probes = get_ecu_probes();
        assert!(
            probes.iter().any(|p| p.tx_addr == "7E0"),
            "Should have engine ECU 7E0"
        );
    }

    #[test]
    fn test_get_ecu_probes_has_transmission() {
        let probes = get_ecu_probes();
        assert!(
            probes.iter().any(|p| p.tx_addr == "7E1"),
            "Should have transmission ECU 7E1"
        );
    }

    #[test]
    fn test_get_ecu_probes_has_abs() {
        let probes = get_ecu_probes();
        assert!(
            probes.iter().any(|p| p.tx_addr == "7E2"),
            "Should have ABS ECU 7E2"
        );
    }

    #[test]
    fn test_get_ecu_probes_has_psa_bsi() {
        let probes = get_ecu_probes();
        assert!(
            probes.iter().any(|p| p.tx_addr == "75D"),
            "Should have PSA BSI 75D"
        );
    }

    #[test]
    fn test_get_ecu_probes_engine_has_dids() {
        let probes = get_ecu_probes();
        let engine = probes.iter().find(|p| p.tx_addr == "7E0").unwrap();
        assert!(
            engine.dids.len() >= 4,
            "Engine ECU should have at least 4 DIDs"
        );
    }

    #[test]
    fn test_ecu_name_fr() {
        assert_eq!(ecu_name("fr", "Moteur", "Engine"), "Moteur");
    }

    #[test]
    fn test_ecu_name_en() {
        assert_eq!(ecu_name("en", "Moteur", "Engine"), "Engine");
    }

    #[test]
    fn test_ecu_name_default_en() {
        assert_eq!(ecu_name("de", "Moteur", "Engine"), "Engine");
    }

    #[test]
    fn test_is_valid_ecu_response_valid() {
        assert!(is_valid_ecu_response("62 F1 90 41 42 43"));
    }

    #[test]
    fn test_is_valid_ecu_response_no_data() {
        assert!(!is_valid_ecu_response("NO DATA"));
    }

    #[test]
    fn test_is_valid_ecu_response_error() {
        assert!(!is_valid_ecu_response("ERROR"));
    }

    #[test]
    fn test_is_valid_ecu_response_unable() {
        assert!(!is_valid_ecu_response("UNABLE TO CONNECT"));
    }

    #[test]
    fn test_is_valid_ecu_response_question_mark() {
        assert!(!is_valid_ecu_response("?"));
    }

    #[test]
    fn test_is_valid_ecu_response_empty() {
        assert!(!is_valid_ecu_response(""));
    }

    #[test]
    fn test_is_valid_ecu_response_whitespace_only() {
        assert!(!is_valid_ecu_response("   "));
    }

    #[test]
    fn test_is_valid_ecu_response_contains_no_data() {
        assert!(!is_valid_ecu_response("7E0 NO DATA"));
    }

    #[test]
    fn test_get_ecu_probes_all_have_names() {
        let probes = get_ecu_probes();
        for probe in &probes {
            assert!(
                !probe.name_fr.is_empty(),
                "Probe {} should have FR name",
                probe.tx_addr
            );
            assert!(
                !probe.name_en.is_empty(),
                "Probe {} should have EN name",
                probe.tx_addr
            );
        }
    }

    #[test]
    fn test_get_ecu_probes_unique_addresses() {
        let probes = get_ecu_probes();
        let mut addrs: Vec<&str> = probes.iter().map(|p| p.tx_addr).collect();
        let count = addrs.len();
        addrs.sort();
        addrs.dedup();
        assert_eq!(addrs.len(), count, "All ECU addresses should be unique");
    }

    #[test]
    fn psa_injection_ecu_is_prioritized_before_generic_secondary_ecus() {
        assert!(ecu_probe_priority("Peugeot", "6A8") < ecu_probe_priority("Peugeot", "7E1"));
        assert_eq!(ecu_probe_priority("Peugeot", "7E0"), 0);
    }
}
