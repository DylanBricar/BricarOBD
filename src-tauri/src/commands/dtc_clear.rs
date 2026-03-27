use tauri::command;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};
use crate::commands::OBDBusyGuard;

/// Wait for ECU to finish processing when NRC 0x78 (ResponsePending) is received.
/// Some ECUs need time to erase flash memory — they send 0x78 while working.
/// The ELM327 handles 0x78 internally: it keeps waiting for the ECU's final response.
/// We just need to re-read with a much longer timeout — do NOT re-send the command,
/// as that would start a new request and confuse the ECU (NRC 0x21 busyRepeatRequest).
fn wait_for_response_pending(conn: &mut crate::obd::connection::Elm327Connection, max_wait_ms: u64) -> Result<String, String> {
    // The ELM327 is still waiting for the ECU's final response after the 0x78.
    // We just need to read with an extended timeout. Send empty CR to trigger read.
    // Some adapters need a nudge to flush their buffer.
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try reading with the full remaining timeout
    match conn.send_command_timeout("", max_wait_ms) {
        Ok(r) if !r.is_empty() && r != "?" && !r.contains("NO DATA") => Ok(r),
        Ok(r) if r == "?" => {
            // ELM327 already sent the response before our nudge — it's gone.
            // The original send_command already got the 0x78, so the real response
            // may have been consumed. Try the command one more time.
            dev_log::log_debug("dtc", "ELM327 returned '?' — response may have been consumed, retrying");
            Err("ECU response lost during pending wait".to_string())
        }
        _ => Err("ECU timed out while processing clear request".to_string()),
    }
}

/// Extract NRC byte from a negative response.
/// Looks for "7F XX YY" pattern where XX is the service ID and YY is the NRC.
fn extract_nrc(response: &str) -> Option<u8> {
    let parts: Vec<&str> = response.split_whitespace().collect();
    for window in parts.windows(3) {
        if window[0] == "7F" {
            if let Ok(nrc) = u8::from_str_radix(window[2], 16) {
                return Some(nrc);
            }
        }
    }
    // Also try unspaced: "7F04XX" or "7F14XX"
    let upper = response.to_uppercase().replace(" ", "");
    if let Some(pos) = upper.find("7F") {
        if pos + 6 <= upper.len() {
            if let Ok(nrc) = u8::from_str_radix(&upper[pos + 4..pos + 6], 16) {
                return Some(nrc);
            }
        }
    }
    None
}

/// Parse a clear DTC response — handles positive and negative responses
fn parse_clear_response(response: &str, method: &str) -> Result<(), String> {
    if response.is_empty() || response.contains("NO DATA") {
        return Err("NO_RESPONSE".to_string());
    }

    // Check for negative response first (before positive, to avoid false match on "44" inside NRC)
    if response.contains("7F") {
        if let Some(nrc) = extract_nrc(response) {
            return match nrc {
                0x78 => Err("RESPONSE_PENDING".to_string()),
                0x22 => Err(crate::commands::connection::err_msg(
                    "L'ECU refuse l'effacement — conditions non remplies (contact mis, moteur arrêté requis)",
                    "ECU refused clear — conditions not met (key on, engine off required)"
                )),
                0x31 => Err(crate::commands::connection::err_msg(
                    "Adresse/requête hors plage",
                    "Request out of range"
                )),
                0x11 | 0x12 => Err(crate::commands::connection::err_msg(
                    "Service non supporté par cet ECU",
                    "Service not supported by this ECU"
                )),
                0x33 | 0x35 | 0x36 | 0x37 => Err(crate::commands::connection::err_msg(
                    "L'ECU exige un déverrouillage de sécurité — non disponible en mode standard",
                    "ECU requires SecurityAccess unlock — not available in standard mode"
                )),
                0x72 => Err(crate::commands::connection::err_msg(
                    "Erreur de programmation — réessayez après avoir coupé/remis le contact",
                    "Programming failure — try cycling ignition off/on"
                )),
                _ => Err(crate::commands::connection::err_msg(
                    &format!("L'ECU a refusé l'effacement (NRC: 0x{:02X}, réponse: {})", nrc, response),
                    &format!("ECU refused the clear request (NRC: 0x{:02X}, response: {})", nrc, response)
                )),
            };
        }
    }

    // Positive response: Mode 04 → "44", UDS 0x14 → "54"
    // Check for exact token match (not substring) to avoid matching hex data
    let tokens: Vec<&str> = response.split_whitespace().collect();
    let has_positive = tokens.iter().any(|t| *t == "44" || *t == "54");
    if has_positive {
        dev_log::log_info("dtc", &format!("{}: DTCs cleared successfully", method));
        return Ok(());
    }
    // Also check unspaced format
    let upper = response.to_uppercase().replace(" ", "");
    if upper.starts_with("44") || upper.starts_with("54") {
        dev_log::log_info("dtc", &format!("{}: DTCs cleared successfully", method));
        return Ok(());
    }

    Err(format!("Unexpected response: {}", response))
}

/// Clear all DTCs — Mode 04 (OBD-II) + UDS 0x14 per ECU (with safety check)
/// Uses 3 strategies to maximize compatibility across all vehicle types:
/// 1. Standard Mode 04 broadcast (works on most OBD-II vehicles)
/// 2. UDS 0x14 FFFFFF per ECU with extended session (European cars, multi-ECU)
/// 3. Mode 04 with extended diagnostic session (vehicles that need session switch)
#[command]
pub async fn clear_dtcs(confirmed: Option<bool>) -> Result<String, String> {
    let risk = crate::obd::safety::SafetyGuard::check_command("04");
    dev_log::log_info("dtc", &format!("Clear DTCs safety check: {:?}", risk));
    if risk == crate::models::RiskLevel::Blocked {
        dev_log::log_warn("dtc", "Clear DTCs blocked by safety guard");
        return Err(crate::commands::connection::err_msg("BLOQUÉ — commande bloquée par la sécurité", "BLOCKED — command blocked by safety system"));
    }

    if risk == crate::models::RiskLevel::Caution && confirmed != Some(true) {
        dev_log::log_info("dtc", "Clear DTCs requires confirmation");
        return Err("CONFIRM_REQUIRED".to_string());
    }

    if is_demo() {
        dev_log::log_info("dtc", "Demo mode: DTCs clear simulated");
        return Ok("OK".to_string());
    }

    tokio::task::spawn_blocking(move || {
        let _guard = match OBDBusyGuard::try_acquire() {
            Ok(g) => g,
            Err(e) => {
                dev_log::log_warn("dtc", &format!("Clear DTCs blocked: {}", e));
                return Err(crate::commands::connection::err_msg(
                    "Une opération OBD est déjà en cours",
                    "An OBD operation is already in progress"
                ));
            }
        };

        let mut cleared = false;
        // Keep the most relevant error for the user — NRC 0x22 "conditions not met"
        // is the most actionable (tells user to turn engine off), so it takes priority.
        let mut last_error = String::new();
        let has_conditions_error = |e: &str| e.contains("conditions") || e.contains("KOEO");

        // ====== Strategy 1: OBD-II Mode 04 broadcast ======
        dev_log::log_info("dtc", "Strategy 1: OBD-II Mode 04 (broadcast)");
        match clear_with_mode04() {
            Ok(()) => { cleared = true; }
            Err(e) => {
                dev_log::log_warn("dtc", &format!("Mode 04 broadcast failed: {}", e));
                last_error = e;
            }
        }

        // ====== Strategy 2: UDS 0x14 FFFFFF per ECU ======
        // UDS services only work on CAN and KWP protocols (not J1850 or ISO 9141)
        if !cleared {
            let is_uds_capable = with_real_connection(|conn| {
                let proto = conn.protocol_num.as_str();
                // CAN: 6,7,8,9,A,B,C — KWP: 4,5 — ISO 9141: 3 — J1850: 1,2
                Ok(matches!(proto, "4" | "5" | "6" | "7" | "8" | "9" | "A" | "B" | "C"))
            }).unwrap_or(false);

            if is_uds_capable {
                dev_log::log_info("dtc", "Strategy 2: UDS 0x14 per ECU with extended session");
                let ecu_addresses = ["7E0", "7E1", "7E2", "7E3", "7E4", "75D", "7C0", "7C1", "7A0", "740", "710", "714"];

                for addr in ecu_addresses {
                    match clear_uds_on_ecu(addr) {
                        Ok(()) => {
                            cleared = true;
                        }
                        Err(e) if e.contains("NO_RESPONSE") || e.contains("not supported") => {
                            dev_log::log_debug("dtc", &format!("UDS 0x14 at {}: skipped ({})", addr, e));
                        }
                        Err(e) => {
                            dev_log::log_warn("dtc", &format!("UDS 0x14 at {} failed: {}", addr, e));
                            if !has_conditions_error(&last_error) {
                                last_error = e;
                            }
                        }
                    }
                }
                let _ = with_real_connection(|conn| conn.reset_headers());
            } else {
                dev_log::log_debug("dtc", "Strategy 2 skipped: protocol does not support UDS");
            }
        }

        // ====== Strategy 3: Mode 04 with extended diagnostic session ======
        // Only on CAN/KWP — extended sessions don't exist on J1850/ISO 9141
        if !cleared {
            let is_uds_capable = with_real_connection(|conn| {
                let proto = conn.protocol_num.as_str();
                Ok(matches!(proto, "4" | "5" | "6" | "7" | "8" | "9" | "A" | "B" | "C"))
            }).unwrap_or(false);

            if is_uds_capable {
                dev_log::log_info("dtc", "Strategy 3: Mode 04 with extended diagnostic session");
                match clear_mode04_extended_session() {
                    Ok(()) => { cleared = true; }
                    Err(e) => {
                        dev_log::log_warn("dtc", &format!("Mode 04 extended session failed: {}", e));
                        if !has_conditions_error(&last_error) { last_error = e; }
                    }
                }
            }
        }

        // ====== Strategy 4: Retry Mode 04 after adapter recovery ======
        // For stubborn adapters (clones) that got confused by UDS commands
        if !cleared {
            dev_log::log_info("dtc", "Strategy 4: Mode 04 after adapter recovery");
            match clear_mode04_with_recovery() {
                Ok(()) => { cleared = true; }
                Err(e) => {
                    dev_log::log_warn("dtc", &format!("Mode 04 recovery failed: {}", e));
                    if !has_conditions_error(&last_error) { last_error = e; }
                }
            }
        }

        // Always clean up: return to default session + broadcast headers
        let _ = with_real_connection(|conn| {
            let _ = conn.send_command_timeout("10 01", 2000);
            conn.reset_headers()
        });

        if !cleared {
            dev_log::log_error("dtc", &format!("All clear strategies failed. Last: {}", last_error));
            return Err(if last_error.is_empty() {
                crate::commands::connection::err_msg(
                    "Aucun ECU n'a répondu. Vérifiez que le contact est mis et le moteur arrêté.",
                    "No ECU responded. Ensure ignition is ON and engine is OFF."
                )
            } else {
                last_error
            });
        }

        // ====== Post-clear verification ======
        dev_log::log_info("dtc", "Verifying DTCs were cleared...");
        std::thread::sleep(std::time::Duration::from_millis(500));
        match with_real_connection(|conn| conn.send_command_timeout("03", 5000)) {
            Ok(response) => {
                if response.contains("NO DATA") || response.contains("43 00") || response.trim().is_empty() {
                    dev_log::log_info("dtc", "Verification OK: no DTCs remaining");
                    tracing::info!("DTCs cleared and verified successfully");
                    Ok("OK".to_string())
                } else if response.contains("43") {
                    dev_log::log_warn("dtc", &format!("Some DTCs still present after clear: {}", response));
                    Ok("PARTIAL".to_string())
                } else {
                    dev_log::log_info("dtc", "Clear completed (verification inconclusive)");
                    Ok("OK".to_string())
                }
            }
            Err(_) => {
                dev_log::log_warn("dtc", "Post-clear verification failed, but clear was acknowledged");
                Ok("OK".to_string())
            }
        }
    }).await.map_err(|e| format!("Task error: {}", e))?
}

/// Strategy 1: Standard OBD-II Mode 04 broadcast clear
fn clear_with_mode04() -> Result<(), String> {
    with_real_connection(|conn| {
        // Ensure we're in broadcast mode with a clean state
        let _ = conn.reset_headers();
        conn.tester_present();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 10s timeout: some ECU (especially on KWP/ISO) need time to process clear
        let response = conn.send_command_timeout("04", 10000)?;
        match parse_clear_response(&response, "Mode 04") {
            Ok(()) => Ok(()),
            Err(e) if e == "RESPONSE_PENDING" => {
                dev_log::log_info("dtc", "ECU processing (NRC 0x78)... waiting up to 15s");
                let final_resp = wait_for_response_pending(conn, 15000)?;
                parse_clear_response(&final_resp, "Mode 04 (after wait)")
            }
            Err(e) => Err(e),
        }
    })
}

/// Strategy 2: UDS 0x14 FFFFFF on a specific ECU with extended diagnostic session
fn clear_uds_on_ecu(ecu_addr: &str) -> Result<(), String> {
    with_real_connection(|conn| {
        conn.set_ecu_header(ecu_addr)?;

        // Check if ECU is alive
        let tp_resp = conn.send_command_timeout("3E00", 2000);
        if tp_resp.is_err() || tp_resp.as_ref().map_or(true, |r| r.contains("NO DATA") || r.contains("ERROR")) {
            return Err("NO_RESPONSE".to_string());
        }

        // Open extended diagnostic session (0x10 03) — many ECUs require this
        // Don't use ? — if session fails, still try the clear (some ECU accept 0x14 in default session)
        match conn.send_command_timeout("10 03", 3000) {
            Ok(r) if r.contains("50 03") || r.contains("5003") => {
                dev_log::log_debug("dtc", &format!("{}: Extended session opened", ecu_addr));
            }
            Ok(r) => {
                dev_log::log_debug("dtc", &format!("{}: Extended session not confirmed ({}), trying clear anyway", ecu_addr, r));
            }
            Err(e) => {
                dev_log::log_debug("dtc", &format!("{}: Extended session failed ({}), trying clear anyway", ecu_addr, e));
            }
        }

        // UDS 0x14 FF FF FF — Clear all DTC groups
        let response = conn.send_command_timeout("14 FF FF FF", 8000)?;
        match parse_clear_response(&response, &format!("UDS 0x14 ({})", ecu_addr)) {
            Ok(()) => Ok(()),
            Err(e) if e == "RESPONSE_PENDING" => {
                dev_log::log_info("dtc", &format!("{}: ECU processing clear (NRC 0x78)...", ecu_addr));
                let final_resp = wait_for_response_pending(conn, 15000)?;
                parse_clear_response(&final_resp, &format!("UDS 0x14 ({}) after wait", ecu_addr))
            }
            Err(e) => Err(e),
        }
    })
}

/// Strategy 4: Mode 04 after full adapter recovery — for stubborn adapters/clones
/// that may have gotten confused by previous UDS commands or session switches
fn clear_mode04_with_recovery() -> Result<(), String> {
    with_real_connection(|conn| {
        // Full recovery: flush buffer, reset adapter state, restore protocol
        conn.reset_headers()?;
        let _ = conn.send_command_timeout("10 01", 2000); // Default session
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Re-set protocol to ensure we're on the right bus
        if !conn.protocol_num.is_empty() {
            let _ = conn.send_command(&format!("ATSP{}", conn.protocol_num));
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        // Wake ECU and try
        conn.tester_present();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let response = conn.send_command_timeout("04", 10000)?;
        parse_clear_response(&response, "Mode 04 (after recovery)")
    })
}

/// Strategy 3: Mode 04 with extended diagnostic session opened first
fn clear_mode04_extended_session() -> Result<(), String> {
    with_real_connection(|conn| {
        conn.reset_headers()?;
        conn.tester_present();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Open extended diagnostic session
        let _ = conn.send_command_timeout("10 03", 3000);
        std::thread::sleep(std::time::Duration::from_millis(200));

        let response = conn.send_command_timeout("04", 8000)?;
        match parse_clear_response(&response, "Mode 04 (extended session)") {
            Ok(()) => Ok(()),
            Err(e) if e == "RESPONSE_PENDING" => {
                dev_log::log_info("dtc", "ECU processing (NRC 0x78) in extended session...");
                let final_resp = wait_for_response_pending(conn, 15000)?;
                parse_clear_response(&final_resp, "Mode 04 (extended, after wait)")
            }
            Err(e) => Err(e),
        }
    })
}
