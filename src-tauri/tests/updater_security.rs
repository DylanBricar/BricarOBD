use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
use tauri_plugin_updater::{Update, UpdaterExt};

// Public upstream test vector, not a production key or private signing material.
// tauri-plugin-updater 2.14.0 / src/updater.rs / offline_tests (Apache-2.0 OR MIT).
// The signature covers the harmless bytes `test`, with signed version 2.0.0.
const PUBLIC_KEY: &str = "untrusted comment: minisign public key: A49EF4746FA6E6D7\nRWTX5qZvdPSepDGWg2bz8n6ArQnBMwgQK1geVosorcbLzcunD+KJ+cYe";
const SIGNATURE: &str = "untrusted comment: signature from tauri secret key\nRUTX5qZvdPSepPF2zsjvP/L/TQtBLRnWRvaq+4omK1UEugtQp/USrGiI1v762sDy3id3aor7bRC9ETyeot6lMHHrWoZ5hjUM1Qc=\ntrusted comment: timestamp:1791649051\tfile:payload\tversion:2.0.0\nDDq2CLOEl5cQmC2eJRHlnZPVViOZsu1CjpPKQoei4A/XBfkDqGO0B+QMrcrHgEQ5qV3mne75tdFWhJpqUXh9AA==";

fn fixture_update() -> Update {
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert(
        "updater".into(),
        json!({ "pubkey": STANDARD.encode(PUBLIC_KEY) }),
    );
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .expect("headless test app");
    app.updater_builder()
        .pubkey(STANDARD.encode(PUBLIC_KEY))
        .target("fixture-target")
        .executable_path(std::env::temp_dir().join("bricarobd-updater-fixture.exe"))
        .configure_client(|_| panic!("offline fixture must not make network requests"))
        .endpoints(vec!["https://example.invalid/update".parse().unwrap()])
        .unwrap()
        .build()
        .unwrap()
        .restore_update(json!({
            "version": "2.0.0",
            "platforms": {
                "fixture-target": {
                    "url": "https://example.invalid/fixture.exe",
                    "signature": STANDARD.encode(SIGNATURE)
                }
            }
        }))
        .unwrap()
        .expect("newer signed fixture version")
}

#[test]
fn verifies_signed_payload_without_legacy_zip_feature() {
    fixture_update().verify(b"test").unwrap();
}

#[test]
fn corrupted_payload_is_rejected_before_installation() {
    let update = fixture_update();
    assert!(update.verify(b"changed").is_err());
    assert!(update.install(b"changed").is_err());
}

#[cfg(windows)]
#[test]
fn signed_non_installer_is_rejected_by_direct_windows_format_handler() {
    // Signature validation succeeds, but these harmless bytes are neither PE nor MSI.
    // The real updater rejects them before writing or launching an installer.
    let result = fixture_update().install(b"test");
    assert!(matches!(
        result,
        Err(tauri_plugin_updater::Error::InvalidUpdaterFormat)
    ));
}
