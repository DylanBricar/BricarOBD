/// Decode an undocumented manufacturer DID without inventing an engineering
/// formula. A label such as "temperature" is not sufficient evidence for a
/// scale, offset, signedness or byte order; guessing here can turn a valid ECU
/// frame into a dangerously misleading diagnostic value.
pub fn decode_did_value(bytes: &[u8], _name: &str) -> (f64, String) {
    if bytes.is_empty() {
        return (0.0, String::new());
    }

    // f64 represents every integer exactly through 53 bits. Keep the numeric
    // convenience value exact and always expose the complete hexadecimal frame.
    let raw = bytes
        .iter()
        .take(6)
        .fold(0_u64, |value, byte| (value << 8) | u64::from(*byte));
    let raw_hex = bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();

    (raw as f64, format!("raw 0x{raw_hex}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manufacturer_labels_do_not_trigger_guessed_formulas() {
        let (value, unit) = decode_did_value(&[0x0C, 0x00], "Engine RPM");
        assert_eq!(value, 3072.0);
        assert_eq!(unit, "raw 0x0C00");
    }

    #[test]
    fn complete_raw_frame_is_exposed() {
        let (value, unit) = decode_did_value(&[0x01, 0x02, 0x03, 0x04], "Injector correction");
        assert_eq!(value, 0x0102_0304 as f64);
        assert_eq!(unit, "raw 0x01020304");
    }

    #[test]
    fn empty_frame_is_empty() {
        assert_eq!(decode_did_value(&[], "Unknown"), (0.0, String::new()));
    }
}
