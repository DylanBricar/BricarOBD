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
    serde_json::from_str(ADVANCED_OPS_JSON).unwrap_or_else(|e| {
        tracing::error!("Failed to parse advanced operations JSON: {}", e);
        RawAdvancedOps {
            categories: HashMap::new(),
            manufacturer_groups: HashMap::new(),
        }
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_categories_not_empty() {
        let categories = get_categories();
        assert!(!categories.is_empty());
    }

    #[test]
    fn test_get_categories_have_id() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.id.is_empty());
        }
    }

    #[test]
    fn test_get_categories_have_names() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.name.en.is_empty());
            assert!(!cat.name.fr.is_empty());
        }
    }

    #[test]
    fn test_get_categories_have_descriptions() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.desc.en.is_empty());
            assert!(!cat.desc.fr.is_empty());
        }
    }

    #[test]
    fn test_get_categories_have_risk() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.risk.is_empty());
        }
    }

    #[test]
    fn test_get_manufacturer_groups_not_empty() {
        let groups = get_manufacturer_groups();
        assert!(!groups.is_empty());
    }

    #[test]
    fn test_get_manufacturer_groups_have_manufacturers() {
        let groups = get_manufacturer_groups();
        for (_, group) in groups {
            assert!(!group.manufacturers.is_empty());
        }
    }

    #[test]
    fn test_get_manufacturer_groups_keys_not_empty() {
        let groups = get_manufacturer_groups();
        for (key, _) in groups {
            assert!(!key.is_empty());
        }
    }
}
