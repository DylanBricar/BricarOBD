use tauri::command;
use crate::models::{ConnectionStatus, PortInfo, VehicleInfo};
use crate::obd::Elm327Connection;
use std::sync::Mutex;

// Re-export helpers so Tauri generate_handler! can find them at connection::*
pub use super::connection_helpers::{OBDBusyGuard, set_obd_busy, is_obd_busy, is_private_ip};
pub use super::connection_wifi_vin::{connect_wifi, scan_wifi, set_manual_vin, has_vin_cache, clear_vin_cache};

pub enum ConnectionMode {
    Disconnected,
    Demo,
    Real(crate::obd::Elm327Connection),
}

pub static CONNECTION: Mutex<ConnectionMode> = Mutex::new(ConnectionMode::Disconnected);
static CURRENT_LANG: Mutex<String> = Mutex::new(String::new());

/// Get the current language setting (default: "en")
pub fn get_lang() -> String {
    let lang = CURRENT_LANG.lock().unwrap_or_else(|e| e.into_inner());
    if lang.is_empty() { "en".to_string() } else { lang.clone() }
}

/// Create a bilingual error message
pub fn err_msg(fr: &str, en: &str) -> String {
    if get_lang() == "fr" { fr.to_string() } else { en.to_string() }
}

/// Set the current language from frontend
#[command]
pub fn set_language(lang: String) {
    let mut guard = CURRENT_LANG.lock().unwrap_or_else(|e| e.into_inner());
    *guard = lang;
}

pub fn is_demo() -> bool {
    matches!(*CONNECTION.lock().unwrap_or_else(|e| e.into_inner()), ConnectionMode::Demo)
}

pub fn is_connected() -> bool {
    !matches!(*CONNECTION.lock().unwrap_or_else(|e| e.into_inner()), ConnectionMode::Disconnected)
}


/// Execute a closure with the real Elm327Connection. Returns Err if not in Real mode.
pub fn with_real_connection<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut Elm327Connection) -> Result<R, String>,
{
    let mut guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    match *guard {
        ConnectionMode::Real(ref mut conn) => f(conn),
        ConnectionMode::Demo => Err(err_msg("Mode démo — pas de connexion réelle", "Demo mode — no real connection")),
        ConnectionMode::Disconnected => Err(err_msg("Non connecté", "Not connected")),
    }
}

/// List available serial ports
#[command]
pub fn list_serial_ports() -> Vec<PortInfo> {
    let ports = Elm327Connection::list_ports();
    crate::obd::dev_log::log_info("connection", &format!("Listed {} serial ports: {:?}", ports.len(), ports.iter().map(|p| &p.name).collect::<Vec<_>>()));
    ports
}

/// Connect to OBD adapter — auto-detect baud rate if baud_rate is 0
#[command]
pub async fn connect_obd(port: String, baud_rate: u32) -> Result<VehicleInfo, String> {
    // Reject if already connected
    {
        let guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(*guard, ConnectionMode::Disconnected) {
            return Err(err_msg("Déjà connecté", "Already connected"));
        }
    }

    let result: Result<(Elm327Connection, VehicleInfo), String> = tokio::task::spawn_blocking(move || {
        // If baud_rate is 0, try common rates; otherwise use the specified one
        let baud_rates = if baud_rate == 0 {
            vec![38400, 115200, 9600, 230400, 500000]
        } else {
            vec![baud_rate]
        };

        crate::obd::dev_log::log_info("connection", &format!("Attempting connection to port {} with baud rates: {:?}", port, baud_rates));

        let overall_start = std::time::Instant::now();
        let overall_timeout = std::time::Duration::from_secs(90);

        let mut last_error = String::new();
        for &baud in &baud_rates {
            if overall_start.elapsed() > overall_timeout {
                crate::obd::dev_log::log_error("connection", "Connection attempt timed out after 90 seconds");
                break;
            }

            let mut conn = Elm327Connection::new();
            crate::obd::dev_log::log_debug("connection", &format!("Trying baud rate: {}", baud));
            match conn.connect(&port, baud) {
                Ok(()) => {
                    crate::obd::dev_log::log_info("connection", &format!("Connected successfully at {} baud", baud));
                    tracing::info!("Connected at {} baud", baud);
                    let vin_response = conn.send_command("0902").unwrap_or_default();
                    let vin = super::vin_parser::parse_vin_response(&vin_response);
                    let vin_info = crate::obd::vin::decode_vin(&vin);

                    crate::obd::dev_log::log_info("connection", &format!("VIN decoded: {}, Make: {}, Year: {}", vin_info.vin, vin_info.make, vin_info.year));

                    conn.vin = vin_info.vin.clone();

                    let model = super::database::find_vehicle_model_sync(&vin_info.make);
                    let info = VehicleInfo {
                        vin: vin_info.vin,
                        make: vin_info.make,
                        model: model.unwrap_or_default(),
                        year: vin_info.year,
                        protocol: conn.protocol.clone(),
                        elm_version: conn.elm_version.clone(),
                    };
                    return Ok((conn, info));
                }
                Err(e) => {
                    crate::obd::dev_log::log_warn("connection", &format!("Baud {} failed: {}", baud, e));
                    tracing::debug!("Baud {} failed: {}", baud, e);
                    last_error = e;
                    continue;
                }
            }
        }
        let diagnostic = if last_error.contains("Permission denied") || last_error.contains("Access denied") {
            last_error.clone()
        } else if last_error.contains("not found") || last_error.contains("No such file") {
            last_error.clone()
        } else if last_error.contains("All 4 connection strategies failed") {
            format!("{} — The adapter was detected but could not communicate with the vehicle ECU. Check that the ignition is ON and the adapter is firmly plugged into the OBD-II port.", last_error)
        } else {
            format!("Connection failed: {}", last_error)
        };
        crate::obd::dev_log::log_error("connection", &diagnostic);
        Err(diagnostic)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?;

    let (conn, info) = result?;
    let mut guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    *guard = ConnectionMode::Real(conn);
    Ok(info)
}

/// Disconnect from OBD adapter
#[command]
pub async fn disconnect_obd() -> Result<(), String> {
    let prev = {
        let mut guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::replace(&mut *guard, ConnectionMode::Disconnected)
    };
    if let ConnectionMode::Real(conn) = prev {
        tokio::task::spawn_blocking(move || {
            let mut c = conn;
            c.disconnect();
        }).await.ok();
    }
    crate::obd::dev_log::log_info("connection", "Disconnected from OBD adapter");

    // Clear PID history and fail counts to prevent stale data leaking into next session
    super::dashboard::clear_pid_history();

    Ok(())
}

/// Connect in demo mode
#[command]
pub fn connect_demo() -> VehicleInfo {
    let mut guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    *guard = ConnectionMode::Demo;
    crate::obd::dev_log::log_info("connection", "Demo mode activated");
    VehicleInfo {
        vin: "VF3LCBHZ6JS000000".to_string(),
        make: "Peugeot".to_string(),
        model: "207 (Demo)".to_string(),
        year: 2018,
        protocol: "ISO 15765-4 CAN (Simulated)".to_string(),
        elm_version: "Demo v1.0".to_string(),
    }
}

/// Get current connection status
#[command]
pub fn get_connection_status() -> ConnectionStatus {
    let guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    match *guard {
        ConnectionMode::Disconnected => ConnectionStatus::Disconnected,
        ConnectionMode::Demo => ConnectionStatus::Demo,
        ConnectionMode::Real(_) => ConnectionStatus::Connected,
    }
}

/// Get available connection types for the platform
#[command]
pub fn get_connection_types() -> Vec<serde_json::Value> {
    let mut types = Vec::new();

    #[cfg(feature = "desktop")]
    types.push(serde_json::json!({
        "type": "serial",
        "name": "USB Serial",
        "available": true,
    }));

    types.push(serde_json::json!({
        "type": "wifi",
        "name": "WiFi (ELM327)",
        "available": true,
    }));

    #[cfg(feature = "mobile")]
    types.push(serde_json::json!({
        "type": "usb_android",
        "name": "USB Serial (Android)",
        "available": true,
    }));

    types.push(serde_json::json!({
        "type": "bluetooth",
        "name": "Bluetooth BLE",
        "available": cfg!(feature = "mobile"),
    }));

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    static LANG_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_get_lang_default() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_language(String::new());
        assert_eq!(get_lang(), "en");
    }

    #[test]
    fn test_get_lang_set_fr() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_language("fr".to_string());
        assert_eq!(get_lang(), "fr");
        set_language("en".to_string());
    }

    #[test]
    fn test_get_lang_set_en() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_language("en".to_string());
        assert_eq!(get_lang(), "en");
    }

    #[test]
    fn test_err_msg_english() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_language("en".to_string());
        assert_eq!(err_msg("Erreur", "Error"), "Error");
    }

    #[test]
    fn test_err_msg_french() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_language("fr".to_string());
        assert_eq!(err_msg("Erreur", "Error"), "Erreur");
        set_language("en".to_string());
    }
}

