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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_result_serialize_camel_case() {
        let op = OperationResult {
            id: "1".into(), name: "Read VIN".into(), name_fr: "Lire VIN".into(),
            sentbytes: "22F190".into(), service: "22".into(), did: "F190".into(),
            op_type: "read".into(), ecu_name: "Engine".into(), ecu_tx: "7E0".into(),
            ecu_rx: "7E8".into(), vehicle: "Peugeot 207".into(), risk: "low".into(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"nameFr\""), "Should use camelCase: nameFr");
        assert!(json.contains("\"ecuName\""), "Should use camelCase: ecuName");
        assert!(json.contains("\"ecuTx\""), "Should use camelCase: ecuTx");
        assert!(json.contains("\"type\""), "op_type should serialize as 'type'");
        assert!(!json.contains("\"op_type\""), "Should not contain snake_case op_type");
    }

    #[test]
    fn test_operation_result_deserialize() {
        let json = r#"{"id":"1","name":"Read VIN","nameFr":"Lire VIN","sentbytes":"22F190","service":"22","did":"F190","type":"read","ecuName":"Engine","ecuTx":"7E0","ecuRx":"7E8","vehicle":"207","risk":"low"}"#;
        let op: OperationResult = serde_json::from_str(json).unwrap();
        assert_eq!(op.op_type, "read");
        assert_eq!(op.ecu_name, "Engine");
    }

    #[test]
    fn test_read_operation_serialize() {
        let op = ReadOperation {
            id: "1".into(), name: "Read VIN".into(), name_fr: "Lire VIN".into(),
            sentbytes: "22F190".into(), did: "F190".into(), ecu_tx: "7E0".into(), ecu_rx: "7E8".into(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"nameFr\""));
        assert!(json.contains("\"ecuTx\""));
    }

    #[test]
    fn test_write_operation_has_risk() {
        let op = WriteOperation {
            id: "1".into(), name: "Write VIN".into(), name_fr: "Écrire VIN".into(),
            sentbytes: "2EF190".into(), did: "F190".into(), ecu_tx: "7E0".into(), ecu_rx: "7E8".into(), risk: "high".into(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"risk\":\"high\""));
    }

    #[test]
    fn test_ecu_profile_serialize() {
        let profile = EcuProfile { name: "Engine".into(), name_fr: "Moteur".into(), tx: "7E0".into(), rx: "7E8".into() };
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("\"nameFr\""));
    }

    #[test]
    fn test_ecu_catalog_result_serialize() {
        let result = EcuCatalogResult {
            filename: "ecm.xml".into(), ecuname: "ECM".into(), address: "7E0".into(),
            group: "Engine".into(), protocol: "CAN".into(), projects: "207,208".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ecuname\""));
    }

    #[test]
    fn test_session_result_serialize() {
        let session = SessionResult {
            id: 1, vin: "VF3LCBHZ6JS000000".into(), make: "Peugeot".into(),
            model: "207".into(), dtc_count: 3, notes: "".into(), timestamp: "2024-01-01".into(),
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("\"dtcCount\":3"));
    }

    #[test]
    fn test_db_stats_serialize() {
        let stats = DbStats { operations: 1000, profiles: 50, ecus: 200 };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"operations\":1000"));
        assert!(json.contains("\"profiles\":50"));
    }

    #[test]
    fn test_operation_result_no_vehicle_serialize() {
        let op = OperationResultNoVehicle {
            id: "1".into(), name: "Test".into(), name_fr: "Test".into(),
            sentbytes: "010C".into(), service: "01".into(), did: "0C".into(),
            op_type: "read".into(), ecu_name: "ECM".into(), ecu_tx: "7E0".into(),
            ecu_rx: "7E8".into(), risk: "low".into(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(!json.contains("vehicle"), "Should not have vehicle field");
        assert!(json.contains("\"type\":\"read\""));
    }

    #[test]
    fn test_roundtrip_db_stats() {
        let stats = DbStats { operations: 3170000, profiles: 90, ecus: 4866 };
        let json = serde_json::to_string(&stats).unwrap();
        let parsed: DbStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.operations, 3170000);
        assert_eq!(parsed.profiles, 90);
        assert_eq!(parsed.ecus, 4866);
    }
}
