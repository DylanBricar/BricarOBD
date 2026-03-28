use std::time::Duration;
use crate::obd::dev_log;
use crate::obd::transport::TransportType;
use super::{Elm327Connection, ChipType};

impl Elm327Connection {
    // ==================== SEND / RECEIVE ====================

    /// Send command with default timeout
    pub(crate) fn send_command(&mut self, cmd: &str) -> Result<String, String> {
        self.send_command_timeout(cmd, self.config.timeout_ms)
    }

    /// Send command with custom timeout — core communication method
    /// Works over any OBDTransport (serial, WiFi, BLE)
    pub(crate) fn send_command_timeout(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, String> {
        let transport = self.transport.as_mut().ok_or("Not connected")?;

        // Log TX
        dev_log::log_tx(cmd);

        // Inter-command delay — clones need more time to process
        let delay = match self.chip_type {
            ChipType::CloneV15 | ChipType::CloneV21 => 50,
            ChipType::Stn1110 | ChipType::Stn2120 => 15,
            _ => 30,
        };
        // WiFi adds latency — moderate increase (50ms floor balances reliability vs speed)
        let delay = if transport.transport_type() == TransportType::WiFi {
            delay.max(50)
        } else {
            delay
        };

        let elapsed = self.last_command_time.elapsed().as_millis() as u64;
        if elapsed < delay {
            std::thread::sleep(Duration::from_millis(delay - elapsed));
        }

        // Send command
        let cmd_bytes = format!("{}\r", cmd);
        transport.write_bytes(cmd_bytes.as_bytes())
            .map_err(|e| { dev_log::log_error("obd", &format!("Write error: {}", e)); e })?;
        transport.flush().map_err(|e| format!("Flush error: {}", e))?;

        self.last_command_time = std::time::Instant::now();

        // Read response (wait for prompt '>')
        let mut response = String::new();
        let max_response_size: usize = 65536; // 64KB cap to prevent OOM from misbehaving adapter
        let mut buf = [0u8; 256]; // Larger buffer for multi-frame responses
        let timeout = std::time::Instant::now();

        // WiFi needs more generous timeout due to TCP latency
        // AT commands (adapter-local) need less boost than OBD commands (CAN bus round-trip)
        let effective_timeout = if transport.transport_type() == TransportType::WiFi {
            let is_at_command = cmd.starts_with("AT") || cmd.starts_with("at");
            if is_at_command {
                timeout_ms.max(1500)  // AT commands: 1.5s floor
            } else {
                timeout_ms.max(3000)  // OBD commands: 3s floor for CAN bus latency
            }
        } else {
            timeout_ms
        };

        while timeout.elapsed() < Duration::from_millis(effective_timeout) {
            match transport.read_bytes(&mut buf, 100) {
                Ok(n) if n > 0 => {
                    for &byte in &buf[..n] {
                        let ch = byte as char;
                        if ch == '>' {
                            let clean = Self::clean_response(&response);
                            dev_log::log_rx(cmd, &clean);
                            self.consecutive_errors = 0;
                            return Ok(clean);
                        }
                        response.push(ch);
                    }
                    // Prevent unbounded memory growth from misbehaving adapter
                    if response.len() > max_response_size {
                        dev_log::log_error("obd", &format!("Response exceeded {}KB cap for: {}", max_response_size / 1024, cmd));
                        self.consecutive_errors += 1;
                        return Err(format!("Response too large (>{}KB)", max_response_size / 1024));
                    }
                }
                Ok(_) => {
                    // Transport read timeout (100ms) already provides backoff; no additional sleep needed
                }
                Err(_) => {
                    self.consecutive_errors += 1;
                    return Err("Transport read error".to_string());
                }
            }
        }

        // Timeout — return what we have if it looks useful
        if !response.is_empty() {
            let clean = Self::clean_response(&response);
            // If we got a partial valid response, use it
            if clean.contains("41") || clean.contains("62") || clean.contains("7E") || clean.contains("59") {
                dev_log::log_rx(cmd, &format!("(partial) {}", clean));
                return Ok(clean);
            }
            dev_log::log_rx(cmd, &format!("(timeout) {}", clean));
            Ok(clean)
        } else {
            self.consecutive_errors += 1;
            dev_log::log_error("obd", &format!("Timeout: {} ({}ms)", cmd, timeout_ms));
            Err(format!("Timeout on command: {}", cmd))
        }
    }

    /// Clean ELM327 response — filter noise, errors, control chars
    fn clean_response(raw: &str) -> String {
        let mut result = String::with_capacity(raw.len());
        let mut first = true;
        for line in raw.split(|c| c == '\r' || c == '\n') {
            let l = line.trim();
            if l.is_empty()
                || l.starts_with("SEARCHING")
                || l.starts_with("BUS INIT")
                || l.starts_with("UNABLE TO CONNECT")
                || l.starts_with("CAN ERROR")
                || l.starts_with("FB ERROR")
                || l.starts_with("DATA ERROR")
                || l.starts_with("BUFFER FULL")
                || l.starts_with("STOPPED")
                || l.starts_with("BUS BUSY")
                || l.starts_with("LP ALERT")
                || l.starts_with("LV RESET")
                || l.starts_with("ACT ALERT")
                || l.starts_with("ERR")
                // NOTE: "NO DATA" is NOT filtered — it carries semantic meaning
                // (PID not supported, ECU not responding) that callers check for
                || l.starts_with("TIMEOUT")
                || l == "?"
                // NOTE: "OK" is kept for AT commands (only valid response),
                // but filtered from OBD data responses where it's noise
                // Filter echo residues (AT commands echoed back)
                || l.starts_with("AT")
                || l.starts_with("at")
            {
                continue;
            }
            if !first {
                result.push('\n');
            }
            result.push_str(l);
            first = false;
        }
        result
    }

    /// Flush input buffer (discard any pending data)
    pub(super) fn flush_buffer(&mut self) {
        if let Some(ref mut transport) = self.transport {
            let mut buf = [0u8; 1024];
            // Read and discard all pending data (up to 10 rounds to be sure)
            for _ in 0..10 {
                match transport.read_bytes(&mut buf, 10) {
                    Ok(0) => break,
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_response_filters_searching() {
        let raw = "SEARCHING\n41 0C 10 20\nSEARCHING";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("SEARCHING"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_bus_init() {
        let raw = "BUS INIT\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("BUS INIT"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_can_error() {
        let raw = "CAN ERROR\n41 0C 10 20\nCAN ERROR";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("CAN ERROR"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_buffer_full() {
        let raw = "BUFFER FULL\nDATA ERROR\n41 0C";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("BUFFER FULL"));
        assert!(!cleaned.contains("DATA ERROR"));
        assert!(cleaned.contains("41 0C"));
    }

    #[test]
    fn test_clean_response_filters_at_echo() {
        let raw = "ATE0\nOK\nATE1\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("ATE0"));
        assert!(!cleaned.contains("ATE1"));
        // OK is kept from AT commands but filtered from data responses
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_preserves_no_data() {
        // NO DATA is semantic information — should NOT be filtered
        let raw = "41 0C\nNO DATA\n41 0D";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.contains("NO DATA"));
    }

    #[test]
    fn test_clean_response_removes_empty_lines() {
        let raw = "41 0C 10 20\n\n\n41 0D 05 10\n";
        let cleaned = Elm327Connection::clean_response(raw);
        // Should have two lines, no empties
        let lines: Vec<&str> = cleaned.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| !l.is_empty()));
    }

    #[test]
    fn test_clean_response_normalizes_line_endings() {
        let raw = "41 0C 10 20\r\n41 0D 05 10\r\n";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.contains("41 0C 10 20"));
        assert!(cleaned.contains("41 0D 05 10"));
        assert!(!cleaned.contains("\r"));
    }

    #[test]
    fn test_clean_response_trims_whitespace() {
        let raw = "  41 0C 10 20  \n  41 0D 05 10  ";
        let cleaned = Elm327Connection::clean_response(raw);
        let lines: Vec<&str> = cleaned.lines().collect();
        assert_eq!(lines[0], "41 0C 10 20");
        assert_eq!(lines[1], "41 0D 05 10");
    }

    #[test]
    fn test_clean_response_filters_unable_to_connect() {
        let raw = "UNABLE TO CONNECT\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("UNABLE TO CONNECT"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_bus_busy() {
        let raw = "BUS BUSY\nFB ERROR\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("BUS BUSY"));
        assert!(!cleaned.contains("FB ERROR"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_err() {
        let raw = "ERR\n41 0C 10 20\nERROR";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("ERR"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_mixed_noise() {
        let raw = "SEARCHING\nBUS INIT\n41 0C 10 20\nCAN ERROR\nSEARCHING\n\nNO DATA";
        let cleaned = Elm327Connection::clean_response(raw);

        // Should keep valid data and NO DATA
        assert!(cleaned.contains("41 0C 10 20"));
        assert!(cleaned.contains("NO DATA"));

        // Should remove noise
        assert!(!cleaned.contains("SEARCHING"));
        assert!(!cleaned.contains("BUS INIT"));
        assert!(!cleaned.contains("CAN ERROR"));
    }

    #[test]
    fn test_clean_response_empty_input() {
        let raw = "";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_response_only_noise() {
        let raw = "SEARCHING\nBUS INIT\nCAN ERROR";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_response_question_mark_filtered() {
        let raw = "?\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("?"));
        assert!(cleaned.contains("41 0C 10 20"));
    }
}
