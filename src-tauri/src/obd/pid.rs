use crate::models::PidDefinition;

/// Helper for bilingual PID names
fn pn(lang: &str, fr: &str, en: &str) -> String {
    if lang == "fr" { fr.to_string() } else { en.to_string() }
}

/// Decode PID value from raw bytes
pub fn decode_pid(pid: u16, data: &[u8]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let a = data[0] as f64;
    let b = data.get(1).copied().unwrap_or(0) as f64;
    let _c = data.get(2).copied().unwrap_or(0) as f64;
    let _d = data.get(3).copied().unwrap_or(0) as f64;

    Some(match pid {
        0x03 => a * 256.0 + b,                                         // Fuel System Status (2 bytes)
        0x04 => (a * 100.0) / 255.0,                                  // Engine Load
        0x05 => a - 40.0,                                              // Coolant Temperature
        0x06 | 0x07 | 0x08 | 0x09 => ((a - 128.0) * 100.0) / 128.0,  // Fuel Trim Bank 1/2
        0x0A => a * 3.0,                                               // Fuel Pressure
        0x0B => a,                                                     // Intake Manifold Pressure
        0x0C => (a * 256.0 + b) / 4.0,                                // Engine RPM
        0x0D => a,                                                     // Vehicle Speed
        0x0E => (a / 2.0) - 64.0,                                     // Timing Advance
        0x0F => a - 40.0,                                              // Intake Air Temperature
        0x10 => (a * 256.0 + b) / 100.0,                              // MAF Rate
        0x11 => (a * 100.0) / 255.0,                                  // Throttle Position
        0x12 => a,                                                     // Commanded Secondary Air
        0x13 => a,                                                     // O2 Sensors Present
        0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1A | 0x1B => (a * 256.0 + b) / 200.0, // O2 Sensor Voltages (2 bytes)
        0x1C => a,                                                     // OBD Standard
        0x1E => a,                                                     // Auxiliary Input Status
        0x1F => a * 256.0 + b,                                         // Run Time Since Start
        0x21 => a * 256.0 + b,                                         // Distance with MIL On
        0x22 => (a * 256.0 + b) * 0.079,                              // Fuel Rail Pressure (rel)
        0x23 => (a * 256.0 + b) * 10.0,                               // Fuel Rail Pressure (diesel)
        0x24 | 0x25 | 0x26 | 0x27 | 0x28 | 0x29 | 0x2A | 0x2B => ((a * 256.0 + b) * 2.0) / 65536.0, // O2 Sensor Lambda
        0x2C => (a * 100.0) / 255.0,                                  // Commanded EGR
        0x2D => ((a - 128.0) * 100.0) / 128.0,                        // EGR Error
        0x2E => (a * 100.0) / 255.0,                                  // Commanded EVAP Purge
        0x2F => (a * 100.0) / 255.0,                                  // Fuel Tank Level
        0x30 => a,                                                     // Warm-ups Since Clear
        0x31 => a * 256.0 + b,                                         // Distance Since Clear
        0x32 => (((a as u16 * 256 + b as u16) as i16) as f64) / 4.0,  // EVAP Vapor Pressure
        0x33 => a,                                                     // Barometric Pressure
        0x34 | 0x35 | 0x36 | 0x37 | 0x38 | 0x39 | 0x3A | 0x3B => ((a * 256.0 + b) * 2.0) / 65536.0, // O2 Sensor Lambda/Current (returns lambda)
        0x3C => (a * 256.0 + b) / 10.0 - 40.0,                        // Catalyst Temp B1S1
        0x3D => (a * 256.0 + b) / 10.0 - 40.0,                        // Catalyst Temp B2S1
        0x3E => (a * 256.0 + b) / 10.0 - 40.0,                        // Catalyst Temp B1S2
        0x3F => (a * 256.0 + b) / 10.0 - 40.0,                        // Catalyst Temp B2S2
        0x42 => (a * 256.0 + b) / 1000.0,                             // Control Module Voltage
        0x43 => ((a * 256.0 + b) * 100.0) / 255.0,                    // Absolute Load Value
        0x44 => ((a * 256.0 + b) * 2.0) / 65536.0,                    // Commanded Equiv Ratio
        0x45 => (a * 100.0) / 255.0,                                  // Relative Throttle Pos
        0x46 => a - 40.0,                                              // Ambient Air Temperature
        0x47 => (a * 100.0) / 255.0,                                  // Absolute Throttle B
        0x48 => (a * 100.0) / 255.0,                                  // Absolute Throttle C
        0x49 => (a * 100.0) / 255.0,                                  // Accel Pedal Pos D
        0x4A => (a * 100.0) / 255.0,                                  // Accel Pedal Pos E
        0x4B => (a * 100.0) / 255.0,                                  // Accel Pedal Pos F
        0x4C => (a * 100.0) / 255.0,                                  // Commanded Throttle
        0x4D => a * 256.0 + b,                                         // Time with MIL On
        0x4E => a * 256.0 + b,                                         // Time Since Clear
        0x51 => a,                                                     // Fuel Type
        0x52 => (a * 100.0) / 255.0,                                  // Ethanol Fuel %
        0x5A => (a * 100.0) / 255.0,                                  // Relative Accel Pos
        0x5B => (a * 100.0) / 255.0,                                  // Hybrid Battery Life
        0x5C => a - 40.0,                                              // Engine Oil Temperature
        0x5D => ((a * 256.0 + b) - 26880.0) / 128.0,                  // Fuel Injection Timing
        0x5E => (a * 256.0 + b) * 0.05,                               // Engine Fuel Rate
        0x61 => a - 125.0,                                             // Driver Torque Demand
        0x62 => a - 125.0,                                             // Actual Engine Torque
        0x63 => a * 256.0 + b,                                         // Engine Ref Torque
        0x67 => b - 40.0,                                              // Engine Coolant Temp 2
        0x68 => b - 40.0,                                              // Intake Air Temp 2
        _ => a,                                                        // Default: raw value A
    })
}

/// Get all standard OBD-II PID definitions (bilingual FR/EN)
pub fn get_pid_definitions(lang: &str) -> Vec<PidDefinition> {
    vec![
        PidDefinition { pid: 0x03, name: pn(lang, "État système carburant", "Fuel System Status"), description: pn(lang, "Mode de fonctionnement du système carburant", "Fuel system operating mode"), unit: "".into(), min: 0.0, max: 255.0, bytes: 2 },
        PidDefinition { pid: 0x04, name: pn(lang, "Charge moteur", "Engine Load"), description: pn(lang, "Valeur calculée de charge moteur", "Calculated engine load value"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x05, name: pn(lang, "Temp. liquide refroidissement", "Coolant Temperature"), description: pn(lang, "Température du liquide de refroidissement", "Engine coolant temperature"), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 1 },
        PidDefinition { pid: 0x06, name: pn(lang, "Correction carburant CT B1", "Short Term Fuel Trim B1"), description: pn(lang, "Correction carburant court terme banc 1", "Short term fuel trim bank 1"), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x07, name: pn(lang, "Correction carburant LT B1", "Long Term Fuel Trim B1"), description: pn(lang, "Correction carburant long terme banc 1", "Long term fuel trim bank 1"), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x08, name: pn(lang, "Correction carburant CT B2", "Short Term Fuel Trim B2"), description: pn(lang, "Correction carburant court terme banc 2", "Short term fuel trim bank 2"), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x09, name: pn(lang, "Correction carburant LT B2", "Long Term Fuel Trim B2"), description: pn(lang, "Correction carburant long terme banc 2", "Long term fuel trim bank 2"), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x0A, name: pn(lang, "Pression carburant", "Fuel Pressure"), description: pn(lang, "Pression du carburant", "Fuel pressure gauge"), unit: "kPa".into(), min: 0.0, max: 765.0, bytes: 1 },
        PidDefinition { pid: 0x0B, name: pn(lang, "Pression collecteur admission", "Intake Manifold Pressure"), description: pn(lang, "Pression absolue du collecteur d'admission", "Intake manifold absolute pressure"), unit: "kPa".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x0C, name: pn(lang, "Régime moteur", "Engine RPM"), description: pn(lang, "Régime moteur", "Engine speed"), unit: "RPM".into(), min: 0.0, max: 16383.75, bytes: 2 },
        PidDefinition { pid: 0x0D, name: pn(lang, "Vitesse véhicule", "Vehicle Speed"), description: pn(lang, "Vitesse du véhicule", "Vehicle speed"), unit: "km/h".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x0E, name: pn(lang, "Avance à l'allumage", "Timing Advance"), description: pn(lang, "Avance à l'allumage cylindre 1", "Ignition timing advance for #1 cylinder"), unit: "°".into(), min: -64.0, max: 63.5, bytes: 1 },
        PidDefinition { pid: 0x0F, name: pn(lang, "Temp. air admission", "Intake Air Temperature"), description: pn(lang, "Température de l'air d'admission", "Intake air temperature"), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 1 },
        PidDefinition { pid: 0x10, name: pn(lang, "Débit air (MAF)", "Mass Air Flow (MAF)"), description: pn(lang, "Débit massique d'air", "Mass air flow sensor air flow rate"), unit: "g/s".into(), min: 0.0, max: 655.35, bytes: 2 },
        PidDefinition { pid: 0x11, name: pn(lang, "Position papillon", "Throttle Position"), description: pn(lang, "Position absolue du papillon", "Absolute throttle position"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x12, name: pn(lang, "Air secondaire commandé", "Commanded Secondary Air"), description: pn(lang, "État de l'air secondaire commandé", "Commanded secondary air status"), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x13, name: pn(lang, "Sondes O2 présentes", "O2 Sensors Present"), description: pn(lang, "Sondes O2 présentes (2 bancs)", "O2 sensors present (2 banks)"), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x14, name: pn(lang, "Sonde O2 B1S1", "O2 Sensor B1S1"), description: pn(lang, "Tension sonde O2 banc 1 capteur 1", "O2 sensor bank 1 sensor 1"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x15, name: pn(lang, "Sonde O2 B1S2", "O2 Sensor B1S2"), description: pn(lang, "Tension sonde O2 banc 1 capteur 2", "O2 sensor bank 1 sensor 2"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x16, name: pn(lang, "Sonde O2 B1S3", "O2 Sensor B1S3"), description: pn(lang, "Tension sonde O2 banc 1 capteur 3", "O2 sensor bank 1 sensor 3"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x17, name: pn(lang, "Sonde O2 B1S4", "O2 Sensor B1S4"), description: pn(lang, "Tension sonde O2 banc 1 capteur 4", "O2 sensor bank 1 sensor 4"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x18, name: pn(lang, "Sonde O2 B2S1", "O2 Sensor B2S1"), description: pn(lang, "Tension sonde O2 banc 2 capteur 1", "O2 sensor bank 2 sensor 1"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x19, name: pn(lang, "Sonde O2 B2S2", "O2 Sensor B2S2"), description: pn(lang, "Tension sonde O2 banc 2 capteur 2", "O2 sensor bank 2 sensor 2"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x1A, name: pn(lang, "Sonde O2 B2S3", "O2 Sensor B2S3"), description: pn(lang, "Tension sonde O2 banc 2 capteur 3", "O2 sensor bank 2 sensor 3"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x1B, name: pn(lang, "Sonde O2 B2S4", "O2 Sensor B2S4"), description: pn(lang, "Tension sonde O2 banc 2 capteur 4", "O2 sensor bank 2 sensor 4"), unit: "V".into(), min: 0.0, max: 1.275, bytes: 2 },
        PidDefinition { pid: 0x1C, name: pn(lang, "Norme OBD", "OBD Standard"), description: pn(lang, "Norme OBD du véhicule", "OBD standard this vehicle conforms to"), unit: "enum".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x1E, name: pn(lang, "Entrée auxiliaire", "Auxiliary Input"), description: pn(lang, "État PTO", "PTO status"), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x1F, name: pn(lang, "Durée fonctionnement moteur", "Engine Run Time"), description: pn(lang, "Durée de fonctionnement du moteur", "Engine run time"), unit: "s".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x21, name: pn(lang, "Distance avec témoin moteur", "Distance with MIL On"), description: pn(lang, "Distance parcourue avec témoin moteur allumé", "Distance traveled with MIL on"), unit: "km".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x22, name: pn(lang, "Pression rampe (relative)", "Fuel Rail Pressure (Rel)"), description: pn(lang, "Pression rampe relative au vide", "Fuel rail pressure relative to vacuum"), unit: "kPa".into(), min: 0.0, max: 5177.27, bytes: 2 },
        PidDefinition { pid: 0x23, name: pn(lang, "Pression rampe (diesel)", "Fuel Rail Pressure (Diesel)"), description: pn(lang, "Pression rampe carburant diesel", "Fuel rail gauge pressure (diesel)"), unit: "kPa".into(), min: 0.0, max: 655350.0, bytes: 2 },
        PidDefinition { pid: 0x24, name: pn(lang, "Lambda sonde O2 S1", "O2 Sensor 1 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 1", "O2 sensor 1 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x25, name: pn(lang, "Lambda sonde O2 S2", "O2 Sensor 2 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 2", "O2 sensor 2 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x26, name: pn(lang, "Lambda sonde O2 S3", "O2 Sensor 3 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 3", "O2 sensor 3 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x27, name: pn(lang, "Lambda sonde O2 S4", "O2 Sensor 4 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 4", "O2 sensor 4 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x28, name: pn(lang, "Lambda sonde O2 S5", "O2 Sensor 5 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 5", "O2 sensor 5 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x29, name: pn(lang, "Lambda sonde O2 S6", "O2 Sensor 6 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 6", "O2 sensor 6 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x2A, name: pn(lang, "Lambda sonde O2 S7", "O2 Sensor 7 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 7", "O2 sensor 7 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x2B, name: pn(lang, "Lambda sonde O2 S8", "O2 Sensor 8 Lambda"), description: pn(lang, "Rapport d'équivalence sonde O2 8", "O2 sensor 8 equivalence ratio"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x2C, name: pn(lang, "EGR commandé", "Commanded EGR"), description: pn(lang, "Position commandée vanne EGR", "Commanded EGR valve position"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x2D, name: pn(lang, "Erreur EGR", "EGR Error"), description: pn(lang, "Pourcentage d'erreur EGR", "EGR error percentage"), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x2E, name: pn(lang, "Purge EVAP commandée", "Commanded EVAP Purge"), description: pn(lang, "Purge évaporative commandée", "Commanded evaporative purge"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x2F, name: pn(lang, "Niveau réservoir carburant", "Fuel Tank Level"), description: pn(lang, "Niveau du réservoir de carburant", "Fuel tank level input"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x30, name: pn(lang, "Mises en chauffe depuis effacement", "Warm-ups Since Clear"), description: pn(lang, "Mises en chauffe depuis effacement codes", "Warm-ups since codes cleared"), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x31, name: pn(lang, "Distance depuis effacement", "Distance Since Clear"), description: pn(lang, "Distance parcourue depuis effacement codes", "Distance traveled since codes cleared"), unit: "km".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x32, name: pn(lang, "Pression vapeur EVAP", "EVAP Vapor Pressure"), description: pn(lang, "Pression vapeur système EVAP", "Evap system vapor pressure"), unit: "Pa".into(), min: -8192.0, max: 8191.75, bytes: 2 },
        PidDefinition { pid: 0x33, name: pn(lang, "Pression barométrique", "Barometric Pressure"), description: pn(lang, "Pression barométrique absolue", "Absolute barometric pressure"), unit: "kPa".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x34, name: pn(lang, "Lambda/Courant sonde O2 S1", "O2 Sensor 1 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 1", "O2 sensor 1 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x35, name: pn(lang, "Lambda/Courant sonde O2 S2", "O2 Sensor 2 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 2", "O2 sensor 2 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x36, name: pn(lang, "Lambda/Courant sonde O2 S3", "O2 Sensor 3 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 3", "O2 sensor 3 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x37, name: pn(lang, "Lambda/Courant sonde O2 S4", "O2 Sensor 4 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 4", "O2 sensor 4 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x38, name: pn(lang, "Lambda/Courant sonde O2 S5", "O2 Sensor 5 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 5", "O2 sensor 5 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x39, name: pn(lang, "Lambda/Courant sonde O2 S6", "O2 Sensor 6 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 6", "O2 sensor 6 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x3A, name: pn(lang, "Lambda/Courant sonde O2 S7", "O2 Sensor 7 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 7", "O2 sensor 7 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x3B, name: pn(lang, "Lambda/Courant sonde O2 S8", "O2 Sensor 8 Lambda/Current"), description: pn(lang, "Rapport d'équivalence/Courant sonde O2 8", "O2 sensor 8 lambda and current"), unit: "λ".into(), min: 0.0, max: 2.0, bytes: 4 },
        PidDefinition { pid: 0x3C, name: pn(lang, "Temp. catalyseur B1S1", "Catalyst Temp B1S1"), description: pn(lang, "Température catalyseur banc 1 capteur 1", "Catalyst temperature bank 1 sensor 1"), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x3D, name: pn(lang, "Temp. catalyseur B2S1", "Catalyst Temp B2S1"), description: pn(lang, "Température catalyseur banc 2 capteur 1", "Catalyst temperature bank 2 sensor 1"), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x3E, name: pn(lang, "Temp. catalyseur B1S2", "Catalyst Temp B1S2"), description: pn(lang, "Température catalyseur banc 1 capteur 2", "Catalyst temperature bank 1 sensor 2"), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x3F, name: pn(lang, "Temp. catalyseur B2S2", "Catalyst Temp B2S2"), description: pn(lang, "Température catalyseur banc 2 capteur 2", "Catalyst temperature bank 2 sensor 2"), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x42, name: pn(lang, "Tension batterie", "Battery Voltage"), description: pn(lang, "Tension du module de commande", "Control module voltage"), unit: "V".into(), min: 0.0, max: 65.535, bytes: 2 },
        PidDefinition { pid: 0x43, name: pn(lang, "Charge absolue", "Absolute Load"), description: pn(lang, "Valeur de charge absolue", "Absolute load value"), unit: "%".into(), min: 0.0, max: 25700.0, bytes: 2 },
        PidDefinition { pid: 0x44, name: pn(lang, "Rapport air/carburant commandé", "Commanded Air-Fuel Ratio"), description: pn(lang, "Rapport d'équivalence air/carburant commandé", "Commanded air-fuel equivalence ratio"), unit: "ratio".into(), min: 0.0, max: 2.0, bytes: 2 },
        PidDefinition { pid: 0x45, name: pn(lang, "Position papillon relative", "Relative Throttle Pos"), description: pn(lang, "Position relative du papillon", "Relative throttle position"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x46, name: pn(lang, "Temp. air ambiant", "Ambient Air Temperature"), description: pn(lang, "Température de l'air ambiant", "Ambient air temperature"), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 1 },
        PidDefinition { pid: 0x47, name: pn(lang, "Position papillon B", "Throttle Position B"), description: pn(lang, "Position absolue papillon B", "Absolute throttle position B"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x48, name: pn(lang, "Position papillon C", "Throttle Position C"), description: pn(lang, "Position absolue papillon C", "Absolute throttle position C"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x49, name: pn(lang, "Position pédale D", "Accelerator Pedal D"), description: pn(lang, "Position pédale accélérateur D", "Accelerator pedal position D"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4A, name: pn(lang, "Position pédale E", "Accelerator Pedal E"), description: pn(lang, "Position pédale accélérateur E", "Accelerator pedal position E"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4B, name: pn(lang, "Position pédale F", "Accelerator Pedal F"), description: pn(lang, "Position pédale accélérateur F", "Accelerator pedal position F"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4C, name: pn(lang, "Papillon commandé", "Commanded Throttle"), description: pn(lang, "Actionneur papillon commandé", "Commanded throttle actuator"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4D, name: pn(lang, "Durée avec témoin moteur", "Time with MIL On"), description: pn(lang, "Durée de fonctionnement avec témoin moteur", "Time run with MIL on"), unit: "min".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x4E, name: pn(lang, "Durée depuis effacement", "Time Since Clear"), description: pn(lang, "Durée depuis effacement des codes", "Time since trouble codes cleared"), unit: "min".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x51, name: pn(lang, "Type carburant", "Fuel Type"), description: pn(lang, "Type de carburant (essence, diesel, etc.)", "Fuel type (gasoline, diesel, etc.)"), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x52, name: pn(lang, "Pourcentage éthanol", "Ethanol Fuel %"), description: pn(lang, "Pourcentage d'éthanol dans le carburant", "Ethanol fuel percentage"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x5A, name: pn(lang, "Position pédale relative", "Relative Accel Pedal"), description: pn(lang, "Position relative pédale accélérateur", "Relative accelerator pedal position"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x5B, name: pn(lang, "Vie batterie hybride", "Hybrid Battery Life"), description: pn(lang, "Durée de vie restante batterie hybride", "Hybrid battery pack remaining life"), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x5C, name: pn(lang, "Temp. huile moteur", "Engine Oil Temperature"), description: pn(lang, "Température de l'huile moteur", "Engine oil temperature"), unit: "°C".into(), min: -40.0, max: 210.0, bytes: 1 },
        PidDefinition { pid: 0x5D, name: pn(lang, "Calage injection", "Fuel Injection Timing"), description: pn(lang, "Calage de l'injection", "Fuel injection timing"), unit: "°".into(), min: -210.0, max: 301.99, bytes: 2 },
        PidDefinition { pid: 0x5E, name: pn(lang, "Consommation carburant", "Engine Fuel Rate"), description: pn(lang, "Consommation de carburant moteur", "Engine fuel rate"), unit: "L/h".into(), min: 0.0, max: 3212.75, bytes: 2 },
        PidDefinition { pid: 0x61, name: pn(lang, "Couple demandé conducteur", "Driver Torque Demand"), description: pn(lang, "Couple moteur demandé par le conducteur", "Driver's demand engine torque"), unit: "%".into(), min: -125.0, max: 130.0, bytes: 1 },
        PidDefinition { pid: 0x62, name: pn(lang, "Couple moteur réel", "Actual Engine Torque"), description: pn(lang, "Couple moteur réel", "Actual engine torque"), unit: "%".into(), min: -125.0, max: 130.0, bytes: 1 },
        PidDefinition { pid: 0x63, name: pn(lang, "Couple référence moteur", "Engine Ref Torque"), description: pn(lang, "Couple de référence moteur", "Engine reference torque"), unit: "Nm".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x67, name: pn(lang, "Temp. refroidissement 2", "Coolant Temperature 2"), description: pn(lang, "Capteurs température liquide refroidissement", "Engine coolant temperature sensors"), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 2 },
        PidDefinition { pid: 0x68, name: pn(lang, "Temp. air admission 2", "Intake Air Temp 2"), description: pn(lang, "Capteur température air d'admission", "Intake air temperature sensor"), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 2 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn test_decode_empty_data() {
        assert_eq!(decode_pid(0x0C, &[]), None);
    }

    #[test]
    fn test_decode_rpm() {
        // RPM = (A*256+B)/4 → [0x0C, 0x00] = (12*256+0)/4 = 768
        assert!(approx_eq(decode_pid(0x0C, &[0x0C, 0x00]).unwrap(), 768.0));
    }

    #[test]
    fn test_decode_rpm_max() {
        // RPM max = (255*256+255)/4 = 16383.75
        assert!(approx_eq(decode_pid(0x0C, &[0xFF, 0xFF]).unwrap(), 16383.75));
    }

    #[test]
    fn test_decode_speed() {
        assert!(approx_eq(decode_pid(0x0D, &[0x3C]).unwrap(), 60.0));
    }

    #[test]
    fn test_decode_speed_zero() {
        assert!(approx_eq(decode_pid(0x0D, &[0x00]).unwrap(), 0.0));
    }

    #[test]
    fn test_decode_coolant_temp() {
        // A-40 → 104-40 = 64°C
        assert!(approx_eq(decode_pid(0x05, &[0x68]).unwrap(), 64.0));
    }

    #[test]
    fn test_decode_coolant_temp_negative() {
        // 0-40 = -40°C
        assert!(approx_eq(decode_pid(0x05, &[0x00]).unwrap(), -40.0));
    }

    #[test]
    fn test_decode_engine_load() {
        // (A*100)/255 → (255*100)/255 = 100%
        assert!(approx_eq(decode_pid(0x04, &[0xFF]).unwrap(), 100.0));
    }

    #[test]
    fn test_decode_engine_load_half() {
        // (128*100)/255 ≈ 50.2%
        assert!(approx_eq(decode_pid(0x04, &[0x80]).unwrap(), 50.196));
    }

    #[test]
    fn test_decode_maf() {
        // (A*256+B)/100 → (1*256+244)/100 = 5.0
        assert!(approx_eq(decode_pid(0x10, &[0x01, 0xF4]).unwrap(), 5.0));
    }

    #[test]
    fn test_decode_maf_zero() {
        assert!(approx_eq(decode_pid(0x10, &[0x00, 0x00]).unwrap(), 0.0));
    }

    #[test]
    fn test_decode_fuel_tank() {
        // (255*100)/255 = 100%
        assert!(approx_eq(decode_pid(0x2F, &[0xFF]).unwrap(), 100.0));
    }

    #[test]
    fn test_decode_fuel_tank_half() {
        // (128*100)/255 ≈ 50.2%
        assert!(approx_eq(decode_pid(0x2F, &[0x80]).unwrap(), 50.196));
    }

    #[test]
    fn test_decode_voltage() {
        // (A*256+B)/1000 → (0x35*256+0x60)/1000 = 13.664V
        assert!(approx_eq(decode_pid(0x42, &[0x35, 0x60]).unwrap(), 13.664));
    }

    #[test]
    fn test_decode_voltage_low() {
        // (0*256+0)/1000 = 0V
        assert!(approx_eq(decode_pid(0x42, &[0x00, 0x00]).unwrap(), 0.0));
    }

    #[test]
    fn test_decode_o2_sensor() {
        // (A*256+B)/200 → (0*256+100)/200 = 0.5V
        assert!(approx_eq(decode_pid(0x14, &[0x00, 0x64]).unwrap(), 0.5));
    }

    #[test]
    fn test_decode_o2_sensor_all_pids() {
        for pid in [0x14u16, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B] {
            assert!(approx_eq(decode_pid(pid, &[0x00, 0x64]).unwrap(), 0.5), "Failed for PID 0x{:02X}", pid);
        }
    }

    #[test]
    fn test_decode_timing_advance_zero() {
        // A/2-64 → 128/2-64 = 0°
        assert!(approx_eq(decode_pid(0x0E, &[0x80]).unwrap(), 0.0));
    }

    #[test]
    fn test_decode_timing_advance_positive() {
        // 192/2-64 = 32°
        assert!(approx_eq(decode_pid(0x0E, &[0xC0]).unwrap(), 32.0));
    }

    #[test]
    fn test_decode_timing_advance_negative() {
        // 0/2-64 = -64°
        assert!(approx_eq(decode_pid(0x0E, &[0x00]).unwrap(), -64.0));
    }

    #[test]
    fn test_decode_evap_vapor_pressure_positive() {
        // [0x00, 0x04] as signed i16 = 4, /4 = 1.0
        assert!(approx_eq(decode_pid(0x32, &[0x00, 0x04]).unwrap(), 1.0));
    }

    #[test]
    fn test_decode_evap_vapor_pressure_negative() {
        // [0xFF, 0xFC] as signed i16 = -4, /4 = -1.0
        assert!(approx_eq(decode_pid(0x32, &[0xFF, 0xFC]).unwrap(), -1.0));
    }

    #[test]
    fn test_decode_fuel_trim_neutral() {
        // ((128-128)*100)/128 = 0%
        assert!(approx_eq(decode_pid(0x06, &[0x80]).unwrap(), 0.0));
    }

    #[test]
    fn test_decode_fuel_trim_rich() {
        // ((255-128)*100)/128 ≈ 99.22%
        assert!(approx_eq(decode_pid(0x06, &[0xFF]).unwrap(), 99.21875));
    }

    #[test]
    fn test_decode_fuel_trim_lean() {
        // ((0-128)*100)/128 = -100%
        assert!(approx_eq(decode_pid(0x06, &[0x00]).unwrap(), -100.0));
    }

    #[test]
    fn test_decode_fuel_trim_all_banks() {
        for pid in [0x06u16, 0x07, 0x08, 0x09] {
            assert!(approx_eq(decode_pid(pid, &[0x80]).unwrap(), 0.0), "Failed for PID 0x{:02X}", pid);
        }
    }

    #[test]
    fn test_decode_unknown_pid() {
        // Default: raw A value
        assert!(approx_eq(decode_pid(0xFE, &[0x42]).unwrap(), 66.0));
    }

    #[test]
    fn test_decode_two_byte_with_one_byte() {
        // RPM with 1 byte — B defaults to 0
        assert!(approx_eq(decode_pid(0x0C, &[0x0C]).unwrap(), 768.0));
    }

    #[test]
    fn test_decode_catalyst_temp() {
        // (A*256+B)/10-40 → (2*256+88)/10-40 = 20°C
        assert!(approx_eq(decode_pid(0x3C, &[0x02, 0x58]).unwrap(), 20.0));
    }

    #[test]
    fn test_decode_all_catalyst_temps() {
        for pid in [0x3Cu16, 0x3D, 0x3E, 0x3F] {
            assert!(approx_eq(decode_pid(pid, &[0x02, 0x58]).unwrap(), 20.0), "Failed for PID 0x{:02X}", pid);
        }
    }

    #[test]
    fn test_decode_commanded_equiv_ratio() {
        // ((A*256+B)*2)/65536 → (0x80*256+0)*2/65536 = 1.0
        assert!(approx_eq(decode_pid(0x44, &[0x80, 0x00]).unwrap(), 1.0));
    }

    #[test]
    fn test_decode_absolute_load() {
        // ((A*256+B)*100)/255 → (0*256+255)*100/255 = 100.0
        assert!(approx_eq(decode_pid(0x43, &[0x00, 0xFF]).unwrap(), 100.0));
    }

    #[test]
    fn test_decode_run_time() {
        // A*256+B → 1*256+0 = 256
        assert!(approx_eq(decode_pid(0x1F, &[0x01, 0x00]).unwrap(), 256.0));
    }

    #[test]
    fn test_decode_throttle_position() {
        // (A*100)/255 → (255*100)/255 = 100%
        assert!(approx_eq(decode_pid(0x11, &[0xFF]).unwrap(), 100.0));
    }

    #[test]
    fn test_decode_barometric_pressure() {
        assert!(approx_eq(decode_pid(0x33, &[0x65]).unwrap(), 101.0));
    }

    #[test]
    fn test_decode_distance_with_mil() {
        assert!(approx_eq(decode_pid(0x21, &[0x00, 0x64]).unwrap(), 100.0));
    }
}
