use std::time::Duration;
use tracing::debug;
use crate::obd::dev_log;
use super::Elm327Connection;

impl Elm327Connection {
    fn parse_pid_response(response: &str, mode: u8, pid: u8) -> Option<Vec<u8>> {
        let response_service = mode + 0x40;
        for line in response.lines() {
            let bytes: Vec<u8> = line
                .split_whitespace()
                .filter_map(|token| u8::from_str_radix(token, 16).ok())
                .collect();
            if let Some(position) = bytes
                .windows(2)
                .position(|window| window == [response_service, pid])
            {
                let data = bytes[position + 2..].to_vec();
                if !data.is_empty() {
                    return Some(data);
                }
            }
        }

        let prefix = format!("{response_service:02X}{pid:02X}");
        let compact: String = response
            .chars()
            .filter(|character| character.is_ascii_hexdigit())
            .collect();
        let start = compact.find(&prefix)? + prefix.len();
        let remaining = &compact[start..];
        let usable_length = remaining.len().min(8) & !1;
        let data: Vec<u8> = (0..usable_length)
            .step_by(2)
            .filter_map(|index| u8::from_str_radix(&remaining[index..index + 2], 16).ok())
            .collect();
        (!data.is_empty()).then_some(data)
    }

    // ==================== PID QUERY WITH RESILIENCE ====================

    /// Send OBD-II PID request with retry logic and error recovery
    pub fn query_pid(&mut self, mode: u8, pid: u8) -> Result<Vec<u8>, String> {
        let cmd = format!("{:02X}{:02X}", mode, pid);
        let expected_prefix = format!("{:02X} {:02X}", mode + 0x40, pid);

        // If we know supported PIDs and this one isn't listed, skip it
        if mode == 0x01 && !self.supported_pids.is_empty() && !self.is_pid_supported(pid) {
            return Err(format!("PID {:02X} not in supported list", pid));
        }

        // Try up to 3 times with escalating recovery
        for attempt in 0..3 {
            let response = match self.send_command(&cmd) {
                Ok(r) => r,
                Err(e) => {
                    if attempt < 2 {
                        debug!("PID {:02X} attempt {} failed: {}, retrying...", pid, attempt + 1, e);
                        // Escalating recovery
                        if attempt == 0 {
                            std::thread::sleep(Duration::from_millis(100));
                        } else {
                            // On second retry, try recovery sequence
                            dev_log::log_warn("obd", &format!("PID {:02X} retry with recovery", pid));
                            let _ = self.send_command("3E00"); // TesterPresent wake-up
                            std::thread::sleep(Duration::from_millis(200));
                        }
                        continue;
                    }
                    return Err(e);
                }
            };

            // Check for "NO DATA" — PID not supported by this ECU
            if response.contains("NO DATA") {
                return Err(format!("PID {:02X} not supported (NO DATA)", pid));
            }

            // Check for negative response (7F = error)
            if response.contains("7F") {
                // 7F XX 31 = serviceNotSupported, 7F XX 12 = subFunctionNotSupported
                return Err(format!("PID {:02X} negative response: {}", pid, response));
            }

            // Parse the first complete positive response without consuming CAN
            // header/PCI bytes or data from another ECU line.
            let matching_lines: Vec<&str> = response.lines().filter(|l| l.contains(&expected_prefix)).collect();
            if matching_lines.len() > 1 {
                dev_log::log_debug("obd", &format!(
                    "PID {:02X}: {} ECUs responded (using first)", pid, matching_lines.len()
                ));
            }
            if let Some(bytes) = Self::parse_pid_response(&response, mode, pid) {
                return Ok(bytes);
            }

            if attempt < 2 {
                debug!("PID {:02X} parse failed, retrying...", pid);
                std::thread::sleep(Duration::from_millis(50 * (attempt as u64 + 1)));
            }
        }

        Err(format!("Invalid response for PID {:02X} after 3 attempts", pid))
    }

    /// Query a UDS DID (Service 0x22) with multi-frame support
    pub fn query_did(&mut self, did: u16) -> Result<Vec<u8>, String> {
        let cmd = format!("22{:04X}", did);
        let response = self.send_command_timeout(&cmd, 3000)?;

        if response.contains("NO DATA") {
            return Err(format!("DID {:04X} not supported", did));
        }

        if response.contains("7F") {
            return Err(format!("DID {:04X} negative response", did));
        }

        // Parse with spaces — match "62" as UDS positive response token (not as data byte)
        let did_high = format!("{:02X}", (did >> 8) & 0xFF);
        let did_low = format!("{:02X}", did & 0xFF);
        if let Some(line) = response.lines().find(|l| {
            let tokens: Vec<&str> = l.split_whitespace().collect();
            tokens.len() >= 3 && tokens.iter().position(|t| *t == "62")
                .map(|p| p + 2 < tokens.len() && tokens[p+1] == did_high && tokens[p+2] == did_low)
                .unwrap_or(false)
        }) {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let pos = match tokens.iter().position(|t| *t == "62") {
                Some(p) => p,
                None => return Err("UDS response token '62' not found".into()),
            };
            let bytes: Vec<u8> = tokens[pos+3..]
                .iter()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            if !bytes.is_empty() {
                return Ok(bytes);
            }
        }

        // Parse without spaces
        let no_space_prefix = format!("62{:04X}", did);
        let hex_str = response.replace(" ", "").replace("\n", "");
        if let Some(start) = hex_str.find(&no_space_prefix) {
            let data_start = start + no_space_prefix.len();
            if data_start < hex_str.len() {
                let data_hex = &hex_str[data_start..];
                let bytes: Vec<u8> = (0..data_hex.len())
                    .step_by(2)
                    .filter_map(|i| {
                        if i + 2 <= data_hex.len() {
                            u8::from_str_radix(&data_hex[i..i+2], 16).ok()
                        } else {
                            None
                        }
                    })
                    .collect();
                if !bytes.is_empty() {
                    return Ok(bytes);
                }
            }
        }

        Err(format!("Invalid response for DID {:04X}", did))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pid_data_after_positive_response_not_after_can_header() {
        assert_eq!(
            Elm327Connection::parse_pid_response("7E8 04 41 0C 1A F8", 0x01, 0x0C),
            Some(vec![0x1A, 0xF8]),
        );
    }

    #[test]
    fn does_not_mix_second_ecu_response_into_pid_data() {
        assert_eq!(
            Elm327Connection::parse_pid_response(
                "7E8 04 41 0C 1A F8\n7E9 04 41 0C 00 00",
                0x01,
                0x0C,
            ),
            Some(vec![0x1A, 0xF8]),
        );
    }
}
