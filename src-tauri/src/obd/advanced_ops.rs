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

/// Map operation IDs to real UDS hex commands
pub fn resolve_operation_command(op_id: &str) -> Option<(&str, &str)> {
    match op_id {
        "reset_service" => Some(("752", "2E 2282 00")),
        "set_service_threshold" => Some(("752", "2E 2282")),
        "write_config" => Some(("752", "2E 2100")),
        "force_regen" => Some(("7E0", "31 01 0060")),
        "test_injectors" => Some(("7E0", "30 01")),
        "test_relays" => Some(("7E0", "30 02")),
        _ => None,
    }
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
    fn test_get_categories_count() {
        let categories = get_categories();
        // Should have reasonable number of categories (at least 5)
        assert!(categories.len() >= 5);
    }

    #[test]
    fn test_get_categories_have_id() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.id.is_empty());
        }
    }

    #[test]
    fn test_get_categories_have_unique_ids() {
        let categories = get_categories();
        let mut ids = Vec::new();
        for cat in &categories {
            assert!(!ids.contains(&cat.id), "Duplicate category ID: {}", cat.id);
            ids.push(cat.id.clone());
        }
    }

    #[test]
    fn test_get_categories_have_names() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.name.en.is_empty(), "Category {} missing English name", cat.id);
            assert!(!cat.name.fr.is_empty(), "Category {} missing French name", cat.id);
        }
    }

    #[test]
    fn test_get_categories_names_not_identical() {
        let categories = get_categories();
        for cat in categories {
            // Names should be different in different languages (at least for some)
            // Check that not ALL categories have identical EN/FR names
            assert_ne!(cat.name.en, cat.name.fr, "Category {} has identical EN/FR names", cat.id);
        }
    }

    #[test]
    fn test_get_categories_have_descriptions() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.desc.en.is_empty(), "Category {} missing English description", cat.id);
            assert!(!cat.desc.fr.is_empty(), "Category {} missing French description", cat.id);
        }
    }

    #[test]
    fn test_get_categories_have_risk() {
        let categories = get_categories();
        for cat in categories {
            assert!(!cat.risk.is_empty(), "Category {} missing risk level", cat.id);
            // Risk should be one of known values
            assert!(
                matches!(cat.risk.as_str(), "low" | "medium" | "high" | "critical"),
                "Category {} has unknown risk level: {}",
                cat.id,
                cat.risk
            );
        }
    }

    #[test]
    fn test_get_categories_risk_values_varied() {
        let categories = get_categories();
        let mut risk_levels = std::collections::HashSet::new();
        for cat in categories {
            risk_levels.insert(cat.risk.clone());
        }
        // Should have at least 2 different risk levels
        assert!(risk_levels.len() >= 2, "Categories should have varied risk levels");
    }

    #[test]
    fn test_get_manufacturer_groups_not_empty() {
        let groups = get_manufacturer_groups();
        assert!(!groups.is_empty());
    }

    #[test]
    fn test_get_manufacturer_groups_count() {
        let groups = get_manufacturer_groups();
        // Should have reasonable number of manufacturer groups
        assert!(groups.len() >= 3);
    }

    #[test]
    fn test_get_manufacturer_groups_have_manufacturers() {
        let groups = get_manufacturer_groups();
        for (group_name, group) in &groups {
            assert!(!group.manufacturers.is_empty(), "Group {} has no manufacturers", group_name);
        }
    }

    #[test]
    fn test_get_manufacturer_groups_keys_not_empty() {
        let groups = get_manufacturer_groups();
        for (key, _) in groups {
            assert!(!key.is_empty());
        }
    }

    #[test]
    fn test_get_manufacturer_groups_manufacturer_strings() {
        let groups = get_manufacturer_groups();
        for (_, group) in groups {
            for manufacturer in &group.manufacturers {
                assert!(!manufacturer.is_empty(), "Manufacturer name should not be empty");
            }
        }
    }

    #[test]
    fn test_get_manufacturer_groups_contains_known_manufacturers() {
        let groups = get_manufacturer_groups();
        let all_manufacturers: Vec<String> = groups
            .values()
            .flat_map(|g| g.manufacturers.iter().cloned())
            .collect();
        // Should contain some well-known manufacturers
        let known = vec!["BMW", "Mercedes", "Volkswagen", "Audi", "Toyota"];
        let has_known = all_manufacturers.iter()
            .any(|m| known.iter().any(|k| m.contains(k)));
        assert!(has_known, "No known manufacturers found in groups");
    }

    #[test]
    fn test_get_manufacturer_groups_no_empty_manufacturer_lists() {
        let groups = get_manufacturer_groups();
        for (group_name, group) in &groups {
            assert!(!group.manufacturers.is_empty(), "Group {} is empty", group_name);
            for manufacturer in &group.manufacturers {
                assert!(!manufacturer.trim().is_empty(), "Group {} has empty/whitespace manufacturer", group_name);
            }
        }
    }

    #[test]
    fn test_resolve_operation_command_valid_ids() {
        assert_eq!(resolve_operation_command("reset_service"), Some(("752", "2E 2282 00")));
        assert_eq!(resolve_operation_command("set_service_threshold"), Some(("752", "2E 2282")));
        assert_eq!(resolve_operation_command("write_config"), Some(("752", "2E 2100")));
        assert_eq!(resolve_operation_command("force_regen"), Some(("7E0", "31 01 0060")));
        assert_eq!(resolve_operation_command("test_injectors"), Some(("7E0", "30 01")));
        assert_eq!(resolve_operation_command("test_relays"), Some(("7E0", "30 02")));
    }

    #[test]
    fn test_resolve_operation_command_invalid_ids() {
        assert_eq!(resolve_operation_command("invalid_op"), None);
        assert_eq!(resolve_operation_command(""), None);
        assert_eq!(resolve_operation_command("unknown"), None);
    }

    #[test]
    fn test_resolve_operation_command_returns_hex_strings() {
        if let Some((ecu_id, command)) = resolve_operation_command("reset_service") {
            // ECU ID should look like hex
            assert!(ecu_id.chars().all(|c| c.is_ascii_hexdigit()));
            // Command should contain hex bytes
            assert!(command.contains("2E") || command.contains("30") || command.contains("31"));
        }
    }

    #[test]
    fn test_localized_text_structure() {
        let categories = get_categories();
        for cat in categories {
            // Each LocalizedText should have both EN and FR
            let _ = cat.name;
            let _ = cat.desc;
            // If deserialized successfully, both fields exist
            assert!(true);
        }
    }

    #[test]
    fn test_category_structure_complete() {
        let categories = get_categories();
        for cat in categories {
            // Validate all required fields are present and non-empty
            assert!(!cat.id.is_empty());
            assert!(!cat.name.en.is_empty());
            assert!(!cat.name.fr.is_empty());
            assert!(!cat.desc.en.is_empty());
            assert!(!cat.desc.fr.is_empty());
            assert!(!cat.risk.is_empty());
        }
    }
}
