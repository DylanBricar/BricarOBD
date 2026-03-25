use std::time::Instant;
use crate::models::{PidValue, DtcCode, DtcStatus, EcuInfo, MonitorStatus};
use std::collections::HashMap;

/// Demo mode: simulates a Peugeot 207 for testing
pub struct DemoConnection {
    start_time: Instant,
    history: HashMap<u16, Vec<f64>>,
}

impl DemoConnection {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            history: HashMap::new(),
        }
    }

    /// Generate simulated PID data
    pub fn get_pid_data(&mut self) -> Vec<PidValue> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let base_rpm = 850.0 + (elapsed / 3.0).sin() * 200.0 + rand_f64() * 50.0;
        let base_speed = (0.0_f64).max((elapsed / 10.0).sin() * 60.0 + 30.0 + rand_f64() * 5.0);
        let base_coolant = 85.0 + (elapsed / 20.0).sin() * 8.0 + rand_f64() * 2.0;
        let base_load = 25.0 + (elapsed / 5.0).sin() * 15.0 + rand_f64() * 5.0;

        let pids_data = vec![
            (0x0Cu16, "RPM", base_rpm, "RPM", 0.0, 8000.0),
            (0x0D, "Vitesse", base_speed, "km/h", 0.0, 250.0),
            (0x05, "Temp. liquide refroid.", base_coolant, "°C", -40.0, 215.0),
            (0x04, "Charge moteur", base_load, "%", 0.0, 100.0),
            (0x0F, "Temp. admission", 32.0 + rand_f64() * 3.0, "°C", -40.0, 215.0),
            (0x10, "Débit air (MAF)", 5.2 + (elapsed / 4.0).sin() * 2.0, "g/s", 0.0, 655.35),
            (0x11, "Position papillon", 15.0 + (elapsed / 2.0).sin() * 8.0, "%", 0.0, 100.0),
            (0x0B, "Pression admission", 95.0 + (elapsed / 3.0).sin() * 5.0, "kPa", 0.0, 255.0),
            (0x0E, "Avance allumage", 12.0 + (elapsed / 2.5).sin() * 4.0, "°", -64.0, 63.5),
            (0x2F, "Niveau carburant", 65.0 - (elapsed % 1000.0) / 1000.0 * 2.0, "%", 0.0, 100.0),
            (0x42, "Tension batterie", 14.1 + (elapsed / 8.0).sin() * 0.3, "V", 0.0, 65.535),
            (0x46, "Temp. ambiante", 22.0 + rand_f64() * 1.0, "°C", -40.0, 215.0),
            (0x33, "Pression atmos.", 101.0 + rand_f64() * 0.5, "kPa", 0.0, 255.0),
            (0x06, "Correctif carburant CT", 2.3 + rand_f64() * 1.0, "%", -100.0, 99.2),
            (0x07, "Correctif carburant LT", 4.1 + rand_f64() * 0.5, "%", -100.0, 99.2),
            (0x1F, "Durée moteur", elapsed, "s", 0.0, 65535.0),
        ];

        pids_data
            .into_iter()
            .map(|(pid, name, value, unit, min, max)| {
                let history = self.history.entry(pid).or_insert_with(Vec::new);
                history.push(value);
                if history.len() > 60 {
                    history.remove(0);
                }

                PidValue {
                    pid,
                    name: name.to_string(),
                    value,
                    unit: unit.to_string(),
                    min,
                    max,
                    history: history.clone(),
                    timestamp: now,
                }
            })
            .collect()
    }

    /// Get demo DTCs
    pub fn get_dtcs() -> Vec<DtcCode> {
        vec![
            DtcCode {
                code: "P0440".to_string(),
                description: "Système de contrôle des émissions par évaporation - Dysfonctionnement".to_string(),
                status: DtcStatus::Active,
                source: "OBD Mode 03".to_string(),
                repair_tips: Some("Vérifier le bouchon de réservoir, les durites EVAP et la valve de purge.".to_string()),
            },
            DtcCode {
                code: "P0500".to_string(),
                description: "Capteur de vitesse du véhicule - Dysfonctionnement".to_string(),
                status: DtcStatus::Pending,
                source: "OBD Mode 07".to_string(),
                repair_tips: Some("Inspecter le capteur VSS, le câblage et les connecteurs.".to_string()),
            },
        ]
    }

    /// Get demo ECU list
    pub fn get_ecus() -> Vec<EcuInfo> {
        vec![
            EcuInfo {
                name: "Moteur (ECM)".to_string(),
                address: "0x7E0".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                    ("F194".to_string(), "EP6DT".to_string()),
                    ("F195".to_string(), "1.6 THP 150".to_string()),
                ]),
            },
            EcuInfo {
                name: "Transmission (TCM)".to_string(),
                address: "0x7E1".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                    ("F191".to_string(), "AL4/DP0".to_string()),
                ]),
            },
            EcuInfo {
                name: "ABS/ESP".to_string(),
                address: "0x7E2".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                ]),
            },
            EcuInfo {
                name: "Airbag (SRS)".to_string(),
                address: "0x7E3".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::new(),
            },
            EcuInfo {
                name: "BSI (Boîtier Servitudes)".to_string(),
                address: "0x75D".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                    ("F18C".to_string(), "BSI 2010".to_string()),
                    ("F191".to_string(), "BSI HW 1.5".to_string()),
                    ("F195".to_string(), "BSI SW 6.2".to_string()),
                ]),
            },
            EcuInfo {
                name: "Climatisation (HVAC)".to_string(),
                address: "0x7E6".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                    ("F195".to_string(), "HVAC 1.0".to_string()),
                ]),
            },
            EcuInfo {
                name: "Tableau de bord".to_string(),
                address: "0x7E5".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                    ("F195".to_string(), "CLUST 2.3".to_string()),
                    ("F18C".to_string(), "2018-03-01".to_string()),
                ]),
            },
        ]
    }

    /// Get demo monitor statuses
    pub fn get_monitors() -> Vec<MonitorStatus> {
        vec![
            MonitorStatus { name_key: "monitors.misfire".into(), available: true, complete: true, description_key: Some("monitors.misfireDesc".into()), specification_key: Some("monitors.misfireSpec".into()) },
            MonitorStatus { name_key: "monitors.fuelSystem".into(), available: true, complete: true, description_key: Some("monitors.fuelSystemDesc".into()), specification_key: Some("monitors.fuelSystemSpec".into()) },
            MonitorStatus { name_key: "monitors.components".into(), available: true, complete: true, description_key: Some("monitors.componentsDesc".into()), specification_key: Some("monitors.componentsSpec".into()) },
            MonitorStatus { name_key: "monitors.catalystB1".into(), available: true, complete: false, description_key: Some("monitors.catalystB1Desc".into()), specification_key: Some("monitors.catalystB1Spec".into()) },
            MonitorStatus { name_key: "monitors.catalystB2".into(), available: false, complete: false, description_key: Some("monitors.catalystB2Desc".into()), specification_key: Some("monitors.catalystB2Spec".into()) },
            MonitorStatus { name_key: "monitors.evap".into(), available: true, complete: false, description_key: Some("monitors.evapDesc".into()), specification_key: Some("monitors.evapSpec".into()) },
            MonitorStatus { name_key: "monitors.o2B1S1".into(), available: true, complete: true, description_key: Some("monitors.o2B1S1Desc".into()), specification_key: Some("monitors.o2B1S1Spec".into()) },
            MonitorStatus { name_key: "monitors.o2HeaterB1S1".into(), available: true, complete: true, description_key: Some("monitors.o2HeaterB1S1Desc".into()), specification_key: Some("monitors.o2HeaterB1S1Spec".into()) },
            MonitorStatus { name_key: "monitors.secondaryAir".into(), available: false, complete: false, description_key: Some("monitors.secondaryAirDesc".into()), specification_key: Some("monitors.secondaryAirSpec".into()) },
            MonitorStatus { name_key: "monitors.ac".into(), available: false, complete: false, description_key: Some("monitors.acDesc".into()), specification_key: Some("monitors.acSpec".into()) },
            MonitorStatus { name_key: "monitors.egrVvt".into(), available: true, complete: true, description_key: Some("monitors.egrVvtDesc".into()), specification_key: Some("monitors.egrVvtSpec".into()) },
        ]
    }
}

/// Simple pseudo-random (no external crate needed)
fn rand_f64() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}
