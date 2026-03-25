use tauri::command;
use crate::models::{EcuInfo, MonitorStatus, PidValue, RiskLevel};
use crate::obd::demo::DemoConnection;
use crate::obd::safety::SafetyGuard;
use crate::obd::anomaly;
use crate::obd::ecu_profiles;
use crate::obd::advanced_ops;
use crate::commands::connection::{is_demo, with_real_connection};

use std::collections::HashMap;

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

/// Scan all ECUs — probes standard OBD-II + manufacturer addresses
#[command]
pub fn scan_ecus() -> Vec<EcuInfo> {
    if is_demo() {
        crate::obd::dev_log::log_info("ecu", "Demo mode: returning simulated ECUs");
        return DemoConnection::get_ecus();
    }

    crate::obd::dev_log::log_info("ecu", "Real mode: starting ECU scan");

    // Standard OBD-II + PSA/VAG/BMW extended addresses
    let addresses = [
        // Standard OBD-II
        ("7E0", "Engine (ECM)"),
        ("7E1", "Transmission (TCM)"),
        ("7E2", "ABS/ESP"),
        ("7E3", "Airbag (SRS)"),
        ("7E4", "Body Control (BCM)"),
        ("7E5", "Instrument Cluster"),
        ("7E6", "HVAC"),
        // PSA/Stellantis extended
        ("75D", "BSI (Body Systems Interface)"),
        ("6A8", "Injection/Engine (PSA)"),
        ("6AD", "ABS/ESP (PSA)"),
        ("76D", "Climate Control (PSA)"),
        ("772", "Parking Sensors"),
        ("734", "Instrument Panel (PSA)"),
        ("7A8", "Radio/Audio"),
        // VAG extended
        ("7E4", "Gateway"),
        ("714", "Steering"),
    ];

    let mut ecus = Vec::new();

    for (tx, name) in &addresses {
        crate::obd::dev_log::log_debug("ecu", &format!("Probing address: {}", tx));
        if with_real_connection(|conn| conn.send_command(&format!("ATSH{}", tx))).is_err() {
            continue;
        }

        if let Ok(response) = with_real_connection(|conn| conn.send_command("3E00")) {
            if !response.contains("NO DATA") && !response.contains("ERROR") && !response.is_empty() {
                let mut dids = HashMap::new();

                // Read common DIDs
                let mut dids_read = 0;
                for (did_cmd, did_key) in [("22F190", "F190"), ("22F195", "F195"), ("22F191", "F191")] {
                    if let Ok(r) = with_real_connection(|conn| conn.send_command(did_cmd)) {
                        if r.contains("62") {
                            let bytes: Vec<u8> = r.split_whitespace()
                                .skip(3)
                                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                                .collect();
                            if let Ok(val) = String::from_utf8(bytes) {
                                let clean: String = val.chars().filter(|c| c.is_ascii_graphic() || *c == ' ').collect();
                                if !clean.is_empty() {
                                    dids.insert(did_key.to_string(), clean.trim().to_string());
                                    dids_read += 1;
                                }
                            }
                        }
                    }
                }

                crate::obd::dev_log::log_info("ecu", &format!("ECU at {} ({}): {} DIDs read", tx, name, dids_read));

                ecus.push(EcuInfo {
                    name: name.to_string(),
                    address: format!("0x{}", tx),
                    protocol: "ISO 15765-4 CAN".to_string(),
                    dids,
                });
            }
        }
    }

    let _ = with_real_connection(|conn| { conn.send_command("ATH0")?; conn.send_command("ATSH7DF") });

    if ecus.is_empty() {
        crate::obd::dev_log::log_warn("ecu", "No ECUs found during real scan");
        tracing::warn!("No ECUs found during real scan");
        Vec::new() // Return empty — no fake data in real mode
    } else {
        crate::obd::dev_log::log_info("ecu", &format!("ECU scan complete: found {} ECUs", ecus.len()));
        tracing::info!("Found {} ECUs", ecus.len());
        ecus
    }
}

/// Read DID from ECU — UDS Service 0x22
#[command]
pub fn read_did(ecu_address: String, did: String) -> Result<String, String> {
    let cmd = format!("22{}", did.replace(" ", ""));
    let risk = SafetyGuard::check_command(&format!("22 {}", did));
    crate::obd::dev_log::log_info("ecu", &format!("Read DID safety check: {:?}", risk));
    if risk == RiskLevel::Blocked {
        crate::obd::dev_log::log_warn("ecu", "Read DID blocked by safety guard");
        return Err("BLOCKED".to_string());
    }

    if is_demo() {
        crate::obd::dev_log::log_debug("ecu", &format!("Demo mode: reading DID {} from {}", did, ecu_address));
        return Ok(format!("[DEMO] 62 {} 56 46 33 4C 43 42", did));
    }

    crate::obd::dev_log::log_info("ecu", &format!("Reading DID {} from ECU {}", did, ecu_address));
    let addr = ecu_address.replace("0x", "");
    let _ = with_real_connection(|conn| conn.send_command(&format!("ATSH{}", addr)));
    let result = with_real_connection(|conn| conn.send_command(&cmd))?;
    crate::obd::dev_log::log_rx(&cmd, &result);
    let _ = with_real_connection(|conn| conn.send_command("ATSH7DF"));
    Ok(result)
}

/// Get OBD monitor statuses — Mode 01 PID 01
#[command]
pub fn get_monitors() -> Vec<MonitorStatus> {
    if is_demo() {
        crate::obd::dev_log::log_debug("ecu", "Demo mode: returning simulated monitor statuses");
        return DemoConnection::get_monitors();
    }

    crate::obd::dev_log::log_info("ecu", "Real mode: reading Mode 01 PID 01 for monitor statuses");
    let response = match with_real_connection(|conn| conn.query_pid(0x01, 0x01)) {
        Ok(bytes) if bytes.len() >= 4 => {
            crate::obd::dev_log::log_rx("0101", &format!("{:02X?}", bytes));
            bytes
        },
        _ => {
            crate::obd::dev_log::log_warn("ecu", "Mode 01 PID 01 read failed, returning empty");
            return Vec::new(); // No fake data — return empty if real read fails
        }
    };

    let b = response[1];
    let c = response[2];
    let d = response[3];

    let mut monitors = Vec::new();

    // Continuous
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

    // Non-continuous
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

/// Send raw UDS command or named operation (Advanced mode — uses elevated safety)
#[command]
pub fn send_raw_command(ecu_address: String, command: String) -> Result<String, String> {
    if let Some((addr, hex_cmd)) = resolve_operation_command(&command) {
        // Advanced mode: allows 2E, 2F, 30, 31 as Caution
        crate::obd::dev_log::log_info("ecu", &format!("Operation resolved: {} → {}", command, hex_cmd));
        let risk = SafetyGuard::check_command_advanced(hex_cmd);
        crate::obd::dev_log::log_debug("ecu", &format!("Safety check result: {:?}", risk));
        if risk == RiskLevel::Blocked {
            crate::obd::dev_log::log_warn("ecu", "Operation blocked by safety guard");
            return Err("BLOCKED".to_string());
        }
        if risk == RiskLevel::Dangerous {
            crate::obd::dev_log::log_warn("ecu", "Operation blocked: dangerous command");
            return Err("DANGEROUS".to_string());
        }

        if is_demo() {
            crate::obd::dev_log::log_debug("ecu", &format!("Demo mode: simulating {} → {}", addr, hex_cmd));
            return Ok(format!("[DEMO] OK — {} → {}", addr, hex_cmd));
        }

        crate::obd::dev_log::log_tx(hex_cmd);
        let _ = with_real_connection(|conn| conn.send_command(&format!("ATSH{}", addr)));
        let result = with_real_connection(|conn| conn.send_command(hex_cmd))?;
        crate::obd::dev_log::log_rx(hex_cmd, &result);
        let _ = with_real_connection(|conn| conn.send_command("ATSH7DF"));
        return Ok(result);
    }

    SafetyGuard::validate_hex(&command)?;
    let risk = SafetyGuard::check_command_advanced(&command);
    crate::obd::dev_log::log_debug("ecu", &format!("Safety check for raw hex: {:?}", risk));
    if risk == RiskLevel::Blocked {
        crate::obd::dev_log::log_warn("ecu", "Raw command blocked by safety guard");
        return Err("BLOCKED".to_string());
    }
    if risk == RiskLevel::Dangerous {
        crate::obd::dev_log::log_warn("ecu", "Raw command blocked: dangerous");
        return Err("DANGEROUS".to_string());
    }

    if is_demo() {
        crate::obd::dev_log::log_debug("ecu", &format!("Demo mode: simulating {} → {}", ecu_address, command));
        return Ok(format!("[DEMO] OK — {} → {}", ecu_address, command));
    }

    crate::obd::dev_log::log_info("ecu", &format!("Sending raw command to ECU {}: {}", ecu_address, command));
    crate::obd::dev_log::log_tx(&command);
    let addr = ecu_address.replace("0x", "");
    let _ = with_real_connection(|conn| conn.send_command(&format!("ATSH{}", addr)));
    let result = with_real_connection(|conn| conn.send_command(&command))?;
    crate::obd::dev_log::log_rx(&command, &result);
    let _ = with_real_connection(|conn| conn.send_command("ATSH7DF"));
    Ok(result)
}

#[command]
pub fn check_anomalies(pid_data: Vec<PidValue>) -> Vec<anomaly::Anomaly> {
    let anomalies = anomaly::check_anomalies(&pid_data);
    crate::obd::dev_log::log_info("ecu", &format!("Anomaly check complete: {} anomalies found", anomalies.len()));
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
