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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub auto_connect: bool,
    pub theme: String,
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
