use std::time::Duration;
use tracing::{debug, info, warn};
use crate::obd::dev_log;
use super::Elm327Connection;

impl Elm327Connection {
    // ==================== PROTOCOL DETECTION ====================

    /// Detect OBD protocol — tries auto-detect first, then manual cycling
    pub(super) fn detect_protocol(&mut self) -> Result<(), String> {
        // First: wake up the ECU with TesterPresent (works even if protocol is wrong)
        // This ensures the ECU is not in sleep mode
        dev_log::log_debug("obd", "Sending wake-up TesterPresent before protocol detection");

        // Try auto-detect with ATSP0
        let _ = self.send_command("ATSP0");
        std::thread::sleep(Duration::from_millis(300));

        // Send 0100 with generous timeout — auto-detect can take 5-10s
        let response = match self.send_command_timeout("0100", 10000) {
            Ok(r) => r,
            Err(_) => String::new(),
        };

        if self.check_valid_pid_response(&response, "41 00") {
            let proto = self.send_command("ATDPN").unwrap_or_default();
            self.protocol_num = proto.trim().chars().last().map(|c| c.to_string()).unwrap_or_default();
            self.protocol = Self::decode_protocol(&proto);
            self.parse_supported_pids_from_response(&response);
            dev_log::log_info("obd", &format!("Auto-detect found protocol: {} ({})", self.protocol, self.protocol_num));
            return Ok(());
        }

        // Auto-detect failed — try manual protocol cycling
        dev_log::log_warn("obd", "Auto-detect failed, cycling protocols manually...");
        self.cycle_protocols()
    }

    /// Cycle through protocols — handles errors gracefully, tries multiple approaches per protocol
    fn cycle_protocols(&mut self) -> Result<(), String> {
        // Order: most common first (CAN 500k, CAN 250k, KWP fast, KWP slow, ISO, J1850)
        let protocols = [
            ("6", 6000,  "ISO 15765-4 CAN 11-bit 500k"),  // Most post-2008 vehicles
            ("8", 6000,  "ISO 15765-4 CAN 11-bit 250k"),
            ("7", 6000,  "ISO 15765-4 CAN 29-bit 500k"),
            ("9", 6000,  "ISO 15765-4 CAN 29-bit 250k"),
            ("5", 8000,  "ISO 14230-4 KWP fast init"),
            ("4", 14000, "ISO 14230-4 KWP 5-baud init"),  // Needs 10s+ for slow init
            ("3", 12000, "ISO 9141-2"),                    // Slow init
            ("1", 6000,  "SAE J1850 PWM"),
            ("2", 6000,  "SAE J1850 VPW"),
            ("A", 6000,  "SAE J1939 CAN 29-bit 250k"),    // Trucks/heavy vehicles
        ];

        for (proto_num, timeout_ms, proto_name) in protocols {
            dev_log::log_debug("obd", &format!("Trying protocol ATSP{} ({}) — timeout {}ms", proto_num, proto_name, timeout_ms));

            let _ = self.send_command(&format!("ATSP{}", proto_num));
            std::thread::sleep(Duration::from_millis(300));

            // For slow protocols (KWP/ISO), set max adapter timeout
            if timeout_ms > 6000 {
                let _ = self.send_command("ATST FF");
            }

            // Try 0100 first (standard PID request)
            let response = match self.send_command_timeout("0100", timeout_ms as u64) {
                Ok(r) => r,
                Err(e) => {
                    debug!("Protocol {} error: {}", proto_num, e);
                    // Restore normal timeout before trying next
                    if timeout_ms > 6000 {
                        let _ = self.send_command("ATST 64");
                    }
                    continue;
                }
            };

            // Check for valid response
            if self.check_valid_pid_response(&response, "41 00") {
                self.protocol_num = proto_num.to_string();
                self.protocol = proto_name.to_string();
                self.parse_supported_pids_from_response(&response);
                // Restore normal timeout
                let _ = self.send_command("ATST 64");
                dev_log::log_info("obd", &format!("Found working protocol: {} (ATSP{})", proto_name, proto_num));
                info!("Found working protocol: {} (ATSP{})", proto_name, proto_num);
                return Ok(());
            }

            // Check for partial success — ECU responded but with an error (communication exists)
            if response.contains("41") || response.contains("7F") || response.contains("7E") {
                warn!("Protocol {} got partial response: {}", proto_num, response);
                dev_log::log_warn("obd", &format!("Protocol {} partial response — using it: {}", proto_num, response));
                self.protocol_num = proto_num.to_string();
                self.protocol = proto_name.to_string();
                let _ = self.send_command("ATST 64");
                return Ok(());
            }

            // Restore normal timeout
            if timeout_ms > 6000 {
                let _ = self.send_command("ATST 64");
            }
        }

        // Last resort: try with headers ON — some vehicles only respond when headers are enabled
        dev_log::log_warn("obd", "All protocols failed with headers off, trying with headers on...");
        self.set_headers(true);

        for (proto_num, timeout_ms, proto_name) in &protocols[..4] {
            let _ = self.send_command(&format!("ATSP{}", proto_num));
            std::thread::sleep(Duration::from_millis(200));

            if let Ok(response) = self.send_command_timeout("0100", *timeout_ms as u64) {
                if response.contains("41 00") || response.contains("4100") {
                    self.protocol_num = proto_num.to_string();
                    self.protocol = proto_name.to_string();
                    self.set_headers(false);
                    dev_log::log_info("obd", &format!("Found protocol with headers: {} (ATSP{})", proto_name, proto_num));
                    return Ok(());
                }
            }
        }

        self.set_headers(false);
        Err("No compatible OBD protocol found after trying all 10 protocols".to_string())
    }

    /// Check if response contains a valid PID response (handles spaces/no-spaces, multi-line)
    pub(super) fn check_valid_pid_response(&self, response: &str, expected_prefix: &str) -> bool {
        if response.contains(expected_prefix) {
            return true;
        }
        // Try without spaces (some adapters strip spaces)
        let no_space_prefix = expected_prefix.replace(" ", "");
        if response.replace(" ", "").contains(&no_space_prefix) {
            return true;
        }
        false
    }

    /// Decode protocol number to human-readable name
    pub(super) fn decode_protocol(proto_str: &str) -> String {
        let num = proto_str.trim().chars().last().unwrap_or('?');
        match num {
            '0' => "Auto",
            '1' => "SAE J1850 PWM",
            '2' => "SAE J1850 VPW",
            '3' => "ISO 9141-2",
            '4' => "ISO 14230-4 KWP (5 baud init)",
            '5' => "ISO 14230-4 KWP (fast init)",
            '6' => "ISO 15765-4 CAN 11-bit 500k",
            '7' => "ISO 15765-4 CAN 29-bit 500k",
            '8' => "ISO 15765-4 CAN 11-bit 250k",
            '9' => "ISO 15765-4 CAN 29-bit 250k",
            'A' | 'a' => "SAE J1939 CAN 29-bit 250k",
            'B' | 'b' => "User CAN 1 (11-bit, user baud)",
            'C' | 'c' => "User CAN 2 (29-bit, user baud)",
            _ => "Unknown",
        }
        .to_string()
    }
}
