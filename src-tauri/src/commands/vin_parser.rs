use crate::obd::dev_log;

/// Strategy 1: Standard multi-frame "49 02 XX <data>"
pub(crate) fn parse_vin_multiframe(response: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for line in response.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 3 && parts[0] == "49" && parts[1] == "02" {
            // Skip "49 02 XX" (service + PID + message counter)
            bytes.extend(
                parts[3..].iter()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
            );
        }
    }
    bytes
}

/// Strategy 2: Single-line response without frame counter "49 02 <all data>"
pub(crate) fn parse_vin_singleline(response: &str) -> Vec<u8> {
    let all_parts: Vec<&str> = response.split_whitespace().collect();
    if all_parts.len() > 2 && all_parts[0] == "49" && all_parts[1] == "02" {
        return all_parts[2..].iter()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
    }
    Vec::new()
}

/// Strategy 3: No-space hex "4902XXXXXXXXXXXX..."
pub(crate) fn parse_vin_nospace(response: &str) -> Vec<u8> {
    let hex_str = response.replace(" ", "").replace("\n", "");
    if let Some(start) = hex_str.find("4902") {
        let data_start = start + 4;
        // Skip message counter byte (2 chars)
        let data_start = if data_start + 2 <= hex_str.len() { data_start + 2 } else { data_start };
        return (data_start..hex_str.len()).step_by(2)
            .filter_map(|i| {
                if i + 2 <= hex_str.len() {
                    u8::from_str_radix(&hex_str[i..i+2], 16).ok()
                } else {
                    None
                }
            })
            .collect();
    }
    Vec::new()
}

/// Strategy 4: CAN multi-frame with headers "7E8 10 14 49 02 01 XX XX XX..."
pub(crate) fn parse_vin_can_header(response: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for line in response.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Look for "49 02" anywhere in the line (skip header bytes)
        if let Some(pos) = parts.iter().position(|&p| p == "49") {
            if pos + 1 < parts.len() && parts[pos + 1] == "02" {
                let data_start = pos + 3; // Skip "49 02 XX"
                if data_start < parts.len() {
                    bytes.extend(
                        parts[data_start..].iter()
                            .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    );
                }
            }
        }
    }
    bytes
}

/// Strategy 5: Raw fallback — try to parse all hex bytes and find ASCII VIN pattern
pub(crate) fn parse_vin_raw_fallback(response: &str) -> Vec<u8> {
    response
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect()
}

/// Convert raw bytes to a validated 17-char VIN string
pub(crate) fn validate_vin(bytes: Vec<u8>) -> String {
    // Convert to ASCII, filter to alphanumeric only
    let vin: String = String::from_utf8(bytes)
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    // VIN must be exactly 17 characters
    if vin.len() == 17 {
        vin
    } else if vin.len() > 17 {
        // Some adapters pad with extra bytes — try to extract 17-char VIN
        // VIN never contains I, O, Q
        for start in 0..=(vin.len() - 17) {
            let candidate = &vin[start..start + 17];
            if candidate.chars().all(|c| c.is_ascii_alphanumeric() && c != 'I' && c != 'O' && c != 'Q') {
                return candidate.to_string();
            }
        }
        String::new()
    } else {
        dev_log::log_warn("vin_parser", &format!("VIN parse: got {} chars instead of 17: {}", vin.len(), vin));
        String::new()
    }
}

/// Parse VIN from Mode 09 response (multi-frame compatible, handles many adapter formats)
pub(crate) fn parse_vin_response(response: &str) -> String {
    if response.is_empty() || response.contains("NO DATA") || response.contains("ERROR") {
        return String::new();
    }
    let strategies: &[fn(&str) -> Vec<u8>] = &[
        parse_vin_multiframe,
        parse_vin_singleline,
        parse_vin_nospace,
        parse_vin_can_header,
        parse_vin_raw_fallback,
    ];
    for strategy in strategies {
        let bytes = strategy(response);
        if !bytes.is_empty() {
            return validate_vin(bytes);
        }
    }
    String::new()
}

/// Check if VIN is valid (format only, advisory check digit validation)
pub(crate) fn is_valid_vin(vin: &str) -> bool {
    // Validate: exactly 17 alphanumeric chars, no I/O/Q
    if vin.len() != 17 {
        return false;
    }
    if !vin.chars().all(|c| c.is_ascii_alphanumeric() && c != 'I' && c != 'O' && c != 'Q') {
        return false;
    }

    // ISO 3779 check digit validation (position 9, advisory only)
    fn transliterate(c: char) -> u32 {
        match c {
            'A'|'J' => 1, 'B'|'K'|'S' => 2, 'C'|'L'|'T' => 3, 'D'|'M'|'U' => 4,
            'E'|'N'|'V' => 5, 'F'|'W' => 6, 'G'|'P'|'X' => 7, 'H'|'Y' => 8,
            'R'|'Z' => 9,
            d if d.is_ascii_digit() => d.to_digit(10).unwrap_or(0),
            _ => 0,
        }
    }
    let weights: [u32; 17] = [8, 7, 6, 5, 4, 3, 2, 10, 0, 9, 8, 7, 6, 5, 4, 3, 2];
    let chars: Vec<char> = vin.chars().collect();
    let sum: u32 = chars.iter().enumerate().map(|(i, &c)| transliterate(c) * weights[i]).sum();
    let check = sum % 11;
    let expected = if check == 10 { 'X' } else { char::from_digit(check, 10).unwrap_or('0') };
    if chars[8] != expected {
        dev_log::log_warn("vin_parser", &format!("VIN check digit mismatch: expected '{}', got '{}'", expected, chars[8]));
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vin_multiframe() {
        // Each line: "49 02 XX" + 3 hex data bytes → 9 bytes total
        let response = "49 02 01 56 46 33\n49 02 02 4C 43 42\n49 02 03 48 5A 36";
        let bytes = parse_vin_multiframe(response);
        assert_eq!(bytes.len(), 9);
        assert_eq!(bytes[0], 0x56); // 'V'
        assert_eq!(bytes[1], 0x46); // 'F'
        assert_eq!(bytes[2], 0x33); // '3'
    }

    #[test]
    fn test_parse_vin_multiframe_empty() {
        let response = "";
        let bytes = parse_vin_multiframe(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_multiframe_no_49_02_header() {
        let response = "48 02 01 56 46 33\n48 02 02 4C 43 42";
        let bytes = parse_vin_multiframe(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_multiframe_mixed_valid_invalid() {
        let response = "49 02 01 56 46 33\n48 02 02 4C 43 42\n49 02 03 48 5A 36";
        let bytes = parse_vin_multiframe(response);
        // Should collect bytes from lines with "49 02" header only
        assert_eq!(bytes.len(), 6);
    }

    #[test]
    fn test_parse_vin_singleline() {
        let response = "49 02 56 46 33 4C 43 42 48 5A 36 4A 53 30 30 30 30 30 30";
        let bytes = parse_vin_singleline(response);
        assert_eq!(bytes.len(), 17);
        assert_eq!(bytes[0], 0x56); // 'V'
    }

    #[test]
    fn test_parse_vin_singleline_empty() {
        let response = "";
        let bytes = parse_vin_singleline(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_singleline_no_49_02() {
        let response = "49 01 56 46 33 4C 43 42";
        let bytes = parse_vin_singleline(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_singleline_exactly_49_02() {
        let response = "49 02";
        let bytes = parse_vin_singleline(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_nospace() {
        let response = "4902564633344C434248355A364A533030303030303030";
        let bytes = parse_vin_nospace(response);
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_parse_vin_nospace_with_newlines() {
        let response = "4902564633\n344C43424\n8355A364A5330303030303030";
        let bytes = parse_vin_nospace(response);
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_parse_vin_nospace_no_4902() {
        let response = "4903564633344C434248355A364A533030303030303030";
        let bytes = parse_vin_nospace(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_nospace_empty() {
        let response = "";
        let bytes = parse_vin_nospace(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_can_header() {
        let response = "7E8 10 14 49 02 01 56 46 33\n7E9 21 4C 43 42 48 5A 36 4A";
        let bytes = parse_vin_can_header(response);
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_parse_vin_can_header_no_49_02() {
        let response = "7E8 10 14 48 02 01 56 46 33";
        let bytes = parse_vin_can_header(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_can_header_empty() {
        let response = "";
        let bytes = parse_vin_can_header(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_can_header_49_without_02_next() {
        let response = "7E8 10 14 49 01 01 56 46 33";
        let bytes = parse_vin_can_header(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_validate_vin_valid_17_chars() {
        let bytes = "VF3LCBHZ6JS000000".as_bytes().to_vec();
        let vin = validate_vin(bytes);
        assert_eq!(vin, "VF3LCBHZ6JS000000");
    }

    #[test]
    fn test_validate_vin_invalid_length_short() {
        let bytes = "VF3LCBHZ6JS00".as_bytes().to_vec();
        let vin = validate_vin(bytes);
        assert_eq!(vin, "");
    }

    #[test]
    fn test_validate_vin_invalid_length_long() {
        let bytes = "VF3LCBHZ6JS000000EXTRA".as_bytes().to_vec();
        let vin = validate_vin(bytes);
        // Should try to extract 17-char substring without I/O/Q
        assert!(vin.is_empty() || vin.len() == 17);
    }

    #[test]
    fn test_validate_vin_with_ioq_padded() {
        let bytes = "VF3LCBHZ6JSIOQQQQ".as_bytes().to_vec();
        let vin = validate_vin(bytes);
        // 17 chars exactly → returned as-is (validate_vin doesn't filter I/O/Q at len==17)
        assert_eq!(vin.len(), 17);
    }

    #[test]
    fn test_validate_vin_with_nonascii() {
        let bytes = vec![0x56, 0x46, 0x33, 0xFF, 0x4C, 0x43, 0x42, 0x48, 0x5A, 0x36, 0x4A, 0x53, 0x30, 0x30, 0x30, 0x30, 0x30];
        let vin = validate_vin(bytes);
        // Non-ASCII bytes are filtered out by from_utf8, resulting in < 17 chars
        assert!(vin.is_empty() || vin.len() < 17);
    }

    #[test]
    fn test_validate_vin_numeric_17_chars() {
        let bytes = "12345678901234567".as_bytes().to_vec();
        let vin = validate_vin(bytes);
        assert_eq!(vin, "12345678901234567");
    }

    #[test]
    fn test_validate_vin_mixed_alphanumeric() {
        let bytes = "WBADT434GG296970A".as_bytes().to_vec();
        let vin = validate_vin(bytes);
        assert_eq!(vin.len(), 17);
    }

    #[test]
    fn test_parse_vin_raw_fallback() {
        let response = "56 46 33 4C 43 42 48 5A 36 4A 53 30 30 30 30 30 30";
        let bytes = parse_vin_raw_fallback(response);
        assert_eq!(bytes.len(), 17);
        assert_eq!(bytes[0], 0x56); // 'V'
    }

    #[test]
    fn test_parse_vin_raw_fallback_empty() {
        let response = "";
        let bytes = parse_vin_raw_fallback(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_raw_fallback_invalid_hex() {
        let response = "ZZ GG HH";
        let bytes = parse_vin_raw_fallback(response);
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn test_parse_vin_response_multiframe_strategy() {
        let response = "49 02 01 56 46 33\n49 02 02 4C 43 42\n49 02 03 48 5A 36 4A 53 30 30 30 30 30 30";
        let vin = parse_vin_response(response);
        assert_eq!(vin.len(), 17);
    }

    #[test]
    fn test_parse_vin_singleline_direct() {
        let response = "49 02 56 46 33 4C 43 42 48 5A 36 4A 53 30 30 30 30 30 30";
        let bytes = parse_vin_singleline(response);
        assert_eq!(bytes.len(), 17);
        let vin = String::from_utf8(bytes).unwrap();
        assert_eq!(vin, "VF3LCBHZ6JS000000");
    }

    #[test]
    fn test_parse_vin_response_empty() {
        let response = "";
        let vin = parse_vin_response(response);
        assert_eq!(vin, "");
    }

    #[test]
    fn test_parse_vin_response_no_data() {
        let response = "NO DATA";
        let vin = parse_vin_response(response);
        assert_eq!(vin, "");
    }

    #[test]
    fn test_parse_vin_response_error() {
        let response = "ERROR";
        let vin = parse_vin_response(response);
        assert_eq!(vin, "");
    }

    #[test]
    fn test_is_valid_vin_valid_format() {
        let vin = "VF3LCBHZ6JS000001";
        assert!(is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_with_numbers() {
        let vin = "12345678901234567";
        assert!(is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_too_short() {
        let vin = "ABC";
        assert!(!is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_too_long() {
        let vin = "VF3LCBHZ6JS0000011";
        assert!(!is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_contains_i() {
        let vin = "WBADT434IG296970A";
        assert!(!is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_contains_o() {
        let vin = "WBADT434OG296970A";
        assert!(!is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_contains_q() {
        let vin = "WBADT434QG296970A";
        assert!(!is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_special_chars() {
        let vin = "WBADT434@G296970A";
        assert!(!is_valid_vin(vin));
    }

    #[test]
    fn test_is_valid_vin_lowercase() {
        // is_valid_vin checks is_ascii_alphanumeric which includes lowercase
        let vin = "wbadt434gg296970a";
        assert!(is_valid_vin(vin));
    }
}
