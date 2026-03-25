use tauri::command;
use crate::models::{DtcCode, DtcStatus};
use crate::obd::demo::DemoConnection;
use crate::obd::dtc;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};

/// Read all DTCs — Mode 03 (active), Mode 07 (pending), Mode 0A (permanent) + UDS 0x19
#[command]
pub fn read_all_dtcs(lang: Option<String>) -> Vec<DtcCode> {
    let lang = lang.as_deref().unwrap_or("en");
    if is_demo() {
        dev_log::log_info("dtc", "Demo mode: returning simulated DTCs");
        return DemoConnection::get_dtcs(lang);
    }

    dev_log::log_info("dtc", "Real mode: starting DTC scan");

    let mut all_dtcs: Vec<DtcCode> = Vec::new();

    // ====== OBD-II Standard Modes ======

    // Mode 03 — Confirmed/Active DTCs (with retry)
    match read_dtc_mode_with_retry("03", DtcStatus::Active, "OBD Mode 03", lang) {
        Ok(dtcs) => {
            dev_log::log_info("dtc", &format!("Mode 03 (Active): {} DTCs", dtcs.len()));
            all_dtcs.extend(dtcs);
        }
        Err(e) => dev_log::log_warn("dtc", &format!("Mode 03 failed: {}", e)),
    }

    // Mode 07 — Pending DTCs
    match read_dtc_mode_with_retry("07", DtcStatus::Pending, "OBD Mode 07", lang) {
        Ok(dtcs) => {
            dev_log::log_info("dtc", &format!("Mode 07 (Pending): {} DTCs", dtcs.len()));
            all_dtcs.extend(dtcs);
        }
        Err(e) => dev_log::log_warn("dtc", &format!("Mode 07 failed: {}", e)),
    }

    // Mode 0A — Permanent DTCs (not all vehicles support this)
    match read_dtc_mode_with_retry("0A", DtcStatus::Permanent, "OBD Mode 0A", lang) {
        Ok(dtcs) => {
            dev_log::log_info("dtc", &format!("Mode 0A (Permanent): {} DTCs", dtcs.len()));
            all_dtcs.extend(dtcs);
        }
        Err(e) => dev_log::log_debug("dtc", &format!("Mode 0A not supported: {}", e)),
    }

    // ====== UDS 0x19 — Read DTC by Status Mask ======
    // Try on standard + extended ECU addresses
    let uds_addresses = ["7E0", "7E1", "7E2", "7E3", "7E4", "75D"];

    for addr in uds_addresses {
        dev_log::log_debug("dtc", &format!("Probing UDS DTC at {}", addr));

        // Set header to target specific ECU
        if with_real_connection(|conn| {
            conn.set_ecu_header(addr)
        }).is_err() {
            continue;
        }

        // UDS 0x19 02 FF — Read all DTCs by status mask (all statuses)
        if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("1902FF", 5000)) {
            if response.contains("59 02") || response.contains("5902") {
                let uds_dtcs = parse_uds_dtc_response(&response, addr, lang);
                if !uds_dtcs.is_empty() {
                    dev_log::log_info("dtc", &format!("UDS 0x19 at {}: {} DTCs", addr, uds_dtcs.len()));
                    all_dtcs.extend(uds_dtcs);
                }
            }
        }

        // Also try 0x19 0F FF (mirror memory DTCs) — some ECUs store extra codes here
        if let Ok(response) = with_real_connection(|conn| conn.send_command_timeout("190FFF", 3000)) {
            if response.contains("59 0F") || response.contains("590F") {
                let mirror_dtcs = parse_uds_dtc_response(&response, addr, lang);
                if !mirror_dtcs.is_empty() {
                    dev_log::log_info("dtc", &format!("UDS 0x19 0F at {}: {} mirror DTCs", addr, mirror_dtcs.len()));
                    for mut d in mirror_dtcs {
                        d.status = DtcStatus::Pending; // Mirror = historical
                        d.source = format!("UDS 0x19 0F ({})", addr);
                        all_dtcs.push(d);
                    }
                }
            }
        }
    }

    // Reset headers to broadcast
    let _ = with_real_connection(|conn| conn.reset_headers());

    // ====== Deduplicate ======
    let mut seen = std::collections::HashSet::new();
    all_dtcs.retain(|d| seen.insert(format!("{}:{:?}", d.code, d.status)));

    // ====== Enrich DTCs with ECU context ======
    for dtc in &mut all_dtcs {
        if dtc.ecu_context.is_none() {
            dtc.ecu_context = match dtc.source.as_str() {
                s if s.contains("7E0") => Some("Engine (ECM)".to_string()),
                s if s.contains("7E1") => Some("Transmission (TCM)".to_string()),
                s if s.contains("7E2") => Some("ABS / ESP".to_string()),
                s if s.contains("7E3") => Some("Airbag (SRS)".to_string()),
                s if s.contains("7E4") => Some("Climate / HVAC".to_string()),
                s if s.contains("75D") => Some("BSI / BCM".to_string()),
                _ => None,
            };
        }
    }

    dev_log::log_info("dtc", &format!("DTC scan complete: {} unique DTCs found", all_dtcs.len()));
    tracing::info!("Read {} DTCs from vehicle", all_dtcs.len());
    all_dtcs
}

/// Read DTCs from a specific OBD mode with retry logic
fn read_dtc_mode_with_retry(mode: &str, status: DtcStatus, source: &str, lang: &str) -> Result<Vec<DtcCode>, String> {
    for attempt in 0..2 {
        let response = match with_real_connection(|conn| conn.send_command_timeout(mode, 5000)) {
            Ok(r) => r,
            Err(e) => {
                if attempt == 0 {
                    dev_log::log_debug("dtc", &format!("Mode {} attempt 1 failed: {}, retrying...", mode, e));
                    // Send TesterPresent to wake up ECU before retry
                    let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    continue;
                }
                return Err(e);
            }
        };

        // Handle NO DATA gracefully
        if response.contains("NO DATA") || response.is_empty() {
            return Ok(Vec::new()); // No DTCs — not an error
        }

        let dtcs = dtc::parse_dtc_response(&response, status.clone(), source, lang);
        return Ok(dtcs);
    }
    Err(format!("Mode {} failed after 2 attempts", mode))
}

/// Parse UDS 0x19 response into DTCs
fn parse_uds_dtc_response(response: &str, ecu_addr: &str, lang: &str) -> Vec<DtcCode> {
    let mut dtcs = Vec::new();

    // Parse all hex bytes from the response
    let bytes: Vec<u8> = response
        .replace("\n", " ")
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    // Find the start of DTC data (after 59 02 XX or 59 0F XX)
    let data_start = if bytes.len() > 3 && bytes[0] == 0x59 {
        3 // Skip Service ID (59), SubFunction (02/0F), StatusMask
    } else if let Some(pos) = bytes.windows(2).position(|w| w[0] == 0x59) {
        pos + 3
    } else {
        return dtcs;
    };

    if data_start >= bytes.len() {
        return dtcs;
    }

    // UDS DTCs: 3 bytes DTC + 1 byte status
    for chunk in bytes[data_start..].chunks(4) {
        if chunk.len() >= 3 {
            // First 2 bytes = standard DTC encoding
            let code = dtc::decode_dtc_bytes(chunk[0], chunk[1]);

            // Third byte sometimes contains sub-code (manufacturer specific)
            // Only use if it adds value
            let full_code = if chunk.len() >= 3 && chunk[2] != 0x00 {
                format!("{}-{:02X}", code, chunk[2])
            } else {
                code.clone()
            };

            // Get description for the base code
            let description = dtc::get_dtc_description(&code, lang);
            let repair_tips = dtc::get_dtc_repair_tips(&code, lang);
            let (causes, quick_check, difficulty) = dtc::get_dtc_repair_data(&code, lang);

            // Determine status from the 4th byte (if present)
            let status = if chunk.len() >= 4 {
                let status_byte = chunk[3];
                if status_byte & 0x01 != 0 {
                    DtcStatus::Active   // testFailed
                } else if status_byte & 0x04 != 0 {
                    DtcStatus::Pending  // pendingDTC
                } else if status_byte & 0x08 != 0 {
                    DtcStatus::Active   // confirmedDTC
                } else {
                    DtcStatus::Pending  // Other status
                }
            } else {
                DtcStatus::Active
            };

            dtcs.push(DtcCode {
                code: if full_code.ends_with("-00") { code } else { full_code },
                description,
                status,
                source: format!("UDS 0x19 ({})", ecu_addr),
                repair_tips,
                causes,
                quick_check,
                difficulty,
                ecu_context: None,
            });
        }
    }

    dtcs
}

/// Clear all DTCs — sends OBD Mode 04 (with safety check)
#[command]
pub fn clear_dtcs() -> Result<String, String> {
    let risk = crate::obd::safety::SafetyGuard::check_command("04");
    dev_log::log_info("dtc", &format!("Clear DTCs safety check: {:?}", risk));
    if risk == crate::models::RiskLevel::Blocked {
        dev_log::log_warn("dtc", "Clear DTCs blocked by safety guard");
        return Err(crate::commands::connection::err_msg("BLOQUÉ — commande bloquée par la sécurité", "BLOCKED — command blocked by safety system"));
    }

    if is_demo() {
        dev_log::log_info("dtc", "Demo mode: DTCs clear simulated");
        return Ok("OK".to_string());
    }

    with_real_connection(|conn| {
        dev_log::log_info("dtc", "Sending Mode 04 (Clear DTCs)");

        // Send TesterPresent first to ensure ECU is awake
        conn.tester_present();

        let response = conn.send_command_timeout("04", 5000)?;
        if response.contains("44") {
            dev_log::log_info("dtc", "DTCs cleared successfully");
            tracing::info!("DTCs cleared successfully");
            Ok("OK".to_string())
        } else if response.is_empty() || response.contains("NO DATA") {
            // Some vehicles don't respond to Mode 04 but still clear
            dev_log::log_warn("dtc", "No response to Mode 04 — DTCs may have been cleared");
            Ok("OK".to_string())
        } else {
            dev_log::log_error("dtc", &format!("Clear DTCs failed: {}", response));
            Err(format!("Clear DTCs failed: {}", response))
        }
    })
}

/// Export DTCs to JSON or text
#[command]
pub fn export_dtcs(dtcs: Vec<DtcCode>, format: String) -> Result<String, String> {
    dev_log::log_info("dtc", &format!("Exporting {} DTCs as {}", dtcs.len(), format));
    match format.as_str() {
        "json" => serde_json::to_string_pretty(&dtcs).map_err(|e| format!("Export failed: {}", e)),
        "text" => {
            Ok(dtcs.iter()
                .map(|d| format!("{} - {} [{}] ({:?})", d.code, d.description, d.source, d.status))
                .collect::<Vec<_>>()
                .join("\n"))
        }
        _ => {
            dev_log::log_warn("dtc", &format!("Unsupported export format: {}", format));
            Err(crate::commands::connection::err_msg("Format non supporté", "Unsupported format"))
        }
    }
}
