use super::Elm327Connection;
use crate::obd::dev_log;
use std::time::Duration;
use tracing::debug;

impl Elm327Connection {
    pub(crate) fn parse_hex_response_line(line: &str) -> (Option<String>, Vec<u8>) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            return (None, Vec::new());
        }

        let mut header = None;
        let mut bytes = Vec::new();
        let mut start = 0;
        if matches!(tokens[0].len(), 3 | 8)
            && tokens[0]
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            header = Some(tokens[0].to_ascii_uppercase());
            start = 1;
        }

        for token in &tokens[start..] {
            let hex: String = token
                .chars()
                .filter(|character| character.is_ascii_hexdigit())
                .collect();
            if hex.len() < 2 || hex.len() % 2 != 0 {
                continue;
            }
            bytes.extend(
                (0..hex.len())
                    .step_by(2)
                    .filter_map(|index| u8::from_str_radix(&hex[index..index + 2], 16).ok()),
            );
        }

        // Compact CAN responses may arrive as `7E8101462...` without spaces.
        if tokens.len() == 1 && tokens[0].len() >= 5 && tokens[0].len() % 2 == 1 {
            let compact = tokens[0];
            if compact[..3]
                .chars()
                .all(|character| character.is_ascii_hexdigit())
            {
                header = Some(compact[..3].to_ascii_uppercase());
                bytes.clear();
                let payload = &compact[3..];
                bytes.extend(
                    (0..payload.len()).step_by(2).filter_map(|index| {
                        u8::from_str_radix(&payload[index..index + 2], 16).ok()
                    }),
                );
            }
        }

        (header, bytes)
    }

    fn parse_framed_response(response: &str, marker: &[u8]) -> Option<Vec<u8>> {
        let mut selected_header = None;
        let mut expected_payload_length = None;
        let mut expected_sequence = 1_u8;
        let mut data = Vec::new();
        let mut started = false;

        for line in response.lines() {
            let (header, bytes) = Self::parse_hex_response_line(line);
            if bytes.is_empty() {
                continue;
            }

            if !started {
                let Some(marker_position) = bytes
                    .windows(marker.len())
                    .position(|window| window == marker)
                else {
                    continue;
                };
                selected_header = header;
                if let Some(first_frame_position) = bytes[..marker_position]
                    .iter()
                    .rposition(|byte| byte & 0xF0 == 0x10)
                {
                    if let Some(length_low) = bytes.get(first_frame_position + 1) {
                        expected_payload_length = Some(
                            (usize::from(bytes[first_frame_position] & 0x0F) << 8)
                                | usize::from(*length_low),
                        );
                    }
                } else if marker_position > 0 {
                    let single_frame_pci = bytes[marker_position - 1];
                    if single_frame_pci & 0xF0 == 0 {
                        expected_payload_length = Some(usize::from(single_frame_pci & 0x0F));
                    }
                }
                data.extend_from_slice(&bytes[marker_position + marker.len()..]);
                started = true;
            } else {
                if selected_header.is_some() && header != selected_header {
                    continue;
                }
                let Some(pci_position) = bytes
                    .iter()
                    .position(|byte| byte & 0xF0 == 0x20 && byte & 0x0F == expected_sequence)
                else {
                    continue;
                };
                data.extend_from_slice(&bytes[pci_position + 1..]);
                expected_sequence = (expected_sequence + 1) & 0x0F;
            }

            if let Some(total_payload_length) = expected_payload_length {
                let expected_data_length = total_payload_length.saturating_sub(marker.len());
                if data.len() >= expected_data_length {
                    data.truncate(expected_data_length);
                    break;
                }
            }
        }

        started
            .then_some(data)
            .filter(|payload| !payload.is_empty())
    }

    pub(crate) fn parse_did_response(response: &str, did: u16) -> Option<Vec<u8>> {
        let marker = [0x62, (did >> 8) as u8, did as u8];
        Self::parse_framed_response(response, &marker)
    }

    fn contains_negative_response(response: &str, requested_service: u8) -> bool {
        response.lines().any(|line| {
            let (_, bytes) = Self::parse_hex_response_line(line);
            bytes
                .windows(3)
                .any(|window| window[0] == 0x7F && window[1] == requested_service)
        })
    }

    fn parse_pid_response(response: &str, mode: u8, pid: u8) -> Option<Vec<u8>> {
        Self::parse_framed_response(response, &[mode + 0x40, pid])
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
                        debug!(
                            "PID {:02X} attempt {} failed: {}, retrying...",
                            pid,
                            attempt + 1,
                            e
                        );
                        // Escalating recovery
                        if attempt == 0 {
                            std::thread::sleep(Duration::from_millis(100));
                        } else {
                            // On second retry, try recovery sequence
                            dev_log::log_warn(
                                "obd",
                                &format!("PID {:02X} retry with recovery", pid),
                            );
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
            if Self::contains_negative_response(&response, mode) {
                // 7F XX 31 = serviceNotSupported, 7F XX 12 = subFunctionNotSupported
                return Err(format!("PID {:02X} negative response: {}", pid, response));
            }

            // Parse the first complete positive response without consuming CAN
            // header/PCI bytes or data from another ECU line.
            let matching_lines: Vec<&str> = response
                .lines()
                .filter(|l| l.contains(&expected_prefix))
                .collect();
            if matching_lines.len() > 1 {
                dev_log::log_debug(
                    "obd",
                    &format!(
                        "PID {:02X}: {} ECUs responded (using first)",
                        pid,
                        matching_lines.len()
                    ),
                );
            }
            if let Some(bytes) = Self::parse_pid_response(&response, mode, pid) {
                return Ok(bytes);
            }

            if attempt < 2 {
                debug!("PID {:02X} parse failed, retrying...", pid);
                std::thread::sleep(Duration::from_millis(50 * (attempt as u64 + 1)));
            }
        }

        Err(format!(
            "Invalid response for PID {:02X} after 3 attempts",
            pid
        ))
    }

    /// Query a UDS DID (Service 0x22) with multi-frame support
    pub fn query_did(&mut self, did: u16) -> Result<Vec<u8>, String> {
        let cmd = format!("22{:04X}", did);
        let response = self.send_command_timeout(&cmd, 3000)?;

        if response.contains("NO DATA") {
            return Err(format!("DID {:04X} not supported", did));
        }

        if Self::contains_negative_response(&response, 0x22) {
            return Err(format!("DID {:04X} negative response", did));
        }

        if let Some(bytes) = Self::parse_did_response(&response, did) {
            return Ok(bytes);
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

    #[test]
    fn assembles_multiframe_mode09_response() {
        let response = concat!(
            "7E8 10 0C 49 04 01 56 46\n",
            "7E9 05 49 04 01 42 41\n",
            "7E8 21 33 4C 43 42 48 5A 36"
        );
        assert_eq!(
            Elm327Connection::parse_pid_response(response, 0x09, 0x04),
            Some(b"\x01VF3LCBHZ6".to_vec())
        );
    }

    #[test]
    fn reassembles_headered_isotp_did_response() {
        let response = concat!(
            "7E8 10 0C 62 F1 90 56 46\n",
            "7E9 21 FF FF FF FF FF FF FF\n",
            "7E8 21 33 4C 43 42 48 5A 36\n",
            "7E8 22 4A 53 30 30 30 30 30"
        );
        assert_eq!(
            Elm327Connection::parse_did_response(response, 0xF190),
            Some(b"VF3LCBHZ6".to_vec())
        );
    }

    #[test]
    fn payload_byte_7f_is_not_a_negative_response() {
        assert!(!Elm327Connection::contains_negative_response(
            "7E8 05 62 20 60 7F 01",
            0x22
        ));
        assert!(Elm327Connection::contains_negative_response(
            "7E8 03 7F 22 31",
            0x22
        ));
    }
}
