use crate::models::PidDefinition;

/// Decode PID value from raw bytes
pub fn decode_pid(pid: u16, data: &[u8]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let a = data[0] as f64;
    let b = data.get(1).copied().unwrap_or(0) as f64;
    let c = data.get(2).copied().unwrap_or(0) as f64;
    let d = data.get(3).copied().unwrap_or(0) as f64;

    Some(match pid {
        0x03 => a,                                                     // Fuel System Status
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
        0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1A | 0x1B => a / 200.0, // O2 Sensor Voltages
        0x1C => a,                                                     // OBD Standard
        0x1E => a,                                                     // Auxiliary Input Status
        0x1F => a * 256.0 + b,                                         // Run Time Since Start
        0x21 => a * 256.0 + b,                                         // Distance with MIL On
        0x22 => (a * 256.0 + b) * 0.079,                              // Fuel Rail Pressure (rel)
        0x23 => (a * 256.0 + b) * 10.0,                               // Fuel Rail Pressure (diesel)
        0x24 => ((a * 256.0 + b) * 2.0) / 65536.0,                    // O2 Sensor 1 Lambda
        0x2C => (a * 100.0) / 255.0,                                  // Commanded EGR
        0x2D => ((a - 128.0) * 100.0) / 128.0,                        // EGR Error
        0x2E => (a * 100.0) / 255.0,                                  // Commanded EVAP Purge
        0x2F => (a * 100.0) / 255.0,                                  // Fuel Tank Level
        0x30 => a,                                                     // Warm-ups Since Clear
        0x31 => a * 256.0 + b,                                         // Distance Since Clear
        0x32 => (a * 256.0 + b) / 4.0,                                // EVAP Vapor Pressure
        0x33 => a,                                                     // Barometric Pressure
        0x34 => (d + c * 256.0) / 256.0 - 128.0,                      // O2 Sensor 1 Current
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

/// Get all standard OBD-II PID definitions
pub fn get_pid_definitions() -> Vec<PidDefinition> {
    vec![
        PidDefinition { pid: 0x03, name: "État système carburant".into(), description: "Fuel system operating mode".into(), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x04, name: "Charge moteur".into(), description: "Calculated engine load value".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x05, name: "Température liquide refroidissement".into(), description: "Engine coolant temperature".into(), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 1 },
        PidDefinition { pid: 0x06, name: "Correction carburant court terme B1".into(), description: "Short term fuel trim bank 1".into(), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x07, name: "Correction carburant long terme B1".into(), description: "Long term fuel trim bank 1".into(), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x08, name: "Correction carburant court terme B2".into(), description: "Short term fuel trim bank 2".into(), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x09, name: "Correction carburant long terme B2".into(), description: "Long term fuel trim bank 2".into(), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x0A, name: "Pression carburant".into(), description: "Fuel pressure gauge".into(), unit: "kPa".into(), min: 0.0, max: 765.0, bytes: 1 },
        PidDefinition { pid: 0x0B, name: "Pression collecteur admission".into(), description: "Intake manifold absolute pressure".into(), unit: "kPa".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x0C, name: "Régime moteur".into(), description: "Engine speed".into(), unit: "RPM".into(), min: 0.0, max: 16383.75, bytes: 2 },
        PidDefinition { pid: 0x0D, name: "Vitesse véhicule".into(), description: "Vehicle speed".into(), unit: "km/h".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x0E, name: "Avance à l'allumage".into(), description: "Ignition timing advance for #1 cylinder".into(), unit: "°".into(), min: -64.0, max: 63.5, bytes: 1 },
        PidDefinition { pid: 0x0F, name: "Température air admission".into(), description: "Intake air temperature".into(), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 1 },
        PidDefinition { pid: 0x10, name: "Débit air (MAF)".into(), description: "Mass air flow sensor air flow rate".into(), unit: "g/s".into(), min: 0.0, max: 655.35, bytes: 2 },
        PidDefinition { pid: 0x11, name: "Position papillon".into(), description: "Absolute throttle position".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x12, name: "Air secondaire commandé".into(), description: "Commanded secondary air status".into(), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x13, name: "Sondes O2 présentes".into(), description: "O2 sensors present (2 banks)".into(), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x14, name: "Sonde O2 B1S1 tension".into(), description: "O2 sensor bank 1 sensor 1".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x15, name: "Sonde O2 B1S2 tension".into(), description: "O2 sensor bank 1 sensor 2".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x16, name: "Sonde O2 B1S3 tension".into(), description: "O2 sensor bank 1 sensor 3".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x17, name: "Sonde O2 B1S4 tension".into(), description: "O2 sensor bank 1 sensor 4".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x18, name: "Sonde O2 B2S1 tension".into(), description: "O2 sensor bank 2 sensor 1".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x19, name: "Sonde O2 B2S2 tension".into(), description: "O2 sensor bank 2 sensor 2".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x1A, name: "Sonde O2 B2S3 tension".into(), description: "O2 sensor bank 2 sensor 3".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x1B, name: "Sonde O2 B2S4 tension".into(), description: "O2 sensor bank 2 sensor 4".into(), unit: "V".into(), min: 0.0, max: 1.275, bytes: 1 },
        PidDefinition { pid: 0x1C, name: "Norme OBD".into(), description: "OBD standard this vehicle conforms to".into(), unit: "enum".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x1E, name: "Entrée auxiliaire".into(), description: "PTO status".into(), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x1F, name: "Durée fonctionnement moteur".into(), description: "Engine run time".into(), unit: "s".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x21, name: "Distance avec témoin moteur".into(), description: "Distance traveled with MIL on".into(), unit: "km".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x22, name: "Pression rampe (relative)".into(), description: "Fuel rail pressure relative to vacuum".into(), unit: "kPa".into(), min: 0.0, max: 5177.27, bytes: 2 },
        PidDefinition { pid: 0x23, name: "Pression rampe (diesel)".into(), description: "Fuel rail gauge pressure (diesel)".into(), unit: "kPa".into(), min: 0.0, max: 655350.0, bytes: 2 },
        PidDefinition { pid: 0x24, name: "Lambda sonde O2 S1".into(), description: "O2 sensor 1 equivalence ratio".into(), unit: "ratio".into(), min: 0.0, max: 2.0, bytes: 2 },
        PidDefinition { pid: 0x2C, name: "EGR commandé".into(), description: "Commanded EGR valve position".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x2D, name: "Erreur EGR".into(), description: "EGR error percentage".into(), unit: "%".into(), min: -100.0, max: 99.2, bytes: 1 },
        PidDefinition { pid: 0x2E, name: "Purge EVAP commandée".into(), description: "Commanded evaporative purge".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x2F, name: "Niveau réservoir carburant".into(), description: "Fuel tank level input".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x30, name: "Mises en chauffe depuis effacement".into(), description: "Warm-ups since codes cleared".into(), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x31, name: "Distance depuis effacement".into(), description: "Distance traveled since codes cleared".into(), unit: "km".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x32, name: "Pression vapeur EVAP".into(), description: "Evap system vapor pressure".into(), unit: "Pa".into(), min: -8192.0, max: 8191.75, bytes: 2 },
        PidDefinition { pid: 0x33, name: "Pression barométrique".into(), description: "Absolute barometric pressure".into(), unit: "kPa".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x34, name: "Courant sonde O2 S1".into(), description: "O2 sensor 1 current".into(), unit: "mA".into(), min: -128.0, max: 127.99, bytes: 4 },
        PidDefinition { pid: 0x3C, name: "Température catalyseur B1S1".into(), description: "Catalyst temperature bank 1 sensor 1".into(), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x3D, name: "Température catalyseur B2S1".into(), description: "Catalyst temperature bank 2 sensor 1".into(), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x3E, name: "Température catalyseur B1S2".into(), description: "Catalyst temperature bank 1 sensor 2".into(), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x3F, name: "Température catalyseur B2S2".into(), description: "Catalyst temperature bank 2 sensor 2".into(), unit: "°C".into(), min: -40.0, max: 6513.5, bytes: 2 },
        PidDefinition { pid: 0x42, name: "Tension module commande".into(), description: "Control module voltage".into(), unit: "V".into(), min: 0.0, max: 65.535, bytes: 2 },
        PidDefinition { pid: 0x43, name: "Charge absolue".into(), description: "Absolute load value".into(), unit: "%".into(), min: 0.0, max: 25700.0, bytes: 2 },
        PidDefinition { pid: 0x44, name: "Rapport air/carburant commandé".into(), description: "Commanded air-fuel equivalence ratio".into(), unit: "ratio".into(), min: 0.0, max: 2.0, bytes: 2 },
        PidDefinition { pid: 0x45, name: "Position papillon relative".into(), description: "Relative throttle position".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x46, name: "Température air ambiant".into(), description: "Ambient air temperature".into(), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 1 },
        PidDefinition { pid: 0x47, name: "Position papillon B".into(), description: "Absolute throttle position B".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x48, name: "Position papillon C".into(), description: "Absolute throttle position C".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x49, name: "Position pédale D".into(), description: "Accelerator pedal position D".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4A, name: "Position pédale E".into(), description: "Accelerator pedal position E".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4B, name: "Position pédale F".into(), description: "Accelerator pedal position F".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4C, name: "Papillon commandé".into(), description: "Commanded throttle actuator".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x4D, name: "Durée avec témoin moteur".into(), description: "Time run with MIL on".into(), unit: "min".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x4E, name: "Durée depuis effacement".into(), description: "Time since trouble codes cleared".into(), unit: "min".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x51, name: "Type carburant".into(), description: "Fuel type (gasoline, diesel, etc.)".into(), unit: "".into(), min: 0.0, max: 255.0, bytes: 1 },
        PidDefinition { pid: 0x52, name: "Pourcentage éthanol".into(), description: "Ethanol fuel percentage".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x5A, name: "Position pédale relative".into(), description: "Relative accelerator pedal position".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x5B, name: "Vie batterie hybride".into(), description: "Hybrid battery pack remaining life".into(), unit: "%".into(), min: 0.0, max: 100.0, bytes: 1 },
        PidDefinition { pid: 0x5C, name: "Température huile moteur".into(), description: "Engine oil temperature".into(), unit: "°C".into(), min: -40.0, max: 210.0, bytes: 1 },
        PidDefinition { pid: 0x5D, name: "Calage injection".into(), description: "Fuel injection timing".into(), unit: "°".into(), min: -210.0, max: 301.99, bytes: 2 },
        PidDefinition { pid: 0x5E, name: "Consommation carburant".into(), description: "Engine fuel rate".into(), unit: "L/h".into(), min: 0.0, max: 3212.75, bytes: 2 },
        PidDefinition { pid: 0x61, name: "Couple demandé conducteur".into(), description: "Driver's demand engine torque".into(), unit: "%".into(), min: -125.0, max: 130.0, bytes: 1 },
        PidDefinition { pid: 0x62, name: "Couple moteur réel".into(), description: "Actual engine torque".into(), unit: "%".into(), min: -125.0, max: 130.0, bytes: 1 },
        PidDefinition { pid: 0x63, name: "Couple référence moteur".into(), description: "Engine reference torque".into(), unit: "Nm".into(), min: 0.0, max: 65535.0, bytes: 2 },
        PidDefinition { pid: 0x67, name: "Température refroidissement 2".into(), description: "Engine coolant temperature sensors".into(), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 2 },
        PidDefinition { pid: 0x68, name: "Température air admission 2".into(), description: "Intake air temperature sensor".into(), unit: "°C".into(), min: -40.0, max: 215.0, bytes: 2 },
    ]
}
