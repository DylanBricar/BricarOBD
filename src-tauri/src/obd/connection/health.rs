use std::time::Duration;
use crate::obd::dev_log;
use super::Elm327Connection;

impl Elm327Connection {
    // ==================== KEEP-ALIVE / HEALTH ====================

    /// Send TesterPresent to keep ECU session alive (call every 2-3s during idle)
    pub fn tester_present(&mut self) -> bool {
        match self.send_command_timeout("3E00", 2000) {
            Ok(r) => r.contains("7E") || r.contains("7e"),
            Err(_) => false,
        }
    }

    /// Maximum consecutive errors before the connection is considered dead
    const MAX_CONSECUTIVE_ERRORS: u32 = 10;

    /// Check if the connection is still healthy
    pub fn health_check(&mut self) -> bool {
        // Quick check: read voltage (AT command, doesn't go to CAN bus)
        if let Ok(response) = self.send_command_timeout("ATRV", 2000) {
            if response.contains('.') && !response.contains("ERROR") {
                self.consecutive_errors = 0;
                return true;
            }
        }
        self.consecutive_errors += 1;

        // If too many errors, force disconnect to prevent infinite retry loops
        if self.consecutive_errors >= Self::MAX_CONSECUTIVE_ERRORS {
            dev_log::log_error("obd", &format!(
                "Connection dead: {} consecutive errors — forcing disconnect",
                self.consecutive_errors
            ));
        }

        false
    }

    /// Returns true if consecutive errors have exceeded the max threshold
    pub fn is_fatally_errored(&self) -> bool {
        self.consecutive_errors >= Self::MAX_CONSECUTIVE_ERRORS
    }

    /// Attempt to recover a broken connection (call when consecutive errors > 3)
    /// Times out after 15 seconds to prevent hanging the application.
    pub fn attempt_recovery(&mut self) -> Result<(), String> {
        dev_log::log_warn("obd", &format!("Attempting recovery (consecutive errors: {})", self.consecutive_errors));

        let recovery_start = std::time::Instant::now();
        let recovery_timeout = Duration::from_secs(15);

        // Step 1: Flush buffer
        self.flush_buffer();
        if recovery_start.elapsed() > recovery_timeout {
            return Err("Recovery timed out".to_string());
        }
        std::thread::sleep(Duration::from_millis(300));

        // Step 2: Send CRs to reset adapter state
        if let Some(ref mut transport) = self.transport {
            let _ = transport.write_bytes(b"\r\r\r");
            let _ = transport.flush();
        }
        std::thread::sleep(Duration::from_millis(200));
        self.flush_buffer();

        // Step 3: Re-configure
        let _ = self.send_command("ATE0");
        let _ = self.send_command("ATS1");

        // Check timeout before step 4
        if recovery_start.elapsed() > recovery_timeout {
            return Err("Recovery timed out".to_string());
        }

        // Step 4: Verify with ATRV
        if let Ok(response) = self.send_command_timeout("ATRV", 2000) {
            if response.contains('.') {
                // Step 5: Restore protocol
                if !self.protocol_num.is_empty() {
                    let _ = self.send_command(&format!("ATSP{}", self.protocol_num));
                }
                self.consecutive_errors = 0;
                dev_log::log_info("obd", "Recovery successful");
                return Ok(());
            }
        }

        // Check timeout before step 6
        if recovery_start.elapsed() > recovery_timeout {
            return Err("Recovery timed out".to_string());
        }

        // Step 6: If still failing, try full re-detect
        let _ = self.send_command("ATSP0");
        std::thread::sleep(Duration::from_millis(300));
        if let Ok(response) = self.send_command_timeout("0100", 8000) {
            if self.check_valid_pid_response(&response, "41 00") {
                let proto = self.send_command("ATDPN").unwrap_or_default();
                self.protocol_num = proto.trim().chars().last().map(|c| c.to_string()).unwrap_or_default();
                self.protocol = Self::decode_protocol(&proto);
                self.consecutive_errors = 0;
                dev_log::log_info("obd", "Recovery via re-detect successful");
                return Ok(());
            }
        }

        Err("Recovery failed — adapter may need physical reconnection".to_string())
    }
}
