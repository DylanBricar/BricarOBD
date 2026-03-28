use tauri::command;
use crate::models::{Mode06Result, FreezeFrameData, PidValue};
use crate::obd::{demo::DemoConnection, pid, dtc, dev_log};
use crate::commands::connection::{is_demo, with_real_connection};
use crate::commands::mode06_names::get_mode06_name;

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
        // Find "46" marker followed by non-zero TID
        let pos = match bytes.windows(2).position(|w| w[0] == 0x46 && w[1] != 0x00) {
            Some(p) => p,
            None => continue,
        };

        if pos + 8 >= bytes.len() { continue; }

        let tid = bytes[pos + 1];

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

        // Check if OBD is busy
        if super::connection::is_obd_busy() {
            dev_log::log_debug("diagnostic", "OBD is busy, skipping Mode 06 read");
            return Vec::new();
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

        // Check if OBD is busy
        if super::connection::is_obd_busy() {
            dev_log::log_debug("diagnostic", "OBD is busy, skipping freeze frame read");
            return Vec::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode06_bitmap_empty() {
        let response = "";
        let bitmap = parse_mode06_bitmap(response);
        assert_eq!(bitmap.len(), 0);
    }

    #[test]
    fn test_parse_mode06_bitmap_single_tid() {
        let response = "46 00 80 00 00 00";
        let bitmap = parse_mode06_bitmap(response);
        assert!(bitmap.contains(&0x01));
        assert_eq!(bitmap.len(), 1);
    }

    #[test]
    fn test_parse_mode06_bitmap_multiple_tids() {
        let response = "46 00 81 00 00 00";
        let bitmap = parse_mode06_bitmap(response);
        assert!(bitmap.contains(&0x01));
        assert!(bitmap.contains(&0x08));
        assert_eq!(bitmap.len(), 2);
    }

    #[test]
    fn test_parse_mode06_bitmap_all_bits_set() {
        let response = "46 00 FF FF FF FF";
        let bitmap = parse_mode06_bitmap(response);
        assert!(bitmap.len() > 0);
    }

    #[test]
    fn test_parse_mode06_bitmap_no_46_prefix() {
        let response = "45 00 80 00 00 00";
        let bitmap = parse_mode06_bitmap(response);
        assert_eq!(bitmap.len(), 0);
    }

    #[test]
    fn test_parse_mode06_bitmap_second_byte_only() {
        let response = "46 00 00 80 00 00";
        let bitmap = parse_mode06_bitmap(response);
        assert!(bitmap.contains(&0x09)); // Bit 0 of byte 1 → TID 9
    }

    #[test]
    fn test_parse_mode06_bitmap_tid_exceeds_0xa0() {
        let response = "46 00 00 00 00 80"; // Would be TID > 0xA0
        let bitmap = parse_mode06_bitmap(response);
        // Should exclude TIDs > 0xA0
        assert!(bitmap.iter().all(|tid| *tid <= 0xA0));
    }

    #[test]
    fn test_parse_mode06_results_empty() {
        let response = "";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_mode06_results_single_result() {
        let response = "46 01 00 00 50 00 30 00 70";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tid, 0x01);
        assert_eq!(results[0].mid, 0x00);
        assert_eq!(results[0].test_value, 80.0);
    }

    #[test]
    fn test_parse_mode06_results_test_passed() {
        let response = "46 01 00 00 50 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn test_parse_mode06_results_test_failed() {
        let response = "46 01 00 00 20 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn test_parse_mode06_results_test_at_lower_bound() {
        let response = "46 01 00 00 40 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert!(results[0].passed);
    }

    #[test]
    fn test_parse_mode06_results_test_at_upper_bound() {
        let response = "46 01 00 00 60 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert!(results[0].passed);
    }

    #[test]
    fn test_parse_mode06_results_test_below_lower_bound() {
        let response = "46 01 00 00 39 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert!(!results[0].passed);
    }

    #[test]
    fn test_parse_mode06_results_test_above_upper_bound() {
        let response = "46 01 00 00 61 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert!(!results[0].passed);
    }

    #[test]
    fn test_parse_mode06_results_multiple_results() {
        let response = "46 01 00 00 50 00 40 00 60\n46 02 00 00 25 00 20 00 30";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].tid, 0x01);
        assert_eq!(results[1].tid, 0x02);
    }

    #[test]
    fn test_parse_mode06_results_skips_bitmap() {
        let response = "46 00 80 00 00 00\n46 01 00 00 50 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        // Should skip "46 00" bitmap response
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tid, 0x01);
    }

    #[test]
    fn test_parse_mode06_results_insufficient_data() {
        let response = "46 01 00";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_mode06_results_no_46_marker() {
        let response = "45 01 00 00 50 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_mode06_results_large_test_values() {
        let response = "46 01 00 FF FF 00 00 FF FF";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].test_value, 65535.0);
        assert_eq!(results[0].max_limit, 65535.0);
    }

    #[test]
    fn test_parse_mode06_results_lang_en() {
        let response = "46 01 00 00 50 00 40 00 60";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 1);
        // Name should be set (get_mode06_name will return defaults if not found)
        assert!(!results[0].name.is_empty());
    }

    #[test]
    fn test_parse_mode06_results_lang_fr() {
        let response = "46 01 00 00 50 00 40 00 60";
        let results = parse_mode06_results(response, "fr");
        assert_eq!(results.len(), 1);
        assert!(!results[0].name.is_empty());
    }

    #[test]
    fn test_parse_mode06_results_with_extra_bytes() {
        let response = "46 01 00 00 50 00 40 00 60 EXTRA";
        let results = parse_mode06_results(response, "en");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tid, 0x01);
    }
}


