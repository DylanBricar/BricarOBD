use tauri::command;
use crate::models::{DtcCode, DtcStatus};
use crate::obd::demo::DemoConnection;
use crate::obd::dtc;
use crate::commands::connection::{is_demo, with_real_connection};

/// Read all DTCs — Mode 03 (active), Mode 07 (pending), Mode 0A (permanent)
#[command]
pub fn read_all_dtcs(lang: Option<String>) -> Vec<DtcCode> {
    let _lang = lang.unwrap_or_else(|| "fr".to_string());
    if is_demo() {
        crate::obd::dev_log::log_info("dtc", "Demo mode: returning simulated DTCs");
        return DemoConnection::get_dtcs();
    }

    crate::obd::dev_log::log_info("dtc", "Real mode: starting DTC scan");

    // Real vehicle: scan all 3 OBD modes
    let mut all_dtcs: Vec<DtcCode> = Vec::new();

    // Mode 03 — Confirmed/Active DTCs
    if let Ok(response) = with_real_connection(|conn| conn.send_command("03")) {
        let parsed = dtc::parse_dtc_response(&response, DtcStatus::Active, "OBD Mode 03");
        crate::obd::dev_log::log_info("dtc", &format!("Mode 03 (Active): found {} DTCs", parsed.len()));
        all_dtcs.extend(parsed);
    }

    // Mode 07 — Pending DTCs
    if let Ok(response) = with_real_connection(|conn| conn.send_command("07")) {
        let parsed = dtc::parse_dtc_response(&response, DtcStatus::Pending, "OBD Mode 07");
        crate::obd::dev_log::log_info("dtc", &format!("Mode 07 (Pending): found {} DTCs", parsed.len()));
        all_dtcs.extend(parsed);
    }

    // Mode 0A — Permanent DTCs
    if let Ok(response) = with_real_connection(|conn| conn.send_command("0A")) {
        let parsed = dtc::parse_dtc_response(&response, DtcStatus::Permanent, "OBD Mode 0A");
        crate::obd::dev_log::log_info("dtc", &format!("Mode 0A (Permanent): found {} DTCs", parsed.len()));
        all_dtcs.extend(parsed);
    }

    // UDS 0x19 02 FF — Read DTC by status mask (all DTCs)
    // Try on standard addresses
    for addr in ["7E0", "7E1", "7E2", "7E3"] {
        crate::obd::dev_log::log_debug("dtc", &format!("Probing UDS DTC scan at address: {}", addr));
        if let Ok(_) = with_real_connection(|conn| {
            conn.send_command(&format!("ATSH{}", addr))
        }) {
            if let Ok(response) = with_real_connection(|conn| conn.send_command("1902FF")) {
                if response.contains("59 02") {
                    // Parse UDS DTC response
                    let bytes: Vec<u8> = response
                        .split_whitespace()
                        .filter_map(|s| u8::from_str_radix(s, 16).ok())
                        .collect();
                    // Skip service ID (59) and subfunction (02)
                    if bytes.len() > 3 {
                        let mut uds_count = 0;
                        for chunk in bytes[3..].chunks(4) {
                            if chunk.len() >= 3 {
                                let code = dtc::decode_dtc_bytes(chunk[0], chunk[1]);
                                let description = dtc::get_dtc_description(&code);
                                let repair_tips = dtc::get_dtc_repair_tips(&code);
                                all_dtcs.push(DtcCode {
                                    code,
                                    description,
                                    status: DtcStatus::Active,
                                    source: format!("UDS 0x19 ({})", addr),
                                    repair_tips,
                                });
                                uds_count += 1;
                            }
                        }
                        crate::obd::dev_log::log_info("dtc", &format!("UDS 0x19 at {}: found {} DTCs", addr, uds_count));
                    }
                }
            }
            // Reset headers
            let _ = with_real_connection(|conn| conn.send_command("ATH0"));
        }
    }

    // Deduplicate by code (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    all_dtcs.retain(|d| seen.insert(d.code.clone()));

    crate::obd::dev_log::log_info("dtc", &format!("DTC scan complete: total {} DTCs found", all_dtcs.len()));
    tracing::info!("Read {} DTCs from vehicle", all_dtcs.len());
    all_dtcs
}

/// Clear all DTCs — sends OBD Mode 04 (with safety check)
#[command]
pub fn clear_dtcs() -> Result<String, String> {
    // Safety guard — Mode 04 is classified as Caution
    let risk = crate::obd::safety::SafetyGuard::check_command("04");
    crate::obd::dev_log::log_info("dtc", &format!("Clear DTCs safety check: {:?}", risk));
    if risk == crate::models::RiskLevel::Blocked {
        crate::obd::dev_log::log_warn("dtc", "Clear DTCs blocked by safety guard");
        return Err("BLOCKED".to_string());
    }

    if is_demo() {
        crate::obd::dev_log::log_info("dtc", "Demo mode: DTCs clear simulated");
        return Ok("OK".to_string());
    }

    // Send Mode 04 (Clear DTCs and stored values)
    with_real_connection(|conn| {
        crate::obd::dev_log::log_info("dtc", "Sending Mode 04 (Clear DTCs)");
        let response = conn.send_command("04")?;
        if response.contains("44") {
            crate::obd::dev_log::log_info("dtc", "Mode 04 response received, DTCs cleared successfully");
            tracing::info!("DTCs cleared successfully");
            Ok("OK".to_string())
        } else {
            crate::obd::dev_log::log_error("dtc", &format!("Clear DTCs failed: {}", response));
            Err(format!("Clear DTCs failed: {}", response))
        }
    })
}

/// Export DTCs to JSON or text
#[command]
pub fn export_dtcs(dtcs: Vec<DtcCode>, format: String) -> Result<String, String> {
    crate::obd::dev_log::log_info("dtc", &format!("Exporting {} DTCs as {}", dtcs.len(), format));
    match format.as_str() {
        "json" => serde_json::to_string_pretty(&dtcs).map_err(|e| format!("Export failed: {}", e)),
        "text" => {
            Ok(dtcs.iter()
                .map(|d| format!("{} - {} [{}]", d.code, d.description, d.source))
                .collect::<Vec<_>>()
                .join("\n"))
        }
        _ => {
            crate::obd::dev_log::log_warn("dtc", &format!("Unsupported export format: {}", format));
            Err("Unsupported format".to_string())
        }
    }
}
