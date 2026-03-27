/// DID value decoding with heuristic-based unit detection
/// Extracted from dashboard.rs to keep module size manageable

/// Decode DID value with heuristic formula based on the parameter name.
/// Order matters: more specific terms are checked first to avoid false positives
/// (e.g., "Battery Current" must match current before battery/voltage).
pub fn decode_did_value(bytes: &[u8], name: &str) -> (f64, String) {
    let raw = if bytes.len() >= 2 {
        (bytes[0] as f64) * 256.0 + (bytes[1] as f64)
    } else if bytes.len() == 1 {
        bytes[0] as f64
    } else {
        return (0.0, String::new());
    };

    let name_lower = name.to_lowercase();

    // === Most specific terms first ===

    // RPM (check before generic terms)
    if name_lower.contains("rpm") || name_lower.contains("régime") || name_lower.contains("regime") {
        return (raw / 4.0, "RPM".into());
    }

    // Speed (check before distance/km)
    if name_lower.contains("speed") || name_lower.contains("vitesse") || name_lower.contains("km/h") {
        return (raw, "km/h".into());
    }

    // Temperature (check before voltage — "temp" is unambiguous)
    if name_lower.contains("temp") || name_lower.contains("température") || name_lower.contains("°c") {
        if raw > 1000.0 {
            return (raw / 10.0 - 40.0, "°C".into());
        }
        return (raw - 40.0, "°C".into());
    }

    // Current/Amperage (check BEFORE voltage — "Battery Current" must match here, not voltage)
    if name_lower.contains("current") || name_lower.contains("courant") || name_lower.contains("ampère") || name_lower.contains("ampere") {
        return (raw / 1000.0, "A".into());
    }

    // Pressure (check BEFORE percentage — "Charge Pressure" must match here, not %)
    if name_lower.contains("press") || name_lower.contains("pression") || name_lower.contains("bar") {
        if name_lower.contains("bar") {
            return (raw / 1000.0, "bar".into());
        }
        return (raw, "kPa".into());
    }

    // Voltage (check after current and pressure)
    if name_lower.contains("volt") || name_lower.contains("tension") || name_lower.contains("battery") || name_lower.contains("batterie") {
        if raw > 1000.0 {
            return (raw / 1000.0, "V".into());
        }
        return (raw / 100.0, "V".into());
    }

    // Percentage (check last among common types — "charge", "load" are ambiguous)
    if name_lower.contains("%") || name_lower.contains("percent") || name_lower.contains("taux")
        || name_lower.contains("throttle") || name_lower.contains("papillon")
        || name_lower.contains("fuel level") || name_lower.contains("niveau") {
        if bytes.len() == 1 {
            return (raw * 100.0 / 255.0, "%".into());
        }
        return (raw * 100.0 / 65535.0, "%".into());
    }
    // "load" and "charge" only if no other category matched
    if name_lower.contains("load") || name_lower.contains("charge") {
        if bytes.len() == 1 {
            return (raw * 100.0 / 255.0, "%".into());
        }
        return (raw * 100.0 / 65535.0, "%".into());
    }

    // Time (seconds or ms)
    if name_lower.contains("time") || name_lower.contains("temps") || name_lower.contains("durée") || name_lower.contains("duration") {
        if raw > 60000.0 {
            return (raw / 1000.0, "s".into());
        }
        return (raw, "ms".into());
    }

    // Distance (km)
    if name_lower.contains("distance") || name_lower.contains("km") || name_lower.contains("mile") {
        return (raw, "km".into());
    }

    // Default: raw value, no unit
    (raw, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_did_value_rpm() {
        // raw = 0x0C*256 + 0x00 = 3072, RPM = raw/4 = 768
        let (value, unit) = decode_did_value(&[0x0C, 0x00], "Engine RPM");
        assert!((value - 768.0).abs() < 0.1);
        assert_eq!(unit, "RPM");
    }

    #[test]
    fn test_decode_did_value_speed() {
        // raw = 0x00*256 + 0x3C = 60
        let (value, unit) = decode_did_value(&[0x00, 0x3C], "Vehicle Speed");
        assert_eq!(value, 60.0);
        assert_eq!(unit, "km/h");
    }

    #[test]
    fn test_decode_did_value_temperature() {
        // raw = 0x00*256 + 0x69 = 105, temp = 105 - 40 = 65
        let (value, unit) = decode_did_value(&[0x00, 0x69], "Coolant Temperature");
        assert!((value - 65.0).abs() < 0.1);
        assert_eq!(unit, "°C");
    }

    #[test]
    fn test_decode_did_value_temperature_high_raw() {
        // raw = 0x0F*256 + 0xA0 = 4000, > 1000 → raw/10 - 40 = 360
        let (value, unit) = decode_did_value(&[0x0F, 0xA0], "Temperature");
        assert!((value - 360.0).abs() < 0.1);
        assert_eq!(unit, "°C");
    }

    #[test]
    fn test_decode_did_value_temperature_small() {
        // raw = 0x50 = 80, temp = 80 - 40 = 40
        let (value, unit) = decode_did_value(&[0x50], "temp");
        assert!((value - 40.0).abs() < 0.1);
        assert_eq!(unit, "°C");
    }

    #[test]
    fn test_decode_did_value_voltage_high() {
        // raw = 0x05*256 + 0xDC = 1500, > 1000 → 1500/1000 = 1.5V
        let (value, unit) = decode_did_value(&[0x05, 0xDC], "Battery Voltage");
        assert!((value - 1.5).abs() < 0.01);
        assert_eq!(unit, "V");
    }

    #[test]
    fn test_decode_did_value_voltage_low() {
        // raw = 0x00*256 + 0x50 = 80, <= 1000 → 80/100 = 0.8V
        let (value, unit) = decode_did_value(&[0x00, 0x50], "Voltage");
        assert!((value - 0.8).abs() < 0.01);
        assert_eq!(unit, "V");
    }

    #[test]
    fn test_decode_did_value_percentage_1byte() {
        // raw = 0x80 = 128, 128*100/255 ≈ 50.2%
        let (value, unit) = decode_did_value(&[0x80], "Throttle Position %");
        assert!((value - 50.2).abs() < 0.5);
        assert_eq!(unit, "%");
    }

    #[test]
    fn test_decode_did_value_time_ms() {
        // raw = 0x00*256 + 0x3C = 60, <= 60000 → 60 ms
        let (value, unit) = decode_did_value(&[0x00, 0x3C], "Engine Run Time");
        assert_eq!(value, 60.0);
        assert_eq!(unit, "ms");
    }

    #[test]
    fn test_decode_did_value_time_seconds() {
        // raw = 0xFF*256 + 0xFF = 65535, > 60000 → 65535/1000 = 65.535s
        let (value, unit) = decode_did_value(&[0xFF, 0xFF], "Duration");
        assert!((value - 65.535).abs() < 0.01);
        assert_eq!(unit, "s");
    }

    #[test]
    fn test_decode_did_value_empty() {
        let (value, unit) = decode_did_value(&[], "Generic Name");
        assert_eq!(value, 0.0);
        assert_eq!(unit, "");
    }

    #[test]
    fn test_decode_did_value_generic_name() {
        // raw = 0x00*256 + 0x42 = 66, no match → raw value
        let (value, unit) = decode_did_value(&[0x00, 0x42], "Unknown DID");
        assert_eq!(value, 66.0);
        assert_eq!(unit, "");
    }

    #[test]
    fn test_decode_did_value_current() {
        // raw = 0x03*256 + 0xE8 = 1000, "Current" → 1000/1000 = 1.0A
        let (value, unit) = decode_did_value(&[0x03, 0xE8], "Battery Current");
        assert!((value - 1.0).abs() < 0.01);
        assert_eq!(unit, "A");
    }

    #[test]
    fn test_decode_did_value_pressure_kpa() {
        // raw = 0x00*256 + 0x64 = 100, "Intake Pressure" → (100, "kPa")
        let (value, unit) = decode_did_value(&[0x00, 0x64], "Intake Pressure");
        assert_eq!(value, 100.0);
        assert_eq!(unit, "kPa");
    }

    #[test]
    fn test_decode_did_value_pressure_bar() {
        // raw = 0x01*256 + 0xF4 = 500, "Charge Pressure bar" → 500/1000 = 0.5 bar
        let (value, unit) = decode_did_value(&[0x01, 0xF4], "Charge Pressure bar");
        assert!((value - 0.5).abs() < 0.01);
        assert_eq!(unit, "bar");
    }
}
