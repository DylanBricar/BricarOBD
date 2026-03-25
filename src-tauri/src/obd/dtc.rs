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
pub fn parse_dtc_response(response: &str, status: DtcStatus, source: &str) -> Vec<DtcCode> {
    let mut dtcs = Vec::new();

    // Response format: "43 01 23 04 56 00 00"
    let bytes: Vec<u8> = response
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if bytes.len() < 2 {
        return dtcs;
    }

    // Skip first byte (mode + 0x40), process pairs
    let data = &bytes[1..];
    for chunk in data.chunks(2) {
        if chunk.len() == 2 && (chunk[0] != 0 || chunk[1] != 0) {
            let code = decode_dtc_bytes(chunk[0], chunk[1]);
            let description = get_dtc_description(&code);
            let repair_tips = get_dtc_repair_tips(&code);

            dtcs.push(DtcCode {
                code,
                description,
                status: status.clone(),
                source: source.to_string(),
                repair_tips,
            });
        }
    }

    dtcs
}

/// Get DTC description from embedded JSON database
pub fn get_dtc_description(code: &str) -> String {
    DTC_DESCRIPTIONS
        .get(code)
        .cloned()
        .unwrap_or_else(|| format!("Code {} - Description non disponible", code))
}

/// Get repair tips for DTC from embedded JSON database (language-aware)
pub fn get_dtc_repair_tips(code: &str) -> Option<String> {
    get_dtc_repair_tips_lang(code, "fr")
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
