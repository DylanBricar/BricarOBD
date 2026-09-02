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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidDefinition {
    pub id: u16,
    pub name: String,
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

pub fn get_generic_ecus() -> &'static [GenericEcu] {
    &ECU_DATABASE.generic_ecus
}

/// The embedded catalog stores DID object keys as base-10 integers.
/// Keeping that rule explicit avoids treating values such as `8288` as hex.
pub fn parse_did_key(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| format!("Invalid decimal DID key: {value}"))
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

pub fn get_did_definitions_for_manufacturer(manufacturer: &str) -> Vec<DidDefinition> {
    let dids_map = &ECU_DATABASE.dids;
    let key = manufacturer_to_did_key(manufacturer);

    let raw_dids = if key.is_empty() {
        // Fallback: try string-contains search
        dids_map
            .iter()
            .find(|(k, _)| k.to_uppercase().contains(&manufacturer.to_uppercase().replace(" ", "_")))
            .map(|(_, dids)| dids)
    } else {
        dids_map.get(key)
    };

    let mut definitions: Vec<DidDefinition> = raw_dids
        .into_iter()
        .flat_map(|dids| dids.iter())
        .filter_map(|(id, name)| match parse_did_key(id) {
            Ok(id) => Some(DidDefinition { id, name: name.clone() }),
            Err(error) => {
                tracing::warn!("Skipping malformed catalog DID {id}: {error}");
                None
            }
        })
        .collect();
    definitions.sort_by(|left, right| left.id.cmp(&right.id).then_with(|| left.name.cmp(&right.name)));
    definitions.dedup_by(|left, right| left.id == right.id && left.name == right.name);
    definitions
}

pub fn get_dids_for_manufacturer(manufacturer: &str) -> Vec<(String, String)> {
    get_did_definitions_for_manufacturer(manufacturer)
        .into_iter()
        .map(|did| (format!("{:04X}", did.id), did.name))
        .collect()
}

/// Return the physical ECU request header for catalog measurements whose
/// ownership is known. Unknown PSA body parameters are deliberately not sent
/// to the engine ECU because a plausible value from the wrong ECU is unsafe.
pub fn get_did_request_header(manufacturer: &str, name: &str) -> Option<&'static str> {
    let manufacturer = manufacturer.to_uppercase();
    let name = name.to_lowercase();
    let is_psa = matches!(
        manufacturer.as_str(),
        "PEUGEOT" | "CITROËN" | "CITROEN" | "DS" | "DS AUTOMOBILES" | "OPEL" | "VAUXHALL"
    );

    if !is_psa {
        return Some("7E0");
    }

    const ENGINE_TERMS: &[&str] = &[
        "inject", "misfire", "rail", "fuel", "engine", "rpm", "turbo", "boost",
        "egr", "exhaust", "coolant", "oil", "torque", "air flow", "airflow",
        "ecu serial", "ecu software", "vin",
    ];
    const BODY_TERMS: &[&str] = &[
        "bsi", "parking", "door", "window", "wiper", "lighting", "alarm", "airbag",
        "climate", "radio", "telematic", "instrument", "maintenance", "service",
    ];

    if ENGINE_TERMS.iter().any(|term| name.contains(term)) {
        Some("6A8")
    } else if BODY_TERMS.iter().any(|term| name.contains(term)) {
        Some("75D")
    } else {
        None
    }
}

pub fn did_discovery_priority(name: &str) -> u8 {
    let name = name.to_lowercase();
    if ["injector", "injection", "misfire", "rail pressure"]
        .iter()
        .any(|term| name.contains(term))
    {
        0
    } else if ["fuel", "engine", "rpm", "boost", "egr", "coolant", "oil"]
        .iter()
        .any(|term| name.contains(term))
    {
        1
    } else {
        2
    }
}

pub fn get_all_manufacturer_dids() -> HashMap<String, Vec<(String, String)>> {
    let mut result = HashMap::new();

    for (key, dids) in &ECU_DATABASE.dids {
        let mut dids_vec: Vec<(String, String)> = dids
            .iter()
            .filter_map(|(id, desc)| parse_did_key(id).ok().map(|id| (format!("{id:04X}"), desc.clone())))
            .collect();
        dids_vec.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        result.insert(key.clone(), dids_vec);
    }

    result
}

pub fn get_manufacturer_ecu_map() -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (make, value) in &ECU_DATABASE.maps.manufacturer_ecu_map {
        result.insert(make.clone(), value.to_string());
    }

    result
}

pub fn get_manufacturer_did_map() -> HashMap<String, String> {
    let mut result = HashMap::new();

    for (make, value) in &ECU_DATABASE.maps.manufacturer_did_map {
        result.insert(make.clone(), value.to_string());
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
