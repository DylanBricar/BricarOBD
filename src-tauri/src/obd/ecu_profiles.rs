use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

const ECU_DATABASE_JSON: &str = include_str!("../../data/ecu_database.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericEcu {
    pub name: String,
    pub request_id: u16,
    pub response_id: u16,
    pub protocol: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuDatabaseRoot {
    pub dids: HashMap<String, HashMap<String, String>>,
    pub generic_ecus: Vec<GenericEcu>,
    pub manufacturer_ecus: HashMap<String, serde_json::Value>,
    pub vehicle_profiles: HashMap<String, serde_json::Value>,
    pub maps: EcuMaps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuMaps {
    #[serde(rename = "MANUFACTURER_ECU_MAP")]
    pub manufacturer_ecu_map: HashMap<String, serde_json::Value>,
    #[serde(rename = "MANUFACTURER_DID_MAP")]
    pub manufacturer_did_map: HashMap<String, serde_json::Value>,
}

static ECU_DATABASE: LazyLock<EcuDatabaseRoot> = LazyLock::new(|| {
    serde_json::from_str(ECU_DATABASE_JSON).unwrap_or_else(|e| {
        tracing::error!("Failed to parse ECU database JSON: {}", e);
        EcuDatabaseRoot {
            dids: HashMap::new(),
            generic_ecus: Vec::new(),
            manufacturer_ecus: HashMap::new(),
            vehicle_profiles: HashMap::new(),
            maps: EcuMaps {
                manufacturer_ecu_map: HashMap::new(),
                manufacturer_did_map: HashMap::new(),
            },
        }
    })
});

pub fn get_generic_ecus() -> Vec<GenericEcu> {
    ECU_DATABASE.generic_ecus.clone()
}

/// Map VIN-decoded make names to DID group keys in the database
fn manufacturer_to_did_key(manufacturer: &str) -> &'static str {
    match manufacturer.to_uppercase().as_str() {
        "PEUGEOT" | "CITROËN" | "CITROEN" | "DS" | "DS AUTOMOBILES" | "OPEL" | "VAUXHALL" => "PSA_EXTENDED_DIDS",
        "VOLKSWAGEN" | "VW" | "AUDI" | "ŠKODA" | "SKODA" | "SEAT" | "PORSCHE" => "VAG_EXTENDED_DIDS",
        "RENAULT" | "DACIA" => "RENAULT_EXTENDED_DIDS",
        "BMW" | "BMW M" | "MINI" => "BMW_EXTENDED_DIDS",
        "MERCEDES-BENZ" | "MERCEDES" => "MERCEDES_EXTENDED_DIDS",
        "TOYOTA" | "LEXUS" => "TOYOTA_EXTENDED_DIDS",
        "HONDA" | "ACURA" => "HONDA_EXTENDED_DIDS",
        "HYUNDAI" | "KIA" | "GENESIS" => "HYUNDAI_KIA_EXTENDED_DIDS",
        "FIAT" | "ALFA ROMEO" | "LANCIA" | "ABARTH" | "MASERATI" => "FIAT_EXTENDED_DIDS",
        "FORD" | "LINCOLN" => "FORD_EXTENDED_DIDS",
        "MAZDA" => "MAZDA_EXTENDED_DIDS",
        "SUBARU" => "SUBARU_EXTENDED_DIDS",
        "VOLVO" => "VOLVO_EXTENDED_DIDS",
        _ => "",
    }
}

pub fn get_dids_for_manufacturer(manufacturer: &str) -> Vec<(String, String)> {
    let dids_map = &ECU_DATABASE.dids;
    let key = manufacturer_to_did_key(manufacturer);

    if key.is_empty() {
        // Fallback: try string-contains search
        return dids_map
            .iter()
            .find(|(k, _)| k.to_uppercase().contains(&manufacturer.to_uppercase().replace(" ", "_")))
            .map(|(_, dids)| dids.iter().map(|(id, desc)| (id.clone(), desc.clone())).collect())
            .unwrap_or_default();
    }

    dids_map
        .get(key)
        .map(|dids| dids.iter().map(|(id, desc)| (id.clone(), desc.clone())).collect())
        .unwrap_or_default()
}

pub fn get_all_manufacturer_dids() -> HashMap<String, Vec<(String, String)>> {
    let mut result = HashMap::new();

    for (key, dids) in &ECU_DATABASE.dids {
        let dids_vec = dids
            .iter()
            .map(|(id, desc)| (id.clone(), desc.clone()))
            .collect();
        result.insert(key.clone(), dids_vec);
    }

    result
}

pub fn get_manufacturer_ecu_map() -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (make, value) in &ECU_DATABASE.maps.manufacturer_ecu_map {
        result.insert(make.clone(), format!("{:?}", value));
    }

    result
}

pub fn get_manufacturer_did_map() -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (make, value) in &ECU_DATABASE.maps.manufacturer_did_map {
        result.insert(make.clone(), format!("{:?}", value));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_database() {
        let generic_ecus = get_generic_ecus();
        assert!(!generic_ecus.is_empty());
    }

    #[test]
    fn test_get_all_manufacturer_dids() {
        let all_dids = get_all_manufacturer_dids();
        assert!(!all_dids.is_empty());
    }

    #[test]
    fn test_get_manufacturer_ecu_map() {
        let ecu_map = get_manufacturer_ecu_map();
        assert!(!ecu_map.is_empty());
    }

    #[test]
    fn test_get_manufacturer_did_map() {
        let did_map = get_manufacturer_did_map();
        assert!(!did_map.is_empty());
    }
}
