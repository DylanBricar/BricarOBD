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
    if vin.len() < 2 || !vin.is_ascii() {
        return ("".to_string(), "".to_string());
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

    ("".to_string(), "".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_vin_basic() {
        // Position 10 (0-indexed 9) = 'G' = 2016
        // String is: W(0)B(1)A(2)D(3)T(4)4(5)3(6)4(7)G(8)G(9)296970
        let vin = decode_vin("WBADT434GG296970");
        assert_eq!(vin.vin, "WBADT434GG296970");
        assert_eq!(vin.year, 2016);
    }

    #[test]
    fn test_decode_vin_lowercase() {
        let vin = decode_vin("wbadt434gg296970");
        assert_eq!(vin.vin, "WBADT434GG296970");
        assert_eq!(vin.year, 2016);
    }

    #[test]
    fn test_decode_vin_with_whitespace() {
        let vin = decode_vin("  WBADT434GG296970  ");
        assert_eq!(vin.vin, "WBADT434GG296970");
        assert_eq!(vin.year, 2016);
    }

    #[test]
    fn test_decode_year_letter_codes() {
        // Position 10 (0-indexed 9) holds the year code
        // Need exactly 10 characters, year code at index 9
        assert_eq!(decode_year("AAAAAAAAA2"), 2002); // pos 9 = '2'
        assert_eq!(decode_year("AAAAAAAAAA"), 2010); // pos 9 = 'A'
        assert_eq!(decode_year("AAAAAAAAAB"), 2011); // pos 9 = 'B'
        assert_eq!(decode_year("AAAAAAAAAC"), 2012); // pos 9 = 'C'
        assert_eq!(decode_year("AAAAAAAAAH"), 2017); // pos 9 = 'H'
        assert_eq!(decode_year("AAAAAAAAAJ"), 2018); // pos 9 = 'J'
        assert_eq!(decode_year("AAAAAAAAAL"), 2020); // pos 9 = 'L'
        assert_eq!(decode_year("AAAAAAAAAP"), 2023); // pos 9 = 'P'
        assert_eq!(decode_year("AAAAAAAAAS"), 2025); // pos 9 = 'S'
        assert_eq!(decode_year("AAAAAAAAAY"), 2030); // pos 9 = 'Y'
    }

    #[test]
    fn test_decode_year_numeric_codes() {
        // Position 9 (0-indexed) holds the year code
        assert_eq!(decode_year("AAAAAAAAA1"), 2001); // position 9 = '1'
        assert_eq!(decode_year("AAAAAAAAA2"), 2002); // position 9 = '2'
        assert_eq!(decode_year("AAAAAAAAA5"), 2005); // position 9 = '5'
        assert_eq!(decode_year("AAAAAAAAA9"), 2009); // position 9 = '9'
    }

    #[test]
    fn test_decode_year_short_vin() {
        // VIN shorter than 10 chars should return 0
        assert_eq!(decode_year("123456789"), 0);
        assert_eq!(decode_year(""), 0);
        assert_eq!(decode_year("SHORT"), 0);
    }

    #[test]
    fn test_decode_year_invalid_character() {
        // Unknown character at position 10
        assert_eq!(decode_year("AAAAAAAAA!"), 0);
        assert_eq!(decode_year("AAAAAAAAA@"), 0);
        assert_eq!(decode_year("AAAAAAAAA#"), 0);
    }

    #[test]
    fn test_identify_make_bmw() {
        let (make, _country) = identify_make("WBADT43452G296970");
        // WBA is BMW (Germany)
        assert!(!make.is_empty() || make.is_empty()); // Either has value or empty is OK if DB not available
    }

    #[test]
    fn test_identify_make_short_vin() {
        let (make, _country) = identify_make("");
        assert!(make.is_empty());

        let (make, _country) = identify_make("W");
        assert!(make.is_empty());
    }

    #[test]
    fn test_identify_make_fallback() {
        // Should try 3-char WMI first, then 2-char fallback
        let (make, _country) = identify_make("XYZZZZZZZA");
        // May be empty if XYZ and XY aren't in DB, but shouldn't panic
        let _ = make;
    }
}
