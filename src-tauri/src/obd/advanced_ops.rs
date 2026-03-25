use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

const ADVANCED_OPS_JSON: &str = include_str!("../../data/advanced_operations.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedText {
    pub en: String,
    pub fr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: LocalizedText,
    pub desc: LocalizedText,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturerGroup {
    pub manufacturers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawAdvancedOps {
    categories: HashMap<String, RawCategory>,
    manufacturer_groups: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawCategory {
    name: LocalizedText,
    desc: LocalizedText,
    risk: String,
}

static PARSED_OPS: LazyLock<RawAdvancedOps> = LazyLock::new(|| {
    serde_json::from_str(ADVANCED_OPS_JSON)
        .expect("Failed to parse advanced operations JSON")
});

pub fn get_categories() -> Vec<Category> {
    PARSED_OPS
        .categories
        .iter()
        .map(|(id, raw)| Category {
            id: id.clone(),
            name: raw.name.clone(),
            desc: raw.desc.clone(),
            risk: raw.risk.clone(),
        })
        .collect()
}

pub fn get_manufacturer_groups() -> HashMap<String, ManufacturerGroup> {
    PARSED_OPS
        .manufacturer_groups
        .iter()
        .map(|(group_name, manufacturers)| {
            (
                group_name.clone(),
                ManufacturerGroup {
                    manufacturers: manufacturers.clone(),
                },
            )
        })
        .collect()
}

pub fn get_all_operations() -> Vec<Category> {
    get_categories()
}
