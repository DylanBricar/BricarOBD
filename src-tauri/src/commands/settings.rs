use std::path::PathBuf;
use tauri::command;
use crate::models::AppSettings;
use crate::obd::dev_log;

/// Get application settings
#[command]
pub fn get_settings() -> AppSettings {
    dev_log::log_debug("settings", "Getting application settings");
    super::database::with_db(|db| Ok(db.get_settings())).unwrap_or_default()
}

/// Save application settings
#[command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    super::database::with_db(|db| {
        db.save_settings_batch(vec![
            ("language", &settings.language),
            ("default_baud_rate", &settings.default_baud_rate.to_string()),
            ("theme", &settings.theme),
            ("auto_connect", &settings.auto_connect.to_string()),
        ])
    })?;
    dev_log::log_info("settings", &format!("Settings saved: lang={}, baud={}, theme={}, auto_connect={}", settings.language, settings.default_baud_rate, settings.theme, settings.auto_connect));
    Ok(())
}

/// Save CSV content to file on disk, returns the full path
#[command]
pub fn save_csv_file(filename: String, content: String) -> Result<String, String> {
    // Sanitize filename - only allow safe characters
    let safe_name: String = filename.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect();
    if safe_name.is_empty() || safe_name.contains("..") {
        dev_log::log_warn("settings", "Invalid filename: sanitization failed");
        return Err(crate::commands::connection::err_msg("Nom de fichier invalide", "Invalid filename"));
    }
    if !safe_name.ends_with(".csv") {
        dev_log::log_warn("settings", &format!("Rejected non-CSV filename: {}", safe_name));
        return Err(crate::commands::connection::err_msg("Le fichier doit se terminer par .csv", "Filename must end with .csv"));
    }

    // Save to Desktop
    let desktop = dirs_next().ok_or("Cannot find Desktop directory")?;
    let dir = desktop.join("BricarOBD_Exports");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create dir: {}", e))?;

    let path = dir.join(&safe_name);

    // Defense in depth: verify resolved path stays inside exports dir
    let canonical_dir = dir.canonicalize().map_err(|e| format!("Cannot resolve dir: {}", e))?;
    // For new files, canonicalize the parent and check
    if let Some(parent) = path.parent() {
        let canonical_parent = parent.canonicalize().map_err(|e| format!("Cannot resolve path: {}", e))?;
        if !canonical_parent.starts_with(&canonical_dir) {
            dev_log::log_warn("settings", &format!("Path traversal blocked: {}", path.display()));
            return Err(crate::commands::connection::err_msg("Traversée de chemin bloquée", "Path traversal blocked"));
        }
    }

    // Write with BOM for Excel compatibility
    let bom_content = format!("\u{FEFF}{}", content);
    std::fs::write(&path, bom_content).map_err(|e| format!("Cannot write file: {}", e))?;

    let full_path = path.to_string_lossy().to_string();
    dev_log::log_info("settings", &format!("CSV saved: {} (size: {} bytes)", full_path, content.len()));
    Ok(full_path)
}

/// Read a CSV file from disk
#[command]
pub fn read_csv_file(path: String) -> Result<String, String> {
    dev_log::log_debug("settings", &format!("Reading CSV file: {}", path));
    let file_path = std::path::PathBuf::from(&path);
    let canonical = file_path.canonicalize().map_err(|e| format!("Invalid path: {}", e))?;
    let desktop = dirs_next().ok_or("Cannot find directory")?;
    let allowed_dir = desktop.join("BricarOBD_Exports");
    let allowed_dir = allowed_dir.canonicalize().map_err(|e| format!("Cannot resolve exports dir: {}", e))?;
    if !canonical.starts_with(&allowed_dir) {
        dev_log::log_warn("settings", &format!("Access denied for path outside exports directory: {}", path));
        return Err(crate::commands::connection::err_msg("Accès refusé : chemin en dehors du répertoire d'exports", "Access denied: path outside exports directory"));
    }

    // Check file size (reject > 50MB)
    let metadata = std::fs::metadata(&canonical).map_err(|e| format!("Cannot read file metadata: {}", e))?;
    const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
    if metadata.len() > MAX_FILE_SIZE {
        dev_log::log_warn("settings", &format!("File too large: {} bytes", metadata.len()));
        return Err(crate::commands::connection::err_msg("Fichier trop volumineux (max 50MB)", "File too large (max 50MB)"));
    }

    dev_log::log_info("settings", &format!("CSV file access allowed: {} (size: {} bytes)", canonical.to_string_lossy(), metadata.len()));
    std::fs::read_to_string(&canonical).map_err(|e| format!("Cannot read file: {}", e))
}

/// List exported CSV files
#[command]
pub fn list_exports() -> Result<Vec<serde_json::Value>, String> {
    let desktop = dirs_next().ok_or("Cannot find Desktop directory")?;
    let dir = desktop.join("BricarOBD_Exports");

    if !dir.exists() {
        dev_log::log_debug("settings", "Exports directory does not exist yet");
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("Cannot read dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "csv").unwrap_or(false) {
            let metadata = std::fs::metadata(&path).ok();
            files.push(serde_json::json!({
                "name": path.file_name().unwrap_or_default().to_string_lossy(),
                "path": path.to_string_lossy(),
                "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                "modified": metadata.and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            }));
        }
    }

    files.sort_by(|a, b| {
        b.get("modified").and_then(|v| v.as_u64())
            .cmp(&a.get("modified").and_then(|v| v.as_u64()))
    });

    dev_log::log_debug("settings", &format!("Listed {} exported CSV files", files.len()));
    Ok(files)
}

/// Open the exports folder in file manager
#[command]
pub fn open_exports_folder() -> Result<(), String> {
    let desktop = dirs_next().ok_or("Cannot find Desktop directory")?;
    let dir = desktop.join("BricarOBD_Exports");
    std::fs::create_dir_all(&dir).ok();
    dev_log::log_info("settings", &format!("Opening exports folder: {}", dir.to_string_lossy()));

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&dir).spawn().ok();

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&dir).spawn().ok();

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&dir).spawn().ok();

    Ok(())
}

/// Get dev console logs — NO logging here to avoid infinite loop
#[command]
pub fn get_dev_logs(since_index: Option<usize>) -> Vec<crate::obd::dev_log::LogEntry> {
    // Always use incremental API — avoids cloning all 5000 entries at once
    crate::obd::dev_log::get_logs_since(since_index.unwrap_or(0))
}

/// Get dev log count — NO logging here to avoid spam
#[command]
pub fn get_dev_log_count() -> usize {
    crate::obd::dev_log::log_count()
}

/// Clear dev logs
#[command]
pub fn clear_dev_logs() {
    dev_log::log_info("settings", "Clearing all dev logs");
    crate::obd::dev_log::clear_logs();
}

/// Add a frontend log entry to the dev console
#[command]
pub fn add_dev_log(level: String, source: String, message: String) {
    let source: String = source.chars().take(64).collect();
    let message: String = message.chars().take(512).collect();
    match level.to_lowercase().as_str() {
        "debug" => dev_log::log_debug(&source, &message),
        "warn" => dev_log::log_warn(&source, &message),
        "error" => dev_log::log_error(&source, &message),
        _ => dev_log::log_info(&source, &message),
    }
}

/// Batch-add frontend log entries to the dev console
#[derive(serde::Deserialize)]
pub struct BatchLogEntry {
    pub level: String,
    pub source: String,
    pub message: String,
}

#[command]
pub fn add_dev_logs_batch(logs: Vec<BatchLogEntry>) {
    for entry in logs.into_iter().take(500) {
        let source: String = entry.source.chars().take(64).collect();
        let message: String = entry.message.chars().take(512).collect();
        match entry.level.to_lowercase().as_str() {
            "debug" => dev_log::log_debug(&source, &message),
            "warn" => dev_log::log_warn(&source, &message),
            "error" => dev_log::log_error(&source, &message),
            _ => dev_log::log_info(&source, &message),
        }
    }
}

fn dirs_next() -> Option<PathBuf> {
    // Try Desktop first, fallback to home
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        let desktop = home.join("Desktop");
        if desktop.exists() {
            return Some(desktop);
        }
        return Some(home);
    }
    // Windows
    if let Some(profile) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        let desktop = profile.join("Desktop");
        if desktop.exists() {
            return Some(desktop);
        }
        return Some(profile);
    }
    None
}

/// Get the path to the logs directory
#[command]
pub fn get_log_dir() -> Option<String> {
    crate::obd::dev_log::get_log_dir_path()
        .map(|p| p.to_string_lossy().to_string())
}

/// Open the logs folder in file manager
#[command]
pub fn open_log_folder() -> Result<(), String> {
    let log_dir = crate::obd::dev_log::get_log_dir_path()
        .ok_or_else(|| "Cannot get log directory".to_string())?;

    dev_log::log_info("settings", &format!("Opening logs folder: {}", log_dir.to_string_lossy()));

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&log_dir).spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&log_dir).spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&log_dir).spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    Ok(())
}
