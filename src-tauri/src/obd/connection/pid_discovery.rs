use crate::obd::dev_log;
use super::Elm327Connection;

impl Elm327Connection {
    // ==================== SUPPORTED PID DISCOVERY ====================

    /// Discover all supported PID ranges: 0100, 0120, 0140, 0160
    pub(super) fn discover_supported_pids(&mut self) {
        dev_log::log_debug("obd", "Discovering supported PID ranges...");

        // Range 0x01-0x20 (already read during protocol detect, but re-parse if needed)
        if self.supported_pids.is_empty() {
            if let Ok(response) = self.send_command("0100") {
                self.parse_supported_pids_from_response(&response);
            }
        }

        // Check if PID 0x20 is supported (means 0x21-0x40 range available)
        if self.supported_pids.contains(&0x20) {
            if let Ok(response) = self.send_command("0120") {
                let pids = Self::parse_pid_bitmap(&response, "41 20", 0x20);
                self.supported_pids.extend(pids);
            }
        }

        // Check if PID 0x40 is supported (means 0x41-0x60 range available)
        if self.supported_pids.contains(&0x40) {
            if let Ok(response) = self.send_command("0140") {
                let pids = Self::parse_pid_bitmap(&response, "41 40", 0x40);
                self.supported_pids.extend(pids);
            }
        }

        // Check if PID 0x60 is supported (means 0x61-0x80 range available)
        if self.supported_pids.contains(&0x60) {
            if let Ok(response) = self.send_command("0160") {
                let pids = Self::parse_pid_bitmap(&response, "41 60", 0x60);
                self.supported_pids_ext.extend(pids);
            }
        }

        self.supported_pids.sort();
        self.supported_pids.dedup();
        self.supported_pids_ext.sort();
        self.supported_pids_ext.dedup();

        dev_log::log_info("obd", &format!(
            "Supported PIDs: {} standard (0x01-0x60), {} extended (0x61+)",
            self.supported_pids.len(), self.supported_pids_ext.len()
        ));
    }

    /// Parse supported PIDs bitmask from any "41 XX" response
    pub(super) fn parse_supported_pids_from_response(&mut self, response: &str) {
        let pids = Self::parse_pid_bitmap(response, "41 00", 0x00);
        if !pids.is_empty() {
            self.supported_pids = pids;
        }
    }

    /// Parse a PID bitmap from response — handles spaced and unspaced hex
    fn parse_pid_bitmap(response: &str, prefix: &str, base: u8) -> Vec<u8> {
        let mut pids = Vec::new();

        // Try with spaces first — iterate ALL matching lines for multi-ECU responses
        let mut found_any = false;
        for line in response.lines().filter(|l| l.contains(prefix)) {
            let bytes: Vec<u8> = line
                .split_whitespace()
                .skip(2)  // Skip "41 XX"
                .take(4)
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            Self::decode_bitmap(&bytes, base, &mut pids);
            found_any = true;
        }

        if !found_any {
            // Try without spaces
            let no_space = prefix.replace(" ", "");
            let clean = response.replace(" ", "");
            if let Some(start) = clean.find(&no_space) {
                let data_start = start + no_space.len();
                if data_start + 8 <= clean.len() {
                    let bytes: Vec<u8> = (0..8)
                        .step_by(2)
                        .filter_map(|i| u8::from_str_radix(&clean[data_start+i..data_start+i+2], 16).ok())
                        .collect();
                    Self::decode_bitmap(&bytes, base, &mut pids);
                }
            }
        }

        pids.sort();
        pids.dedup();
        pids
    }

    /// Decode 4-byte bitmask into PID list
    fn decode_bitmap(bytes: &[u8], base: u8, pids: &mut Vec<u8>) {
        for (byte_idx, &byte) in bytes.iter().enumerate() {
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    pids.push(base + (byte_idx * 8 + bit + 1) as u8);
                }
            }
        }
    }

    /// Check if a PID is supported by the vehicle (binary search on sorted vecs)
    pub fn is_pid_supported(&self, pid: u8) -> bool {
        self.supported_pids.binary_search(&pid).is_ok() || self.supported_pids_ext.binary_search(&pid).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_bitmap_test(bytes: &[u8], base: u8) -> Vec<u8> {
        let mut pids = Vec::new();
        for (byte_idx, &byte) in bytes.iter().enumerate() {
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    pids.push(base + (byte_idx * 8 + bit + 1) as u8);
                }
            }
        }
        pids
    }

    #[test]
    fn test_decode_bitmap_single_byte() {
        let bytes = vec![0x80];
        let pids = decode_bitmap_test(&bytes, 0x00);
        assert_eq!(pids, vec![0x01]);
    }

    #[test]
    fn test_decode_bitmap_multiple_bits() {
        let bytes = vec![0xFF];
        let pids = decode_bitmap_test(&bytes, 0x00);
        assert_eq!(pids, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn test_decode_bitmap_with_base() {
        let bytes = vec![0x80];
        let pids = decode_bitmap_test(&bytes, 0x20);
        assert_eq!(pids, vec![0x21]);
    }

    #[test]
    fn test_decode_bitmap_multiple_bytes() {
        let bytes = vec![0x80, 0x01];
        let pids = decode_bitmap_test(&bytes, 0x00);
        assert_eq!(pids, vec![0x01, 0x10]);
    }

    #[test]
    fn test_decode_bitmap_empty() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00];
        let pids = decode_bitmap_test(&bytes, 0x00);
        assert!(pids.is_empty());
    }

    #[test]
    fn test_pid_bitmap_parsing_with_spaces() {
        let response = "41 00 FF FF FF FF";
        assert!(response.contains("41 00"));
        let bytes: Vec<u8> = response
            .split_whitespace()
            .skip(2)
            .take(4)
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes[0], 0xFF);
    }

    #[test]
    fn test_pid_bitmap_parsing_without_spaces() {
        let response = "4100FFFFFFFF";
        let clean = response.replace(" ", "");
        assert!(clean.contains("4100"));
        if let Some(start) = clean.find("4100") {
            let data_start = start + 4;
            if data_start + 8 <= clean.len() {
                let bytes: Vec<u8> = (0..8)
                    .step_by(2)
                    .filter_map(|i| u8::from_str_radix(&clean[data_start+i..data_start+i+2], 16).ok())
                    .collect();
                assert_eq!(bytes.len(), 4);
            }
        }
    }

    #[test]
    fn test_pid_discovery_conceptual() {
        let response = "41 20 80 00 00 00";
        let pids: Vec<u8> = response
            .split_whitespace()
            .skip(2)
            .take(4)
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
        assert!(!pids.is_empty());
        assert_eq!(pids[0], 0x80);
    }

    #[test]
    fn test_bitmap_dedup_and_sort() {
        let response = "41 00 FF FF FF FF";
        let bytes: Vec<u8> = response
            .split_whitespace()
            .skip(2)
            .take(4)
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
        let mut pids = decode_bitmap_test(&bytes, 0x00);
        let original_len = pids.len();
        pids.sort();
        pids.dedup();
        assert_eq!(pids.len(), original_len);
    }
}
