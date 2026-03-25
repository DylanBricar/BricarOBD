use serde_json::{json, Value};
use std::sync::LazyLock;

static WMI_DATABASE: LazyLock<Value> = LazyLock::new(|| {
    let json_str = include_str!("../../data/wmi_database.json");
    serde_json::from_str(json_str).unwrap_or_else(|_| json!({}))
});

/// Decode VIN (Vehicle Identification Number)
pub fn decode_vin(vin: &str) -> VinInfo {
    let vin = vin.trim().to_uppercase();
    let (make, country) = identify_make(&vin);
    let year = decode_year(&vin);

    VinInfo { vin: vin.clone(), make, country, year }
}

pub struct VinInfo {
    pub vin: String,
    pub make: String,
    pub country: String,
    pub year: u16,
}

/// Identify make and country from WMI (first 3 chars, fallback to 2 chars)
fn identify_make(vin: &str) -> (String, String) {
    if vin.len() < 2 {
        return ("Inconnu".to_string(), "Inconnu".to_string());
    }

    let wmi3 = &vin[..3.min(vin.len())];
    let wmi2 = &vin[..2];

    // Try 3-char WMI first
    if let Some(entry) = WMI_DATABASE.get(wmi3) {
        if let (Some(make), Some(country)) = (entry.get("make"), entry.get("country")) {
            if let (Some(make_str), Some(country_str)) = (make.as_str(), country.as_str()) {
                return (make_str.to_string(), country_str.to_string());
            }
        }
    }

    // Try 2-char fallback
    if let Some(entry) = WMI_DATABASE.get(wmi2) {
        if let (Some(make), Some(country)) = (entry.get("make"), entry.get("country")) {
            if let (Some(make_str), Some(country_str)) = (make.as_str(), country.as_str()) {
                return (make_str.to_string(), country_str.to_string());
            }
        }
    }

    ("Inconnu".to_string(), "Inconnu".to_string())
}

/// Decode model year from VIN position 10
fn decode_year(vin: &str) -> u16 {
    if vin.len() < 10 {
        return 0;
    }

    let year_char = vin.chars().nth(9).unwrap_or('0');
    match year_char {
        'A' => 2010, 'B' => 2011, 'C' => 2012, 'D' => 2013,
        'E' => 2014, 'F' => 2015, 'G' => 2016, 'H' => 2017,
        'J' => 2018, 'K' => 2019, 'L' => 2020, 'M' => 2021,
        'N' => 2022, 'P' => 2023, 'R' => 2024, 'S' => 2025,
        'T' => 2026, 'V' => 2027, 'W' => 2028, 'X' => 2029,
        'Y' => 2030,
        '1' => 2001, '2' => 2002, '3' => 2003, '4' => 2004,
        '5' => 2005, '6' => 2006, '7' => 2007, '8' => 2008,
        '9' => 2009,
        _ => 0,
    }
}
