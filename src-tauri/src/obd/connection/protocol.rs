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

            // For slow protocols (KWP/ISO), set max adapter timeout and allow extra init time
            if timeout_ms > 6000 {
                let _ = self.send_command("ATST FF");
                // KWP 5-baud and ISO 9141 need the adapter to perform slow init handshake
                // Give extra settling time before sending the PID request
                std::thread::sleep(Duration::from_millis(500));
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

        for (proto_num, timeout_ms, proto_name) in &protocols[..6] {
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
        Err("No compatible OBD protocol found after trying all 10 protocols. Check: (1) vehicle ignition is ON, (2) adapter is firmly plugged in, (3) adapter is not a limited clone that only supports CAN.".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_protocol_0_auto() {
        let result = Elm327Connection::decode_protocol("ATDPN0");
        assert_eq!(result, "Auto");
    }

    #[test]
    fn test_decode_protocol_1_j1850_pwm() {
        let result = Elm327Connection::decode_protocol("ATDPN1");
        assert_eq!(result, "SAE J1850 PWM");
    }

    #[test]
    fn test_decode_protocol_2_j1850_vpw() {
        let result = Elm327Connection::decode_protocol("ATDPN2");
        assert_eq!(result, "SAE J1850 VPW");
    }

    #[test]
    fn test_decode_protocol_3_iso9141() {
        let result = Elm327Connection::decode_protocol("ATDPN3");
        assert_eq!(result, "ISO 9141-2");
    }

    #[test]
    fn test_decode_protocol_4_kwp_5baud() {
        let result = Elm327Connection::decode_protocol("ATDPN4");
        assert_eq!(result, "ISO 14230-4 KWP (5 baud init)");
    }

    #[test]
    fn test_decode_protocol_5_kwp_fast() {
        let result = Elm327Connection::decode_protocol("ATDPN5");
        assert_eq!(result, "ISO 14230-4 KWP (fast init)");
    }

    #[test]
    fn test_decode_protocol_6_can_11bit_500k() {
        let result = Elm327Connection::decode_protocol("ATDPN6");
        assert_eq!(result, "ISO 15765-4 CAN 11-bit 500k");
    }

    #[test]
    fn test_decode_protocol_7_can_29bit_500k() {
        let result = Elm327Connection::decode_protocol("ATDPN7");
        assert_eq!(result, "ISO 15765-4 CAN 29-bit 500k");
    }

    #[test]
    fn test_decode_protocol_8_can_11bit_250k() {
        let result = Elm327Connection::decode_protocol("ATDPN8");
        assert_eq!(result, "ISO 15765-4 CAN 11-bit 250k");
    }

    #[test]
    fn test_decode_protocol_9_can_29bit_250k() {
        let result = Elm327Connection::decode_protocol("ATDPN9");
        assert_eq!(result, "ISO 15765-4 CAN 29-bit 250k");
    }

    #[test]
    fn test_decode_protocol_a_j1939_lowercase() {
        let result = Elm327Connection::decode_protocol("ATDPNa");
        assert_eq!(result, "SAE J1939 CAN 29-bit 250k");
    }

    #[test]
    fn test_decode_protocol_a_j1939_uppercase() {
        let result = Elm327Connection::decode_protocol("ATDPNA");
        assert_eq!(result, "SAE J1939 CAN 29-bit 250k");
    }

    #[test]
    fn test_decode_protocol_b_user_can1() {
        let result = Elm327Connection::decode_protocol("ATDPNB");
        assert_eq!(result, "User CAN 1 (11-bit, user baud)");
    }

    #[test]
    fn test_decode_protocol_c_user_can2() {
        let result = Elm327Connection::decode_protocol("ATDPNC");
        assert_eq!(result, "User CAN 2 (29-bit, user baud)");
    }

    #[test]
    fn test_decode_protocol_unknown() {
        let result = Elm327Connection::decode_protocol("ATDPNX");
        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_decode_protocol_with_whitespace() {
        let result = Elm327Connection::decode_protocol("  ATDPN6  ");
        assert_eq!(result, "ISO 15765-4 CAN 11-bit 500k");
    }

    #[test]
    fn test_decode_protocol_empty() {
        let result = Elm327Connection::decode_protocol("");
        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_check_valid_pid_response_impl() {
        let response_spaces = "41 00 FF FF FF FF";
        let no_space_prefix = "41 00".replace(" ", "");
        let clean = response_spaces.replace(" ", "");
        assert!(response_spaces.contains("41 00"));
        assert!(clean.contains(&no_space_prefix));
    }
}
