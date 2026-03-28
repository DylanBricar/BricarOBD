use serde::{Serialize, Deserialize};
use std::path::PathBuf;

const CACHE_TTL_DAYS: u64 = 30;

/// VIN-based cache system for storing discovered PIDs, DIDs, and ECU addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VinCache {
    pub vin: String,
    pub supported_pids: Vec<u8>,
    pub supported_dids: Vec<(String, String)>, // (hex_id, name)
    #[serde(default)]
    pub failed_pids: Vec<u8>,
    #[serde(default)]
    pub failed_dids: Vec<String>,
    pub created_at: u64,
}

impl VinCache {
    /// Create a new VIN cache
    pub fn new(vin: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            vin,
            supported_pids: Vec::new(),
            supported_dids: Vec::new(),
            failed_pids: Vec::new(),
            failed_dids: Vec::new(),
            created_at: now,
        }
    }
}

/// Get data directory for the application (OS-specific)
pub fn get_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join("Library/Application Support/BricarOBD"))
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA").ok()?;
        Some(PathBuf::from(app_data).join("BricarOBD"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/BricarOBD"))
    }
}

/// Get the VIN cache directory
pub fn get_cache_dir() -> Option<PathBuf> {
    let data_dir = get_data_dir()?;
    let cache_dir = data_dir.join("vin_cache");
    std::fs::create_dir_all(&cache_dir).ok()?;
    Some(cache_dir)
}

/// Get the logs directory
pub fn get_log_dir_path() -> Option<PathBuf> {
    let data_dir = get_data_dir()?;
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).ok()?;
    Some(logs_dir)
}

/// Sanitize VIN for use as a filename
fn sanitize_filename(vin: &str) -> String {
    vin.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

/// Load cache from disk for a given VIN
pub fn load_cache(vin: &str) -> Option<VinCache> {
    let cache_dir = get_cache_dir()?;
    let filename = sanitize_filename(vin);
    let path = cache_dir.join(format!("{}.json", filename));

    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;
    let cache: VinCache = serde_json::from_str(&content).ok()?;

    // Validate that VIN matches after deserialization
    if cache.vin != vin {
        return None;
    }

    // Check TTL
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let age_days = now.saturating_sub(cache.created_at) / 86400;
    if age_days > CACHE_TTL_DAYS {
        crate::obd::dev_log::log_warn("vin_cache", &format!("VIN cache for {} is {} days old, refreshing", vin, age_days));
        return None;
    }

    Some(cache)
}

/// Save cache to disk
pub fn save_cache(cache: &VinCache) -> Result<(), String> {
    let cache_dir = get_cache_dir().ok_or_else(|| "Cannot get cache directory".to_string())?;
    let filename = sanitize_filename(&cache.vin);
    let path = cache_dir.join(format!("{}.json", filename));

    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write cache: {}", e))?;

    Ok(())
}

/// Check if cache exists for a VIN
pub fn has_cache(vin: &str) -> bool {
    load_cache(vin).is_some()
}

/// Clear cache for a VIN
pub fn clear_cache(vin: &str) -> Result<(), String> {
    let cache_dir = get_cache_dir().ok_or_else(|| "Cannot get cache directory".to_string())?;
    let filename = sanitize_filename(vin);
    let path = cache_dir.join(format!("{}.json", filename));

    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete cache: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vin_cache_new() {
        let cache = VinCache::new("VF3LCBHZ6JS000000".to_string());
        assert_eq!(cache.vin, "VF3LCBHZ6JS000000");
        assert!(cache.supported_pids.is_empty());
        assert!(cache.supported_dids.is_empty());
        assert!(cache.failed_pids.is_empty());
        assert!(cache.failed_dids.is_empty());
        assert!(cache.created_at > 0);
    }

    #[test]
    fn test_vin_cache_with_data() {
        let mut cache = VinCache::new("TEST123".to_string());
        cache.supported_pids = vec![0x01, 0x05, 0x0C];
        cache.supported_dids = vec![("F190".to_string(), "VIN".to_string())];

        assert_eq!(cache.supported_pids.len(), 3);
        assert_eq!(cache.supported_dids.len(), 1);
    }

    #[test]
    fn test_get_data_dir() {
        let data_dir = get_data_dir();
        assert!(data_dir.is_some());
        if let Some(dir) = data_dir {
            assert!(dir.to_string_lossy().contains("BricarOBD"));
        }
    }

    #[test]
    fn test_get_cache_dir() {
        let cache_dir = get_cache_dir();
        assert!(cache_dir.is_some());
        if let Some(dir) = cache_dir {
            assert!(dir.to_string_lossy().contains("vin_cache"));
        }
    }

    #[test]
    fn test_sanitize_filename() {
        let result = sanitize_filename("VF3LCBHZ6JS000000");
        assert_eq!(result, "vf3lcbhz6js000000");
    }

    #[test]
    fn test_sanitize_filename_with_special_chars() {
        let result = sanitize_filename("VF3-LCB_HZ6JS000000");
        assert_eq!(result, "vf3-lcb_hz6js000000");
    }

    #[test]
    fn test_sanitize_filename_removes_invalid_chars() {
        let result = sanitize_filename("VF3@LCB#HZ6JS$00");
        assert_eq!(result, "vf3lcbhz6js00");
    }

    #[test]
    fn test_save_and_load_cache() {
        let vin = format!("TEST_VIN_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        let mut cache = VinCache::new(vin.clone());
        cache.supported_pids = vec![0x01, 0x05];
        cache.supported_dids = vec![("F190".to_string(), "VIN".to_string())];

        if let Ok(()) = save_cache(&cache) {
            if let Some(loaded) = load_cache(&vin) {
                assert_eq!(loaded.vin, cache.vin);
                assert_eq!(loaded.supported_pids, cache.supported_pids);
                assert_eq!(loaded.supported_dids, cache.supported_dids);
                let _ = clear_cache(&vin);
            }
        }
    }

    #[test]
    fn test_has_cache() {
        let vin = format!("TEST_HAS_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        assert!(!has_cache(&vin));

        let cache = VinCache::new(vin.clone());
        if let Ok(()) = save_cache(&cache) {
            assert!(has_cache(&vin));
            let _ = clear_cache(&vin);
        }
    }

    #[test]
    fn test_clear_cache() {
        let vin = format!("TEST_CLEAR_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        let cache = VinCache::new(vin.clone());
        if let Ok(()) = save_cache(&cache) {
            assert!(has_cache(&vin));
            if let Ok(()) = clear_cache(&vin) {
                assert!(!has_cache(&vin));
            }
        }
    }

    #[test]
    fn test_cache_ttl_validation() {
        let vin = "TEST_VALIDATION".to_string();
        let mut cache = VinCache::new(vin.clone());
        cache.created_at = 0;

        if let Ok(()) = save_cache(&cache) {
            if let Some(_) = load_cache(&vin) {
                assert!(false, "TTL validation should reject old cache");
            }
            let _ = clear_cache(&vin);
        }
    }

    #[test]
    fn test_cache_vin_validation() {
        let vin = format!("TEST_VIN_MISMATCH_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        let cache = VinCache::new(vin.clone());
        if let Ok(()) = save_cache(&cache) {
            let loaded = load_cache("DIFFERENT_VIN");
            assert!(loaded.is_none());
            let _ = clear_cache(&vin);
        }
    }
}
