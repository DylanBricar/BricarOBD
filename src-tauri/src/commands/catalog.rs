use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{command, AppHandle, Manager};

use super::database;

pub const CATALOG_FILE_NAME: &str = "bricarobd.db";
pub const CATALOG_DOWNLOAD_URL: &str =
    "https://github.com/DylanBricar/BricarOBD/releases/download/db-v1/bricarobd.db";
pub const CATALOG_EXPECTED_BYTES: u64 = 527_691_776;
pub const CATALOG_EXPECTED_SHA256: &str =
    "fccab50a9e422588d20c0e36737a3056b3b0dcca9ec6f8909a4900cfd34f9699";
#[cfg(feature = "mobile")]
const PROGRESS_STEP_BYTES: u64 = 1024 * 1024;

#[cfg(feature = "mobile")]
fn encode_hex(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(HEX_DIGITS[(byte >> 4) as usize] as char);
        encoded.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}

static DOWNLOAD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStatus {
    pub ready: bool,
    pub downloading: bool,
    pub expected_bytes: u64,
    pub stats: Option<CatalogStats>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStats {
    pub operations: u64,
    pub profiles: u64,
    pub ecus: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDownloadEvent {
    pub stage: &'static str,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

#[cfg(feature = "mobile")]
struct DownloadGuard;

#[cfg(feature = "mobile")]
impl Drop for DownloadGuard {
    fn drop(&mut self) {
        DOWNLOAD_IN_PROGRESS.store(false, Ordering::Release);
    }
}

pub fn app_catalog_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to resolve app data directory: {error}"))?;
    Ok(app_data_dir.join(CATALOG_FILE_NAME))
}

pub fn catalog_status() -> CatalogStatus {
    let stats = database::database_stats().map(|(operations, profiles, ecus)| CatalogStats {
        operations,
        profiles,
        ecus,
    });
    CatalogStatus {
        ready: stats.is_some(),
        downloading: DOWNLOAD_IN_PROGRESS.load(Ordering::Acquire),
        expected_bytes: CATALOG_EXPECTED_BYTES,
        stats,
    }
}

#[command]
pub fn get_catalog_status() -> CatalogStatus {
    catalog_status()
}

#[cfg(feature = "mobile")]
#[command]
pub async fn download_catalog(
    app: AppHandle,
    on_event: tauri::ipc::Channel<CatalogDownloadEvent>,
) -> Result<CatalogStatus, String> {
    use futures::StreamExt;
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncWriteExt;

    if database::is_database_initialized() {
        return Ok(catalog_status());
    }
    if DOWNLOAD_IN_PROGRESS
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("Catalog download is already in progress".to_string());
    }
    let _guard = DownloadGuard;

    let catalog_path = app_catalog_path(&app)?;
    let app_data_dir = catalog_path
        .parent()
        .ok_or("Catalog path has no parent directory")?;
    tokio::fs::create_dir_all(app_data_dir)
        .await
        .map_err(|error| format!("Unable to create catalog directory: {error}"))?;
    let partial_path = app_data_dir.join(format!("{CATALOG_FILE_NAME}.part"));

    let client = reqwest::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(30 * 60))
        .build()
        .map_err(|error| format!("Unable to create download client: {error}"))?;
    let response = client
        .get(CATALOG_DOWNLOAD_URL)
        .send()
        .await
        .map_err(|error| format!("Catalog download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Catalog server rejected the download: {error}"))?;

    if let Some(content_length) = response.content_length() {
        if content_length != CATALOG_EXPECTED_BYTES {
            return Err(format!(
                "Unexpected catalog size: {content_length} bytes (expected {CATALOG_EXPECTED_BYTES})"
            ));
        }
    }

    let _ = on_event.send(CatalogDownloadEvent {
        stage: "started",
        downloaded_bytes: 0,
        total_bytes: CATALOG_EXPECTED_BYTES,
    });

    let mut file = tokio::fs::File::create(&partial_path)
        .await
        .map_err(|error| format!("Unable to create temporary catalog: {error}"))?;
    let mut hasher = Sha256::new();
    let mut downloaded_bytes = 0_u64;
    let mut last_reported_bytes = 0_u64;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("Catalog download interrupted: {error}"))?;
        downloaded_bytes = downloaded_bytes
            .checked_add(chunk.len() as u64)
            .ok_or("Catalog size overflow")?;
        if downloaded_bytes > CATALOG_EXPECTED_BYTES {
            let _ = tokio::fs::remove_file(&partial_path).await;
            return Err("Downloaded catalog is larger than expected".to_string());
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| format!("Unable to write catalog: {error}"))?;
        hasher.update(&chunk);
        if downloaded_bytes.saturating_sub(last_reported_bytes) >= PROGRESS_STEP_BYTES {
            let _ = on_event.send(CatalogDownloadEvent {
                stage: "progress",
                downloaded_bytes,
                total_bytes: CATALOG_EXPECTED_BYTES,
            });
            last_reported_bytes = downloaded_bytes;
        }
    }
    file.flush()
        .await
        .map_err(|error| format!("Unable to flush catalog: {error}"))?;
    file.sync_all()
        .await
        .map_err(|error| format!("Unable to persist catalog: {error}"))?;
    drop(file);

    let actual_sha256 = encode_hex(&hasher.finalize());
    if downloaded_bytes != CATALOG_EXPECTED_BYTES || actual_sha256 != CATALOG_EXPECTED_SHA256 {
        let _ = tokio::fs::remove_file(&partial_path).await;
        return Err(format!(
            "Catalog integrity check failed (size {downloaded_bytes}, SHA-256 {actual_sha256})"
        ));
    }

    tokio::fs::rename(&partial_path, &catalog_path)
        .await
        .map_err(|error| format!("Unable to install verified catalog: {error}"))?;

    let user_db_path = app_data_dir.join("bricarobd-user.db");
    database::init_database_internal(&catalog_path, &user_db_path)?;
    let status = catalog_status();
    let _ = on_event.send(CatalogDownloadEvent {
        stage: "finished",
        downloaded_bytes,
        total_bytes: CATALOG_EXPECTED_BYTES,
    });
    Ok(status)
}

#[cfg(not(feature = "mobile"))]
#[command]
pub async fn download_catalog(
    _app: AppHandle,
    _on_event: tauri::ipc::Channel<CatalogDownloadEvent>,
) -> Result<CatalogStatus, String> {
    Err("The desktop catalog is included in the installer".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_catalog_identity_is_pinned() {
        assert_eq!(CATALOG_EXPECTED_BYTES, 527_691_776);
        assert_eq!(CATALOG_EXPECTED_SHA256.len(), 64);
        assert!(CATALOG_DOWNLOAD_URL.starts_with("https://github.com/"));
        assert!(!CATALOG_DOWNLOAD_URL.contains('?'));
    }

    #[cfg(feature = "mobile")]
    #[test]
    fn digest_bytes_are_encoded_as_lowercase_hex() {
        assert_eq!(encode_hex(&[0x00, 0x0f, 0x10, 0xab, 0xff]), "000f10abff");
    }
}
