use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::models::{PidValue, DtcCode, DtcStatus, EcuInfo, MonitorStatus, Mode06Result, FreezeFrameData};
use std::collections::{HashMap, VecDeque};

/// Helper for bilingual names
fn pn<'a>(lang: &str, fr: &'a str, en: &'a str) -> &'a str {
    if lang == "fr" { fr } else { en }
}

/// Demo mode: simulates a Peugeot 207 for testing
pub struct DemoConnection {
    start_time: Instant,
    history: HashMap<u16, VecDeque<f64>>,
    lang: String,
}

impl DemoConnection {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            history: HashMap::new(),
            lang: crate::commands::connection::get_lang(),
        }
    }

    /// Update language from global setting
    pub fn refresh_lang(&mut self) {
        self.lang = crate::commands::connection::get_lang();
    }

    /// Generate simulated PID data
    pub fn get_pid_data(&mut self) -> Vec<PidValue> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let base_rpm = 850.0 + (elapsed / 3.0).sin() * 200.0 + rand_f64() * 50.0;
        let base_speed = (0.0_f64).max((elapsed / 10.0).sin() * 60.0 + 30.0 + rand_f64() * 5.0);
        let base_coolant = 85.0 + (elapsed / 20.0).sin() * 8.0 + rand_f64() * 2.0;
        let base_load = 25.0 + (elapsed / 5.0).sin() * 15.0 + rand_f64() * 5.0;

        let l = self.lang.as_str();
        let pids_data = vec![
            (0x0Cu16, pn(l, "Régime moteur", "Engine RPM"), base_rpm, "RPM", 0.0, 8000.0),
            (0x0D, pn(l, "Vitesse véhicule", "Vehicle Speed"), base_speed, "km/h", 0.0, 250.0),
            (0x05, pn(l, "Temp. liquide refroid.", "Coolant Temperature"), base_coolant, "°C", -40.0, 215.0),
            (0x04, pn(l, "Charge moteur", "Engine Load"), base_load, "%", 0.0, 100.0),
            (0x0F, pn(l, "Temp. admission", "Intake Air Temperature"), 32.0 + rand_f64() * 3.0, "°C", -40.0, 215.0),
            (0x10, pn(l, "Débit air (MAF)", "Mass Air Flow (MAF)"), 5.2 + (elapsed / 4.0).sin() * 2.0, "g/s", 0.0, 655.35),
            (0x11, pn(l, "Position papillon", "Throttle Position"), 15.0 + (elapsed / 2.0).sin() * 8.0, "%", 0.0, 100.0),
            (0x0B, pn(l, "Pression admission", "Intake Manifold Pressure"), 95.0 + (elapsed / 3.0).sin() * 5.0, "kPa", 0.0, 255.0),
            (0x0E, pn(l, "Avance allumage", "Timing Advance"), 12.0 + (elapsed / 2.5).sin() * 4.0, "°", -64.0, 63.5),
            (0x2F, pn(l, "Niveau carburant", "Fuel Tank Level"), 65.0 - (elapsed % 1000.0) / 1000.0 * 2.0, "%", 0.0, 100.0),
            (0x42, pn(l, "Tension batterie", "Battery Voltage"), 14.1 + (elapsed / 8.0).sin() * 0.3, "V", 0.0, 65.535),
            (0x46, pn(l, "Temp. ambiante", "Ambient Air Temperature"), 22.0 + rand_f64() * 1.0, "°C", -40.0, 215.0),
            (0x33, pn(l, "Pression atmos.", "Barometric Pressure"), 101.0 + rand_f64() * 0.5, "kPa", 0.0, 255.0),
            (0x06, pn(l, "Correctif carburant CT", "Short Term Fuel Trim"), 2.3 + rand_f64() * 1.0, "%", -100.0, 99.2),
            (0x07, pn(l, "Correctif carburant LT", "Long Term Fuel Trim"), 4.1 + rand_f64() * 0.5, "%", -100.0, 99.2),
            (0x1F, pn(l, "Durée moteur", "Engine Run Time"), elapsed, "s", 0.0, 65535.0),
        ];

        pids_data
            .into_iter()
            .map(|(pid, name, value, unit, min, max)| {
                let history = self.history.entry(pid).or_insert_with(VecDeque::new);
                history.push_back(value);
                if history.len() > 120 {
                    history.pop_front();
                }

                PidValue {
                    pid,
                    name: name.to_string(),
                    value,
                    unit: unit.to_string(),
                    min,
                    max,
                    history: history.iter().cloned().collect(),
                    timestamp: now,
                }
            })
            .collect()
    }

    /// Get demo DTCs
    pub fn get_dtcs(lang: &str) -> Vec<DtcCode> {
        let codes = vec![("P0440", DtcStatus::Active, "OBD Mode 03"), ("P0500", DtcStatus::Pending, "OBD Mode 07")];
        codes.into_iter().map(|(code, status, source)| {
            let description = crate::obd::dtc::get_dtc_description(code, lang);
            let repair_tips = crate::obd::dtc::get_dtc_repair_tips(code, lang);
            let (causes, quick_check, difficulty) = crate::obd::dtc::get_dtc_repair_data(code, lang);
            DtcCode {
                code: code.to_string(),
                description,
                status,
                source: source.to_string(),
                repair_tips,
                causes,
                quick_check,
                difficulty,
                ecu_context: Some("Engine (ECM)".to_string()),
            }
        }).collect()
    }

    /// Get demo ECU list (bilingual)
    pub fn get_ecus(lang: &str) -> Vec<EcuInfo> {
        vec![
            EcuInfo {
                name: pn(lang, "Moteur (ECM)", "Engine (ECM)").to_string(),
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
                name: pn(lang, "BSI (Boîtier Servitudes Intelligent)", "BSI (Body Systems Interface)").to_string(),
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
                name: pn(lang, "Climatisation (HVAC)", "HVAC").to_string(),
                address: "0x7E6".to_string(),
                protocol: "ISO 15765-4 CAN".to_string(),
                dids: HashMap::from([
                    ("F190".to_string(), "VF3LCBHZ6JS000000".to_string()),
                    ("F195".to_string(), "HVAC 1.0".to_string()),
                ]),
            },
            EcuInfo {
                name: pn(lang, "Tableau de bord", "Instrument Cluster").to_string(),
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

    /// Get demo Mode 06 test results
    pub fn get_mode06_results(lang: &str) -> Vec<Mode06Result> {
        let pn = |fr: &str, en: &str| -> String {
            if lang == "fr" { fr.to_string() } else { en.to_string() }
        };
        vec![
            Mode06Result { tid: 0x01, mid: 0x00, name: pn("Raté d'allumage Cyl.1", "Misfire Cylinder 1"), unit: "count".into(), test_value: 0.0, min_limit: 0.0, max_limit: 5.0, passed: true },
            Mode06Result { tid: 0x02, mid: 0x00, name: pn("Raté d'allumage Cyl.2", "Misfire Cylinder 2"), unit: "count".into(), test_value: 1.0, min_limit: 0.0, max_limit: 5.0, passed: true },
            Mode06Result { tid: 0x03, mid: 0x00, name: pn("Raté d'allumage Cyl.3", "Misfire Cylinder 3"), unit: "count".into(), test_value: 0.0, min_limit: 0.0, max_limit: 5.0, passed: true },
            Mode06Result { tid: 0x04, mid: 0x00, name: pn("Raté d'allumage Cyl.4", "Misfire Cylinder 4"), unit: "count".into(), test_value: 0.0, min_limit: 0.0, max_limit: 5.0, passed: true },
            Mode06Result { tid: 0x21, mid: 0x01, name: pn("Efficacité catalyseur B1", "Catalyst Efficiency B1"), unit: "ratio".into(), test_value: 0.82, min_limit: 0.70, max_limit: 1.00, passed: true },
            Mode06Result { tid: 0x29, mid: 0x11, name: pn("Temps réponse O2 B1S1", "O2 Response Time B1S1"), unit: "ms".into(), test_value: 120.0, min_limit: 0.0, max_limit: 100.0, passed: false },
            Mode06Result { tid: 0x31, mid: 0x00, name: pn("Fuite EVAP (grande)", "EVAP Large Leak"), unit: "Pa".into(), test_value: 12.0, min_limit: 0.0, max_limit: 50.0, passed: true },
            Mode06Result { tid: 0x35, mid: 0x00, name: pn("Purge EVAP", "EVAP Purge Flow"), unit: "%".into(), test_value: 95.0, min_limit: 75.0, max_limit: 100.0, passed: true },
            Mode06Result { tid: 0x3C, mid: 0x00, name: pn("Débit EGR", "EGR Flow Rate"), unit: "%".into(), test_value: 88.0, min_limit: 60.0, max_limit: 100.0, passed: true },
        ]
    }

    /// Get demo Mode 02 Freeze Frame data
    pub fn get_freeze_frame(lang: &str) -> Option<FreezeFrameData> {
        let pn = |fr: &str, en: &str| -> String {
            if lang == "fr" { fr.to_string() } else { en.to_string() }
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let pids = vec![
            PidValue { pid: 0x0C, name: pn("Régime moteur", "Engine RPM"), value: 850.0, unit: "RPM".into(), min: 0.0, max: 8000.0, history: vec![], timestamp: now },
            PidValue { pid: 0x0D, name: pn("Vitesse véhicule", "Vehicle Speed"), value: 0.0, unit: "km/h".into(), min: 0.0, max: 250.0, history: vec![], timestamp: now },
            PidValue { pid: 0x05, name: pn("Temp. liquide refroid.", "Coolant Temperature"), value: 90.0, unit: "°C".into(), min: -40.0, max: 215.0, history: vec![], timestamp: now },
            PidValue { pid: 0x04, name: pn("Charge moteur", "Engine Load"), value: 22.0, unit: "%".into(), min: 0.0, max: 100.0, history: vec![], timestamp: now },
            PidValue { pid: 0x2F, name: pn("Niveau carburant", "Fuel Tank Level"), value: 65.0, unit: "%".into(), min: 0.0, max: 100.0, history: vec![], timestamp: now },
            PidValue { pid: 0x11, name: pn("Position papillon", "Throttle Position"), value: 14.0, unit: "%".into(), min: 0.0, max: 100.0, history: vec![], timestamp: now },
            PidValue { pid: 0x06, name: pn("Correctif carburant CT", "Short Term Fuel Trim"), value: 3.1, unit: "%".into(), min: -100.0, max: 99.2, history: vec![], timestamp: now },
            PidValue { pid: 0x07, name: pn("Correctif carburant LT", "Long Term Fuel Trim"), value: 5.2, unit: "%".into(), min: -100.0, max: 99.2, history: vec![], timestamp: now },
        ];
        Some(FreezeFrameData {
            dtc_code: "P0440".into(),
            frame_number: 0,
            pids,
        })
    }
}

/// xorshift64 PRNG state (seeded once on first call)
static PRNG_STATE: AtomicU64 = AtomicU64::new(0);

/// Simple xorshift64 PRNG returning f64 in [0.0, 1.0)
fn rand_f64() -> f64 {
    let mut state = PRNG_STATE.load(Ordering::Relaxed);
    if state == 0 {
        // Seed from system time on first call
        state = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);
        if state == 0 { state = 1; }
    }
    // xorshift64 iteration
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    PRNG_STATE.store(state, Ordering::Relaxed);
    ((state >> 11) as f64) * (1.0 / 9007199254740992.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dtcs_en() {
        let dtcs = DemoConnection::get_dtcs("en");
        assert_eq!(dtcs.len(), 2);
        assert_eq!(dtcs[0].code, "P0440");
        assert_eq!(dtcs[0].status, DtcStatus::Active);
        assert_eq!(dtcs[1].code, "P0500");
        assert_eq!(dtcs[1].status, DtcStatus::Pending);
    }

    #[test]
    fn test_get_dtcs_fr() {
        let dtcs = DemoConnection::get_dtcs("fr");
        assert_eq!(dtcs.len(), 2);
        assert_eq!(dtcs[0].code, "P0440");
        assert_eq!(dtcs[1].code, "P0500");
    }

    #[test]
    fn test_get_dtcs_have_descriptions() {
        let dtcs = DemoConnection::get_dtcs("en");
        for dtc in dtcs {
            assert!(!dtc.description.is_empty());
        }
    }

    #[test]
    fn test_get_ecus_en() {
        let ecus = DemoConnection::get_ecus("en");
        assert_eq!(ecus.len(), 7);
        assert_eq!(ecus[0].name, "Engine (ECM)");
        assert_eq!(ecus[0].address, "0x7E0");
    }

    #[test]
    fn test_get_ecus_fr() {
        let ecus = DemoConnection::get_ecus("fr");
        assert_eq!(ecus.len(), 7);
        assert_eq!(ecus[0].name, "Moteur (ECM)");
    }

    #[test]
    fn test_get_ecus_have_protocol() {
        let ecus = DemoConnection::get_ecus("en");
        for ecu in ecus {
            assert!(!ecu.protocol.is_empty());
            assert!(!ecu.address.is_empty());
        }
    }

    #[test]
    fn test_get_monitors() {
        let monitors = DemoConnection::get_monitors();
        assert_eq!(monitors.len(), 11);
        assert!(!monitors[0].name_key.is_empty());
    }

    #[test]
    fn test_get_monitors_completeness() {
        let monitors = DemoConnection::get_monitors();
        let available_count = monitors.iter().filter(|m| m.available).count();
        assert!(available_count > 0);
        let complete_count = monitors.iter().filter(|m| m.complete).count();
        assert!(complete_count > 0);
    }

    #[test]
    fn test_get_mode06_results_en() {
        let results = DemoConnection::get_mode06_results("en");
        assert_eq!(results.len(), 9);
        assert!(!results[0].name.is_empty());
    }

    #[test]
    fn test_get_mode06_results_fr() {
        let results = DemoConnection::get_mode06_results("fr");
        assert_eq!(results.len(), 9);
    }

    #[test]
    fn test_get_mode06_results_have_limits() {
        let results = DemoConnection::get_mode06_results("en");
        for result in results {
            assert!(result.min_limit <= result.max_limit);
        }
    }

    #[test]
    fn test_get_freeze_frame_en() {
        let frame = DemoConnection::get_freeze_frame("en");
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.dtc_code, "P0440");
        assert!(!frame.pids.is_empty());
    }

    #[test]
    fn test_get_freeze_frame_fr() {
        let frame = DemoConnection::get_freeze_frame("fr");
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.dtc_code, "P0440");
    }

    #[test]
    fn test_get_freeze_frame_has_pids() {
        let frame = DemoConnection::get_freeze_frame("en");
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert!(frame.pids.len() > 0);
        for pid in frame.pids {
            assert!(pid.pid > 0);
            assert!(!pid.name.is_empty());
            assert!(!pid.unit.is_empty());
        }
    }

    #[test]
    fn test_demo_connection_new() {
        let conn = DemoConnection::new();
        assert!(!conn.lang.is_empty());
    }

    #[test]
    fn test_demo_connection_get_pid_data() {
        let mut conn = DemoConnection::new();
        let pids = conn.get_pid_data();
        assert!(!pids.is_empty());
        for pid in pids {
            assert!(pid.pid > 0);
            assert!(!pid.name.is_empty());
            assert!(!pid.unit.is_empty());
            assert!(pid.min <= pid.max);
        }
    }
}
