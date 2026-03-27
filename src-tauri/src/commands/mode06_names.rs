/// Helper for bilingual names
pub(crate) fn pn(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
}

/// Get bilingual Mode 06 test name and unit — comprehensive TID/MID coverage
pub(crate) fn get_mode06_name(tid: u8, mid: u8, lang: &str) -> (String, String) {
    match (tid, mid) {
        // === Misfire Monitoring (TID 0x01-0x0A) ===
        (0x01, _) => (pn(lang, "Raté d'allumage Cyl.1", "Misfire Cylinder 1"), "count".into()),
        (0x02, _) => (pn(lang, "Raté d'allumage Cyl.2", "Misfire Cylinder 2"), "count".into()),
        (0x03, _) => (pn(lang, "Raté d'allumage Cyl.3", "Misfire Cylinder 3"), "count".into()),
        (0x04, _) => (pn(lang, "Raté d'allumage Cyl.4", "Misfire Cylinder 4"), "count".into()),
        (0x05, _) => (pn(lang, "Raté d'allumage Cyl.5", "Misfire Cylinder 5"), "count".into()),
        (0x06, _) => (pn(lang, "Raté d'allumage Cyl.6", "Misfire Cylinder 6"), "count".into()),
        (0x07, _) => (pn(lang, "Raté d'allumage Cyl.7", "Misfire Cylinder 7"), "count".into()),
        (0x08, _) => (pn(lang, "Raté d'allumage Cyl.8", "Misfire Cylinder 8"), "count".into()),
        (0x09, _) => (pn(lang, "Raté d'allumage général", "General Misfire"), "count".into()),
        (0x0A, _) => (pn(lang, "Raté d'allumage aléatoire", "Random Misfire"), "count".into()),

        // === Fuel System Monitoring (TID 0x11-0x16) ===
        (0x11, _) => (pn(lang, "Correction carburant court terme B1", "Short Term Fuel Trim B1"), "%".into()),
        (0x12, _) => (pn(lang, "Correction carburant long terme B1", "Long Term Fuel Trim B1"), "%".into()),
        (0x13, _) => (pn(lang, "Correction carburant court terme B2", "Short Term Fuel Trim B2"), "%".into()),
        (0x14, _) => (pn(lang, "Correction carburant long terme B2", "Long Term Fuel Trim B2"), "%".into()),
        (0x15, _) => (pn(lang, "Pression système carburant", "Fuel System Pressure"), "kPa".into()),
        (0x16, _) => (pn(lang, "Pression rampe carburant", "Fuel Rail Pressure"), "kPa".into()),

        // === Catalyst Monitoring (TID 0x21-0x24) ===
        (0x21, 0x01) => (pn(lang, "Efficacité catalyseur B1", "Catalyst Efficiency B1"), "ratio".into()),
        (0x21, 0x02) => (pn(lang, "Efficacité catalyseur B2", "Catalyst Efficiency B2"), "ratio".into()),
        (0x21, _) => (pn(lang, "Efficacité catalyseur", "Catalyst Efficiency"), "ratio".into()),
        (0x22, 0x01) => (pn(lang, "Vieillissement catalyseur B1", "Catalyst Aging B1"), "ratio".into()),
        (0x22, 0x02) => (pn(lang, "Vieillissement catalyseur B2", "Catalyst Aging B2"), "ratio".into()),
        (0x22, _) => (pn(lang, "Vieillissement catalyseur", "Catalyst Aging"), "ratio".into()),
        (0x23, _) => (pn(lang, "Chauffage catalyseur B1", "Catalyst Heater B1"), "s".into()),
        (0x24, _) => (pn(lang, "Chauffage catalyseur B2", "Catalyst Heater B2"), "s".into()),

        // === O2 Sensor Monitoring (TID 0x29-0x2C) ===
        (0x29, 0x11) => (pn(lang, "Temps réponse O2 B1S1", "O2 Response Time B1S1"), "ms".into()),
        (0x29, 0x21) => (pn(lang, "Temps réponse O2 B1S2", "O2 Response Time B1S2"), "ms".into()),
        (0x29, _) => (pn(lang, "Temps réponse O2", "O2 Response Time"), "ms".into()),
        (0x2A, 0x11) => (pn(lang, "Amplitude O2 B1S1", "O2 Amplitude B1S1"), "V".into()),
        (0x2A, 0x21) => (pn(lang, "Amplitude O2 B1S2", "O2 Amplitude B1S2"), "V".into()),
        (0x2A, _) => (pn(lang, "Amplitude O2", "O2 Amplitude"), "V".into()),
        (0x2B, 0x11) => (pn(lang, "Temps réponse O2 B2S1", "O2 Response Time B2S1"), "ms".into()),
        (0x2B, 0x21) => (pn(lang, "Temps réponse O2 B2S2", "O2 Response Time B2S2"), "ms".into()),
        (0x2B, _) => (pn(lang, "Temps réponse O2 B2", "O2 Response Time B2"), "ms".into()),
        (0x2C, 0x11) => (pn(lang, "Amplitude O2 B2S1", "O2 Amplitude B2S1"), "V".into()),
        (0x2C, 0x21) => (pn(lang, "Amplitude O2 B2S2", "O2 Amplitude B2S2"), "V".into()),
        (0x2C, _) => (pn(lang, "Amplitude O2 B2", "O2 Amplitude B2"), "V".into()),

        // === EVAP System Monitoring (TID 0x31-0x35) ===
        (0x31, _) => (pn(lang, "Fuite EVAP (grande)", "EVAP Large Leak"), "Pa".into()),
        (0x32, _) => (pn(lang, "Fuite EVAP (petite)", "EVAP Small Leak"), "Pa".into()),
        (0x33, _) => (pn(lang, "Étanchéité canister EVAP", "EVAP Canister Close"), "Pa".into()),
        (0x34, _) => (pn(lang, "Pression EVAP", "EVAP System Pressure"), "Pa".into()),
        (0x35, _) => (pn(lang, "Purge EVAP", "EVAP Purge Flow"), "%".into()),

        // === EGR Monitoring (TID 0x3C-0x3D) ===
        (0x3C, _) => (pn(lang, "Débit EGR", "EGR Flow Rate"), "%".into()),
        (0x3D, _) => (pn(lang, "Erreur EGR", "EGR Error"), "%".into()),

        // === Secondary Air Injection (TID 0x41-0x42) ===
        (0x41, _) => (pn(lang, "Injection air secondaire B1", "Secondary Air Injection B1"), "g/s".into()),
        (0x42, _) => (pn(lang, "Injection air secondaire B2", "Secondary Air Injection B2"), "g/s".into()),

        // === A/C System (TID 0x51-0x52) ===
        (0x51, _) => (pn(lang, "Système A/C réfrigérant", "A/C Refrigerant"), "g".into()),
        (0x52, _) => (pn(lang, "Pression A/C", "A/C Pressure"), "kPa".into()),

        // === Heated O2 Sensor Heater (TID 0x61-0x66) ===
        (0x61, _) => (pn(lang, "Chauffage sonde O2 B1S1", "O2 Heater B1S1"), "s".into()),
        (0x62, _) => (pn(lang, "Chauffage sonde O2 B1S2", "O2 Heater B1S2"), "s".into()),
        (0x63, _) => (pn(lang, "Chauffage sonde O2 B2S1", "O2 Heater B2S1"), "s".into()),
        (0x64, _) => (pn(lang, "Chauffage sonde O2 B2S2", "O2 Heater B2S2"), "s".into()),
        (0x65, _) => (pn(lang, "Résistance chauffage O2 B1", "O2 Heater Resistance B1"), "Ω".into()),
        (0x66, _) => (pn(lang, "Résistance chauffage O2 B2", "O2 Heater Resistance B2"), "Ω".into()),

        // === VVT (Variable Valve Timing) (TID 0x71-0x74) ===
        (0x71, _) => (pn(lang, "Distribution variable admission B1", "VVT Intake B1"), "°".into()),
        (0x72, _) => (pn(lang, "Distribution variable échappement B1", "VVT Exhaust B1"), "°".into()),
        (0x73, _) => (pn(lang, "Distribution variable admission B2", "VVT Intake B2"), "°".into()),
        (0x74, _) => (pn(lang, "Distribution variable échappement B2", "VVT Exhaust B2"), "°".into()),

        // === Thermostat (TID 0x81) ===
        (0x81, _) => (pn(lang, "Thermostat", "Thermostat"), "°C".into()),

        // === Cold Start Emissions (TID 0x91) ===
        (0x91, _) => (pn(lang, "Réduction émissions démarrage à froid", "Cold Start Emission Reduction"), "s".into()),

        // === DPF / GPF (TID 0xA1-0xA3) ===
        (0xA1, _) => (pn(lang, "Pression différentielle FAP", "DPF Differential Pressure"), "kPa".into()),
        (0xA2, _) => (pn(lang, "Température entrée FAP", "DPF Inlet Temperature"), "°C".into()),
        (0xA3, _) => (pn(lang, "Régénération FAP", "DPF Regeneration"), "count".into()),

        // === Fallback ===
        _ => (format!("Test {:02X}/{:02X}", tid, mid), "raw".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pn_english() {
        assert_eq!(pn("en", "Erreur", "Error"), "Error");
    }

    #[test]
    fn test_pn_french() {
        assert_eq!(pn("fr", "Erreur", "Error"), "Erreur");
    }

    #[test]
    fn test_pn_other_lang_defaults_to_english() {
        assert_eq!(pn("es", "Erreur", "Error"), "Error");
    }

    #[test]
    fn test_get_mode06_name_misfire() {
        let (name, unit) = get_mode06_name(0x01, 0x00, "en");
        assert_eq!(name, "Misfire Cylinder 1");
        assert_eq!(unit, "count");
    }

    #[test]
    fn test_get_mode06_name_misfire_fr() {
        let (name, unit) = get_mode06_name(0x01, 0x00, "fr");
        assert_eq!(name, "Raté d'allumage Cyl.1");
        assert_eq!(unit, "count");
    }

    #[test]
    fn test_get_mode06_name_catalyst() {
        let (name, unit) = get_mode06_name(0x21, 0x01, "en");
        assert_eq!(name, "Catalyst Efficiency B1");
        assert_eq!(unit, "ratio");
    }

    #[test]
    fn test_get_mode06_name_catalyst_fr() {
        let (name, unit) = get_mode06_name(0x21, 0x01, "fr");
        assert_eq!(name, "Efficacité catalyseur B1");
        assert_eq!(unit, "ratio");
    }

    #[test]
    fn test_get_mode06_name_o2_sensor() {
        let (name, unit) = get_mode06_name(0x29, 0x11, "en");
        assert_eq!(name, "O2 Response Time B1S1");
        assert_eq!(unit, "ms");
    }

    #[test]
    fn test_get_mode06_name_o2_sensor_fr() {
        let (name, unit) = get_mode06_name(0x29, 0x11, "fr");
        assert_eq!(name, "Temps réponse O2 B1S1");
        assert_eq!(unit, "ms");
    }

    #[test]
    fn test_get_mode06_name_evap() {
        let (name, unit) = get_mode06_name(0x31, 0x00, "en");
        assert_eq!(name, "EVAP Large Leak");
        assert_eq!(unit, "Pa");
    }

    #[test]
    fn test_get_mode06_name_evap_fr() {
        let (name, unit) = get_mode06_name(0x31, 0x00, "fr");
        assert_eq!(name, "Fuite EVAP (grande)");
        assert_eq!(unit, "Pa");
    }

    #[test]
    fn test_get_mode06_name_egr() {
        let (name, unit) = get_mode06_name(0x3C, 0x00, "en");
        assert_eq!(name, "EGR Flow Rate");
        assert_eq!(unit, "%");
    }

    #[test]
    fn test_get_mode06_name_egr_fr() {
        let (name, unit) = get_mode06_name(0x3C, 0x00, "fr");
        assert_eq!(name, "Débit EGR");
        assert_eq!(unit, "%");
    }

    #[test]
    fn test_get_mode06_name_thermostat() {
        let (name, unit) = get_mode06_name(0x81, 0x00, "en");
        assert_eq!(name, "Thermostat");
        assert_eq!(unit, "°C");
    }

    #[test]
    fn test_get_mode06_name_thermostat_fr() {
        let (name, unit) = get_mode06_name(0x81, 0x00, "fr");
        assert_eq!(name, "Thermostat");
        assert_eq!(unit, "°C");
    }

    #[test]
    fn test_get_mode06_name_dpf() {
        let (name, unit) = get_mode06_name(0xA1, 0x00, "en");
        assert_eq!(name, "DPF Differential Pressure");
        assert_eq!(unit, "kPa");
    }

    #[test]
    fn test_get_mode06_name_dpf_fr() {
        let (name, unit) = get_mode06_name(0xA1, 0x00, "fr");
        assert_eq!(name, "Pression différentielle FAP");
        assert_eq!(unit, "kPa");
    }

    #[test]
    fn test_get_mode06_name_fallback() {
        let (name, unit) = get_mode06_name(0xFF, 0xFF, "en");
        assert_eq!(name, "Test FF/FF");
        assert_eq!(unit, "raw");
    }
}
