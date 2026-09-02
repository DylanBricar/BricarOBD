use crate::commands::connection::{is_demo, with_real_connection};
use crate::commands::ecu_scan::{
    build_ecu_info, ecu_probe_priority, get_ecu_probes, probe_ecu_alive, read_ecu_dids,
};
use crate::commands::OBDBusyGuard;
use crate::models::{EcuInfo, MonitorStatus, PidValue, RiskLevel};
use crate::obd::advanced_ops;
use crate::obd::anomaly;
use crate::obd::demo::DemoConnection;
use crate::obd::dev_log;
use crate::obd::ecu_profiles;
use crate::obd::nrc;
use crate::obd::safety::SafetyGuard;
use tauri::command;

/// Get user's language from the global setting
fn get_user_lang() -> String {
    super::connection::get_lang()
}

/// Scan all ECUs — probes standard OBD-II + manufacturer addresses with multi-method discovery
#[command]
pub async fn scan_ecus(manufacturer: Option<String>) -> Vec<EcuInfo> {
    tokio::task::spawn_blocking(move || {
        let _guard = match OBDBusyGuard::acquire_with_wait(15) {
            Ok(g) => g,
            Err(e) => {
                dev_log::log_warn("ecu", &format!("ECU scan blocked after wait: {}", e));
                return Vec::new();
            }
        };

        if is_demo() {
            dev_log::log_info("ecu", "Demo mode: returning simulated ECUs");
            let lang = get_user_lang();
            return DemoConnection::get_ecus(&lang);
        }

        dev_log::log_info(
            "ecu",
            "Real mode: starting ECU scan with multi-method discovery",
        );

        let mut probes = get_ecu_probes();
        let manufacturer = manufacturer.unwrap_or_default();
        probes.sort_by_key(|probe| ecu_probe_priority(&manufacturer, probe.tx_addr));
        let mut ecus = Vec::new();
        let mut found_addresses = std::collections::HashSet::new();
        let scan_start = std::time::Instant::now();
        let max_scan_duration = std::time::Duration::from_secs(60);

        for probe in &probes {
            if super::connection::is_obd_cancelled() {
                break;
            }
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
            dev_log::log_info(
                "ecu",
                &format!("ECU at {}: {} DIDs read", probe.tx_addr, dids_read),
            );

            ecus.push(build_ecu_info(probe, &dids));
        }

        // Reset headers to broadcast
        let _ = with_real_connection(|conn| conn.reset_headers());

        if ecus.is_empty() {
            dev_log::log_warn(
                "ecu",
                "No ECUs found during scan — vehicle may need ignition on",
            );
            tracing::warn!("No ECUs found during real scan");
        } else {
            dev_log::log_info(
                "ecu",
                &format!("ECU scan complete: {} ECUs found", ecus.len()),
            );
            tracing::info!("Found {} ECUs", ecus.len());
        }

        ecus
    })
    .await
    .unwrap_or_else(|e| {
        dev_log::log_error("ecu", &format!("scan_ecus task failed: {}", e));
        Vec::new()
    })
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
        return Err(super::connection::err_msg(
            "BLOQUÉ — commande bloquée par la sécurité",
            "BLOCKED — command blocked by safety system",
        ));
    }

    if is_demo() {
        dev_log::log_debug(
            "ecu",
            &format!("Demo mode: reading DID {} from {}", did, ecu_address),
        );
        return Ok(format!("[DEMO] 62 {} 56 46 33 4C 43 42", did));
    }

    tokio::task::spawn_blocking(move || {
        let _guard = OBDBusyGuard::acquire_with_wait(10)?;
        let cmd = format!("22{}", did.replace(" ", ""));
        dev_log::log_info(
            "ecu",
            &format!("Reading DID {} from ECU {}", did, ecu_address),
        );
        let addr = ecu_address.replace("0x", "");

        let result = with_real_connection(|conn| {
            conn.set_ecu_header(&addr)?;
            conn.tester_present();
            let result = conn.send_command_timeout(&cmd, 5000);
            let reset_result = conn.reset_headers();
            match (result, reset_result) {
                (Ok(response), Ok(())) => Ok(response),
                (Err(error), _) => Err(error),
                (Ok(_), Err(error)) => Err(error),
            }
        });

        match result {
            Ok(r) => {
                if r.contains("NO DATA") {
                    Err(format!("DID {} not supported by ECU {}", did, ecu_address))
                } else if r.contains("7F") {
                    // Parse negative response code
                    let lang = get_user_lang();
                    let nrc_msg = nrc::parse_negative_response(&r, &lang);
                    Err(format!("DID {} error: {}", did, nrc_msg))
                } else {
                    Ok(r)
                }
            }
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

fn decode_monitor_statuses(response: &[u8]) -> Vec<MonitorStatus> {
    if response.len() < 4 {
        return Vec::new();
    }
    let b = response[1];
    let c = response[2];
    let d = response[3];
    let mut monitors = Vec::new();

    fn add_monitor(
        monitors: &mut Vec<MonitorStatus>,
        bit: u8,
        key: &str,
        available_bits: u8,
        incomplete_bits: u8,
    ) {
        let available = available_bits & bit != 0;
        monitors.push(MonitorStatus {
            name_key: key.to_string(),
            available,
            complete: available && incomplete_bits & bit == 0,
            description_key: Some(format!("{key}Desc")),
            specification_key: Some(format!("{key}Spec")),
        });
    }

    for (support_bit, incomplete_bit, key) in [
        (0x01, 0x10, "monitors.misfire"),
        (0x02, 0x20, "monitors.fuelSystem"),
        (0x04, 0x40, "monitors.components"),
    ] {
        let available = b & support_bit != 0;
        monitors.push(MonitorStatus {
            name_key: key.to_string(),
            available,
            complete: available && b & incomplete_bit == 0,
            description_key: Some(format!("{key}Desc")),
            specification_key: Some(format!("{key}Spec")),
        });
    }

    if b & 0x08 == 0 {
        for (bit, key) in [
            (0x01, "monitors.catalyst"),
            (0x02, "monitors.heatedCatalyst"),
            (0x04, "monitors.evap"),
            (0x08, "monitors.secondaryAir"),
            (0x10, "monitors.ac"),
            (0x20, "monitors.oxygenSensor"),
            (0x40, "monitors.oxygenSensorHeater"),
            (0x80, "monitors.egrVvt"),
        ] {
            add_monitor(&mut monitors, bit, key, c, d);
        }
    } else {
        for (bit, key) in [
            (0x01, "monitors.nmhcCatalyst"),
            (0x02, "monitors.noxScr"),
            (0x08, "monitors.boostPressure"),
            (0x20, "monitors.exhaustGasSensor"),
            (0x40, "monitors.pmFilter"),
            (0x80, "monitors.egrVvt"),
        ] {
            add_monitor(&mut monitors, bit, key, c, d);
        }
    }

    monitors
}

/// Get OBD monitor statuses — Mode 01 PID 01, with retry and wake-up
#[command]
pub async fn get_monitors() -> Vec<MonitorStatus> {
    tokio::task::spawn_blocking(|| {
        if is_demo() {
            dev_log::log_debug("ecu", "Demo mode: returning simulated monitor statuses");
            return DemoConnection::get_monitors();
        }

        // Wait for any ongoing OBD operation (e.g. ECU scan) to finish before querying
        let _guard = match OBDBusyGuard::acquire_with_wait(10) {
            Ok(g) => g,
            Err(error) => {
                dev_log::log_warn("ecu", &format!("get_monitors postponed: {error}"));
                return Vec::new();
            }
        };

        dev_log::log_info(
            "ecu",
            "Real mode: reading Mode 01 PID 01 for monitor statuses",
        );

        // Ensure headers are reset to broadcast before querying monitors
        let _ = with_real_connection(|conn| conn.reset_headers());

        // Try up to 2 times — first attempt may fail if ECU is asleep
        let response = match with_real_connection(|conn| conn.query_pid(0x01, 0x01)) {
            Ok(bytes) if bytes.len() >= 4 => {
                dev_log::log_rx("0101", &format!("{:02X?}", bytes));
                bytes
            }
            _ => {
                // Retry after wake-up
                dev_log::log_warn("ecu", "PID 01 failed, trying wake-up + retry...");
                let _ = with_real_connection(|conn| {
                    conn.tester_present();
                    Ok(())
                });
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

        decode_monitor_statuses(&response)
    })
    .await
    .unwrap_or_default()
}

/// Execute a UDS command against an ECU: set header → tester_present → send → log_rx → reset headers
/// All operations run under a single CONNECTION lock to prevent interleaved commands.
fn execute_uds_command(addr: &str, hex_cmd: &str) -> Result<String, String> {
    with_real_connection(|conn| {
        conn.set_ecu_header(addr)?;
        conn.tester_present();
        let result = conn.send_command_timeout(hex_cmd, 8000);
        dev_log::log_rx(hex_cmd, result.as_deref().unwrap_or("(error)"));
        let reset_result = conn.reset_headers();
        match (result, reset_result) {
            (Ok(response), Ok(())) => Ok(response),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    })
}

/// Send raw UDS command or named operation (Advanced mode — uses elevated safety)
#[command]
pub async fn send_raw_command(
    ecu_address: String,
    command: String,
    _confirmed: Option<bool>,
) -> Result<String, String> {
    if advanced_ops::is_named_operation(&command) {
        return Err(super::connection::err_msg(
            "Opération désactivée : aucun profil véhicule/ECU vérifié n'est disponible",
            "Operation disabled: no verified vehicle/ECU profile is available",
        ));
    }

    let command = SafetyGuard::normalize_hex(&command)?;
    let risk = SafetyGuard::check_command(&command);
    dev_log::log_debug("ecu", &format!("Safety check for raw hex: {:?}", risk));
    if risk != RiskLevel::Safe {
        dev_log::log_warn(
            "ecu",
            "Raw command blocked: only read-only services are allowed",
        );
        return Err(super::connection::err_msg(
            "Commande refusée : seules les lectures sont autorisées",
            "Command refused: only read-only services are allowed",
        ));
    }

    if is_demo() {
        dev_log::log_debug(
            "ecu",
            &format!("Demo mode: simulating {} → {}", ecu_address, command),
        );
        return Ok(format!("[DEMO] OK — {} → {}", ecu_address, command));
    }

    dev_log::log_info(
        "ecu",
        &format!("Sending raw command to ECU {}: {}", ecu_address, command),
    );
    dev_log::log_tx(&command);
    let addr = ecu_address.replace("0x", "");
    let cmd = command.clone();
    tokio::task::spawn_blocking(move || {
        let _guard = super::connection::OBDBusyGuard::try_acquire()?;
        execute_uds_command(&addr, &cmd)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[command]
pub fn check_anomalies(pid_data: Vec<PidValue>) -> Vec<anomaly::Anomaly> {
    let anomalies = anomaly::check_anomalies(&pid_data);
    dev_log::log_info(
        "ecu",
        &format!("Anomaly check: {} anomalies found", anomalies.len()),
    );
    anomalies
}

#[command]
pub fn get_generic_ecus() -> Vec<ecu_profiles::GenericEcu> {
    dev_log::log_debug("ecu", "get_generic_ecus");
    ecu_profiles::get_generic_ecus().to_vec()
}

#[command]
pub fn get_manufacturer_dids(manufacturer: String) -> Vec<(String, String)> {
    dev_log::log_debug(
        "ecu",
        &format!("get_manufacturer_dids: manufacturer='{}'", manufacturer),
    );
    ecu_profiles::get_dids_for_manufacturer(&manufacturer)
}

#[command]
pub fn get_all_manufacturer_dids() -> std::collections::HashMap<String, Vec<(String, String)>> {
    dev_log::log_debug("ecu", "get_all_manufacturer_dids");
    ecu_profiles::get_all_manufacturer_dids()
}

#[cfg(test)]
mod readiness_tests {
    use super::*;

    #[test]
    fn spark_readiness_uses_sae_bit_names() {
        let monitors = decode_monitor_statuses(&[0, 0x00, 0x62, 0x00]);
        let keys: Vec<&str> = monitors
            .iter()
            .filter(|monitor| monitor.available)
            .map(|monitor| monitor.name_key.as_str())
            .collect();
        assert_eq!(
            keys,
            vec![
                "monitors.heatedCatalyst",
                "monitors.oxygenSensor",
                "monitors.oxygenSensorHeater"
            ]
        );
    }

    #[test]
    fn compression_readiness_uses_diesel_monitor_set() {
        let monitors = decode_monitor_statuses(&[0, 0x08, 0xCA, 0x00]);
        let keys: Vec<&str> = monitors
            .iter()
            .filter(|monitor| monitor.available)
            .map(|monitor| monitor.name_key.as_str())
            .collect();
        assert_eq!(
            keys,
            vec![
                "monitors.noxScr",
                "monitors.boostPressure",
                "monitors.pmFilter",
                "monitors.egrVvt"
            ]
        );
    }
}

#[command]
pub fn get_advanced_categories() -> Vec<advanced_ops::Category> {
    dev_log::log_debug("ecu", "get_advanced_categories");
    advanced_ops::get_categories()
}

#[command]
pub fn get_advanced_manufacturer_groups(
) -> std::collections::HashMap<String, advanced_ops::ManufacturerGroup> {
    dev_log::log_debug("ecu", "get_advanced_manufacturer_groups");
    advanced_ops::get_manufacturer_groups()
}
