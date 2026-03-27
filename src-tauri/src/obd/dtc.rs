use std::collections::HashMap;
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};

use crate::models::{DtcCode, DtcStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairTips {
    pub causes: Option<HashMap<String, Vec<String>>>,
    pub quick_check: Option<HashMap<String, String>>,
    pub difficulty: Option<u32>,
}

static DTC_DESCRIPTIONS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/dtc_descriptions.json"))
        .unwrap_or_default()
});

static DTC_DESCRIPTIONS_FR: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/dtc_descriptions_fr.json"))
        .unwrap_or_default()
});

static DTC_REPAIR_TIPS: LazyLock<HashMap<String, RepairTips>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/dtc_repair_tips.json"))
        .unwrap_or_default()
});

/// Decode DTC from 2-byte raw data
pub fn decode_dtc_bytes(b1: u8, b2: u8) -> String {
    let prefix = match (b1 >> 6) & 0x03 {
        0 => 'P',
        1 => 'C',
        2 => 'B',
        3 => 'U',
        _ => 'P',
    };
    let digit1 = (b1 >> 4) & 0x03;
    let digit2 = b1 & 0x0F;
    format!("{}{}{:X}{:02X}", prefix, digit1, digit2, b2)
}

/// Parse DTC response from Mode 03/07/0A
pub fn parse_dtc_response(response: &str, status: DtcStatus, source: &str, lang: &str) -> Vec<DtcCode> {
    let mut dtcs = Vec::new();

    // Response format: "43 01 23 04 56 00 00" or CAN format with count byte
    let bytes: Vec<u8> = response
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if bytes.len() < 2 {
        return dtcs;
    }

    // Skip first byte (mode + 0x40)
    let after_header = &bytes[1..];

    // Detect and skip CAN count byte if present
    // For CAN: count byte at position 0, then DTC pairs
    // Validation: potential_count > 0 && potential_count * 2 + 1 == after_header.len()
    let data = if !after_header.is_empty() {
        let potential_count = after_header[0] as usize;
        if potential_count > 0 && potential_count * 2 + 1 == after_header.len() {
            // Valid CAN format with count byte
            &after_header[1..]
        } else {
            // Standard OBD format (no count byte)
            after_header
        }
    } else {
        after_header
    };

    for chunk in data.chunks(2) {
        if chunk.len() == 2 && (chunk[0] != 0 || chunk[1] != 0) {
            let code = decode_dtc_bytes(chunk[0], chunk[1]);
            let description = get_dtc_description(&code, lang);
            let repair_tips = get_dtc_repair_tips(&code, lang);
            let (causes, quick_check, difficulty) = get_dtc_repair_data(&code, lang);

            dtcs.push(DtcCode {
                code,
                description,
                status: status.clone(),
                source: source.to_string(),
                repair_tips,
                causes,
                quick_check,
                difficulty,
                ecu_context: None,
            });
        }
    }

    dtcs
}

/// Get DTC description from embedded JSON database (bilingual: FR file + EN fallback)
pub fn get_dtc_description(code: &str, lang: &str) -> String {
    if lang == "fr" {
        if let Some(desc) = DTC_DESCRIPTIONS_FR.get(code) {
            return desc.clone();
        }
    }
    DTC_DESCRIPTIONS
        .get(code)
        .cloned()
        .unwrap_or_else(|| {
            if lang == "fr" {
                format!("Code {} — description non disponible", code)
            } else {
                format!("Code {} — description not available", code)
            }
        })
}

/// Get repair tips for DTC from embedded JSON database (language-aware)
pub fn get_dtc_repair_tips(code: &str, lang: &str) -> Option<String> {
    get_dtc_repair_tips_lang(code, lang)
}

/// Get repair tips in a specific language ("fr" or "en")
pub fn get_dtc_repair_tips_lang(code: &str, lang: &str) -> Option<String> {
    DTC_REPAIR_TIPS.get(code).map(|tips| {
        let mut result = String::new();
        let causes_label = if lang == "fr" { "Causes possibles:" } else { "Possible causes:" };
        let check_label = if lang == "fr" { "Vérification rapide: " } else { "Quick check: " };

        if let Some(causes) = &tips.causes {
            // Try requested language, fallback to other
            let lang_causes = causes.get(lang).or_else(|| causes.get(if lang == "fr" { "en" } else { "fr" }));
            if let Some(causes_list) = lang_causes {
                if !causes_list.is_empty() {
                    result.push_str(causes_label);
                    result.push('\n');
                    for cause in causes_list {
                        result.push_str(&format!("- {}\n", cause));
                    }
                }
            }
        }

        if let Some(quick_check) = &tips.quick_check {
            let lang_check = quick_check.get(lang).or_else(|| quick_check.get(if lang == "fr" { "en" } else { "fr" }));
            if let Some(check) = lang_check {
                if !check.is_empty() {
                    result.push_str(check_label);
                    result.push_str(check);
                    result.push('\n');
                }
            }
        }

        result.trim_end().to_string()
    })
}

/// Get structured repair data (causes, quick_check, difficulty)
pub fn get_dtc_repair_data(code: &str, lang: &str) -> (Option<Vec<String>>, Option<String>, Option<u32>) {
    if let Some(tips) = DTC_REPAIR_TIPS.get(code) {
        let lang_key = if lang == "fr" { "fr" } else { "en" };
        let causes = tips.causes.as_ref()
            .and_then(|c| c.get(lang_key))
            .cloned();
        let quick_check = tips.quick_check.as_ref()
            .and_then(|q| q.get(lang_key))
            .cloned();
        let difficulty = tips.difficulty;
        (causes, quick_check, difficulty)
    } else {
        (None, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DtcStatus;

    #[test]
    fn test_decode_dtc_bytes_powertrain() {
        // P0001: Fuel Volume Regulator Control Circuit Open
        // Prefix P = 0x0, digit1 = 0, digit2 = 0, rest = 0x01
        let code = decode_dtc_bytes(0x00, 0x01);
        assert_eq!(code, "P0001");

        // P0200: Injector Circuit Malfunction
        let code = decode_dtc_bytes(0x02, 0x00);
        assert_eq!(code, "P0200");

        // P0ABC
        let code = decode_dtc_bytes(0x0A, 0xBC);
        assert_eq!(code, "P0ABC");
    }

    #[test]
    fn test_decode_dtc_bytes_chassis() {
        // C-codes (Chassis) have prefix bit = 01
        // C0001
        let code = decode_dtc_bytes(0x40, 0x01);
        assert_eq!(code, "C0001");

        // C0234
        let code = decode_dtc_bytes(0x42, 0x34);
        assert_eq!(code, "C0234");
    }

    #[test]
    fn test_decode_dtc_bytes_body() {
        // B-codes (Body) have prefix bit = 10
        // B0001
        let code = decode_dtc_bytes(0x80, 0x01);
        assert_eq!(code, "B0001");

        // B0ABC
        let code = decode_dtc_bytes(0x8A, 0xBC);
        assert_eq!(code, "B0ABC");
    }

    #[test]
    fn test_decode_dtc_bytes_network() {
        // U-codes (Network) have prefix bits 7:6 = 11
        // digit1 = bits 5:4 (gives 0-3)
        // digit2 = bits 3:0 (gives 0-F as hex)
        // "U0001" = (0xC0, 0x01) → U + 0 + 0 + 01 = U0001
        let code = decode_dtc_bytes(0xC0, 0x01);
        assert_eq!(code, "U0001");

        // "U0ABC" = (0xC0 | 0x0A = 0xCA, 0xBC) → U + 0 + A + BC = U0ABC
        let code = decode_dtc_bytes(0xCA, 0xBC);
        assert_eq!(code, "U0ABC");
    }

    #[test]
    fn test_parse_dtc_response_multiple_codes() {
        // Typical Mode 03 response: "43 01 23 04 56 00 00"
        // 43 = mode + 0x40, 01 23 = first DTC, 04 56 = second DTC, 00 00 = padding
        let response = "43 01 23 04 56 00 00";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        assert_eq!(dtcs.len(), 2);
        assert_eq!(dtcs[0].code, "P0123");
        assert_eq!(dtcs[0].source, "mode03");

        assert_eq!(dtcs[1].code, "P0456");
    }

    #[test]
    fn test_parse_dtc_response_single_code() {
        let response = "43 01 23";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        assert_eq!(dtcs.len(), 1);
        assert_eq!(dtcs[0].code, "P0123");
    }

    #[test]
    fn test_parse_dtc_response_no_codes() {
        // Empty response or all zeros
        let response = "43 00 00 00 00";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        assert_eq!(dtcs.len(), 0);
    }

    #[test]
    fn test_parse_dtc_response_no_data() {
        // Response too short
        let response = "43";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        assert_eq!(dtcs.len(), 0);
    }

    #[test]
    fn test_parse_dtc_response_empty() {
        let response = "";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        assert_eq!(dtcs.len(), 0);
    }

    #[test]
    fn test_parse_dtc_response_pending() {
        let response = "47 12 34 56 78";
        let dtcs = parse_dtc_response(response, DtcStatus::Pending, "mode07", "en");

        assert_eq!(dtcs.len(), 2);
    }

    #[test]
    fn test_decode_dtc_bytes_all_prefixes() {
        // Test each prefix combination
        let p_code = decode_dtc_bytes(0x00, 0x01); // P-code
        assert!(p_code.starts_with('P'));

        let c_code = decode_dtc_bytes(0x40, 0x01); // C-code
        assert!(c_code.starts_with('C'));

        let b_code = decode_dtc_bytes(0x80, 0x01); // B-code
        assert!(b_code.starts_with('B'));

        let u_code = decode_dtc_bytes(0xC0, 0x01); // U-code
        assert!(u_code.starts_with('U'));
    }

    #[test]
    fn test_get_dtc_description() {
        // Test that description lookup doesn't panic on missing codes
        let desc = get_dtc_description("P0000", "en");
        assert!(!desc.is_empty());

        let unknown_desc = get_dtc_description("PXXXX", "en");
        assert!(!unknown_desc.is_empty()); // Fallback message
    }

    #[test]
    fn test_parse_dtc_response_mixed_codes() {
        // Test with different DTC types (P, C, B, U codes)
        let response = "43 00 01 40 02 80 03 C0 04";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        assert_eq!(dtcs.len(), 4);
        assert!(dtcs[0].code.starts_with('P'));
        assert!(dtcs[1].code.starts_with('C'));
        assert!(dtcs[2].code.starts_with('B'));
        assert!(dtcs[3].code.starts_with('U'));
    }

    #[test]
    fn test_parse_dtc_invalid_hex() {
        // Response with invalid hex (should be filtered out)
        let response = "43 ZZ ZZ";
        let dtcs = parse_dtc_response(response, DtcStatus::Active, "mode03", "en");

        // Should filter out invalid hex and return empty
        assert_eq!(dtcs.len(), 0);
    }
}
