/// Database result models extracted from database.rs
/// Provides shared struct definitions for operation and session queries

use serde::{Deserialize, Serialize};

/// Result of a single operation query (read/write/diagnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub id: String,
    pub name: String,
    pub name_fr: String,
    pub sentbytes: String,
    pub service: String,
    pub did: String,
    #[serde(rename = "type")]
    pub op_type: String,
    pub ecu_name: String,
    pub ecu_tx: String,
    pub ecu_rx: String,
    pub vehicle: String,
    pub risk: String,
}

/// Result of an operation query without vehicle info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResultNoVehicle {
    pub id: String,
    pub name: String,
    pub name_fr: String,
    pub sentbytes: String,
    pub service: String,
    pub did: String,
    #[serde(rename = "type")]
    pub op_type: String,
    pub ecu_name: String,
    pub ecu_tx: String,
    pub ecu_rx: String,
    pub risk: String,
}

/// Read operation specific result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadOperation {
    pub id: String,
    pub name: String,
    pub name_fr: String,
    pub sentbytes: String,
    pub did: String,
    pub ecu_tx: String,
    pub ecu_rx: String,
}

/// Write operation specific result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteOperation {
    pub id: String,
    pub name: String,
    pub name_fr: String,
    pub sentbytes: String,
    pub did: String,
    pub ecu_tx: String,
    pub ecu_rx: String,
    pub risk: String,
}

/// ECU profile information from vehicle_profiles table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcuProfile {
    pub name: String,
    pub name_fr: String,
    pub tx: String,
    pub rx: String,
}

/// ECU catalog entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcuCatalogResult {
    pub filename: String,
    pub ecuname: String,
    pub address: String,
    pub group: String,
    pub protocol: String,
    pub projects: String,
}

/// Session entry for diagnostics history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResult {
    pub id: i64,
    pub vin: String,
    pub make: String,
    pub model: String,
    pub dtc_count: i32,
    pub notes: String,
    pub timestamp: String,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStats {
    pub operations: u64,
    pub profiles: u64,
    pub ecus: u64,
}
