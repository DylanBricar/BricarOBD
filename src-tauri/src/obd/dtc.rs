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

/// Get repair tips for DTC from embedded JSON database
pub fn get_dtc_repair_tips(code: &str) -> Option<String> {
    DTC_REPAIR_TIPS.get(code).map(|tips| {
        let mut result = String::new();

        if let Some(causes) = &tips.causes {
            if let Some(fr_causes) = causes.get("fr") {
                if !fr_causes.is_empty() {
                    result.push_str("Causes possibles:\n");
                    for cause in fr_causes {
                        result.push_str(&format!("- {}\n", cause));
                    }
                }
            }
        }

        if let Some(quick_check) = &tips.quick_check {
            if let Some(fr_check) = quick_check.get("fr") {
                if !fr_check.is_empty() {
                    result.push_str("Vérification rapide: ");
                    result.push_str(fr_check);
                    result.push('\n');
                }
            }
        }

        result.trim_end().to_string()
    })
}
