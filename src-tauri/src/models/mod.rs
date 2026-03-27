use serde::{Deserialize, Serialize};

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Demo,
    Error,
}

/// Vehicle information from VIN + OBD
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleInfo {
    pub vin: String,
    pub make: String,
    pub model: String,
    pub year: u16,
    pub protocol: String,
    pub elm_version: String,
}

/// PID value with history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PidValue {
    pub pid: u16,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub history: Vec<f64>,
    pub timestamp: u64,
}

/// DTC code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtcCode {
    pub code: String,
    pub description: String,
    pub status: DtcStatus,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair_tips: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecu_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DtcStatus {
    Active,
    Pending,
    Permanent,
}

/// ECU information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcuInfo {
    pub name: String,
    pub address: String,
    pub protocol: String,
    pub dids: std::collections::HashMap<String, String>,
}

/// Monitor status — uses i18n keys for frontend translation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorStatus {
    pub name_key: String,
    pub available: bool,
    pub complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specification_key: Option<String>,
}

/// Serial port info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub name: String,
    pub description: String,
}

/// ECU database definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcuDefinition {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub address: String,
    pub protocol: String,
    pub param_count: u32,
    pub request_count: u32,
}

/// App settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub default_baud_rate: u32,
    #[serde(default)]
    pub auto_connect: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "fr".to_string(),
            default_baud_rate: 38400,
            auto_connect: false,
            theme: "dark".to_string(),
        }
    }
}

/// OBD-II PID definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PidDefinition {
    pub pid: u16,
    pub name: String,
    pub description: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub bytes: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "fr");
        assert_eq!(settings.default_baud_rate, 38400);
        assert_eq!(settings.auto_connect, false);
        assert_eq!(settings.theme, "dark");
    }

    #[test]
    fn test_connection_status_disconnected_serialize() {
        let status = ConnectionStatus::Disconnected;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"disconnected\"");
    }

    #[test]
    fn test_connection_status_connecting_serialize() {
        let status = ConnectionStatus::Connecting;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"connecting\"");
    }

    #[test]
    fn test_connection_status_connected_serialize() {
        let status = ConnectionStatus::Connected;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"connected\"");
    }

    #[test]
    fn test_connection_status_demo_serialize() {
        let status = ConnectionStatus::Demo;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"demo\"");
    }

    #[test]
    fn test_connection_status_error_serialize() {
        let status = ConnectionStatus::Error;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"error\"");
    }

    #[test]
    fn test_dtc_status_active_serialize() {
        let status = DtcStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_dtc_status_pending_serialize() {
        let status = DtcStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"pending\"");
    }

    #[test]
    fn test_dtc_status_permanent_serialize() {
        let status = DtcStatus::Permanent;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"permanent\"");
    }

    #[test]
    fn test_risk_level_safe_serialize() {
        let level = RiskLevel::Safe;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"safe\"");
    }

    #[test]
    fn test_risk_level_caution_serialize() {
        let level = RiskLevel::Caution;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"caution\"");
    }

    #[test]
    fn test_risk_level_dangerous_serialize() {
        let level = RiskLevel::Dangerous;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"dangerous\"");
    }

    #[test]
    fn test_risk_level_blocked_serialize() {
        let level = RiskLevel::Blocked;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"blocked\"");
    }

    #[test]
    fn test_vehicle_info_serialize() {
        let vehicle = VehicleInfo {
            vin: "VF3LCBHZ6JS000000".to_string(),
            make: "Peugeot".to_string(),
            model: "207".to_string(),
            year: 2010,
            protocol: "ISO 15765-4 CAN 11-bit 500k".to_string(),
            elm_version: "1.5".to_string(),
        };
        let json = serde_json::to_string(&vehicle).unwrap();
        let deserialized: VehicleInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.vin, vehicle.vin);
        assert_eq!(deserialized.make, vehicle.make);
        assert_eq!(deserialized.year, vehicle.year);
    }

    #[test]
    fn test_pid_value_serialize() {
        let pid = PidValue {
            pid: 0x0C,
            name: "Engine RPM".to_string(),
            value: 3000.0,
            unit: "RPM".to_string(),
            min: 0.0,
            max: 8000.0,
            history: vec![2500.0, 2750.0, 3000.0],
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&pid).unwrap();
        let deserialized: PidValue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pid, 0x0C);
        assert_eq!(deserialized.value, 3000.0);
        assert_eq!(deserialized.history.len(), 3);
    }

    #[test]
    fn test_dtc_code_serialize() {
        let dtc = DtcCode {
            code: "P0440".to_string(),
            description: "Evaporative system leak".to_string(),
            status: DtcStatus::Active,
            source: "OBD Mode 03".to_string(),
            repair_tips: Some("Check fuel cap".to_string()),
            causes: Some(vec!["Faulty valve".to_string()]),
            quick_check: Some("Visual inspection".to_string()),
            difficulty: Some(2),
            ecu_context: Some("Engine (ECM)".to_string()),
        };
        let json = serde_json::to_string(&dtc).unwrap();
        let deserialized: DtcCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, "P0440");
        assert_eq!(deserialized.status, DtcStatus::Active);
        assert!(deserialized.repair_tips.is_some());
    }

    #[test]
    fn test_monitor_status_serialize() {
        let monitor = MonitorStatus {
            name_key: "monitors.misfire".to_string(),
            available: true,
            complete: true,
            description_key: Some("monitors.misfireDesc".to_string()),
            specification_key: Some("monitors.misfireSpec".to_string()),
        };
        let json = serde_json::to_string(&monitor).unwrap();
        let deserialized: MonitorStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name_key, "monitors.misfire");
        assert!(deserialized.available);
    }

    #[test]
    fn test_mode06_result_serialize() {
        let result = Mode06Result {
            tid: 0x01,
            mid: 0x00,
            name: "Misfire Cylinder 1".to_string(),
            unit: "count".to_string(),
            test_value: 0.0,
            min_limit: 0.0,
            max_limit: 5.0,
            passed: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: Mode06Result = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tid, 0x01);
        assert!(deserialized.passed);
    }

    #[test]
    fn test_freeze_frame_data_serialize() {
        let frame = FreezeFrameData {
            dtc_code: "P0440".to_string(),
            frame_number: 0,
            pids: vec![PidValue {
                pid: 0x0C,
                name: "Engine RPM".to_string(),
                value: 850.0,
                unit: "RPM".to_string(),
                min: 0.0,
                max: 8000.0,
                history: vec![],
                timestamp: 1234567890,
            }],
        };
        let json = serde_json::to_string(&frame).unwrap();
        let deserialized: FreezeFrameData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.dtc_code, "P0440");
        assert_eq!(deserialized.pids.len(), 1);
    }

    #[test]
    fn test_app_settings_serialize_deserialize() {
        let settings = AppSettings {
            language: "en".to_string(),
            default_baud_rate: 57600,
            auto_connect: true,
            theme: "light".to_string(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.language, "en");
        assert_eq!(deserialized.default_baud_rate, 57600);
        assert_eq!(deserialized.auto_connect, true);
        assert_eq!(deserialized.theme, "light");
    }
}

/// Safety risk level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Caution,
    Dangerous,
    Blocked,
}

/// Mode 06 Test Result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mode06Result {
    pub tid: u8,
    pub mid: u8,
    pub name: String,
    pub unit: String,
    pub test_value: f64,
    pub min_limit: f64,
    pub max_limit: f64,
    pub passed: bool,
}

/// Mode 02 Freeze Frame Data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreezeFrameData {
    pub dtc_code: String,
    pub frame_number: u8,
    pub pids: Vec<PidValue>,
}

/// Vehicle information extended (CalID, CVN, ECU Name)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleInfoExtended {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecu_name: Option<String>,
}
