use tauri::command;
use crate::models::{ConnectionStatus, PortInfo, VehicleInfo};
use crate::obd::Elm327Connection;
use crate::obd::transport::{self, OBDTransport};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use super::vin_parser;

enum ConnectionMode {
    Disconnected,
    Demo,
    Real(crate::obd::Elm327Connection),
}

static CONNECTION: Mutex<ConnectionMode> = Mutex::new(ConnectionMode::Disconnected);
static CURRENT_LANG: Mutex<String> = Mutex::new(String::new());
static OBD_BUSY: AtomicBool = AtomicBool::new(false);

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

/// Set OBD busy state
pub fn set_obd_busy(busy: bool) {
    OBD_BUSY.store(busy, Ordering::SeqCst);
}

/// Check if OBD is busy
pub fn is_obd_busy() -> bool {
    OBD_BUSY.load(Ordering::SeqCst)
}

/// Guard to manage OBD busy state with RAII semantics
pub struct OBDBusyGuard;

impl OBDBusyGuard {
    /// Try to acquire the OBD lock, fail immediately if already busy
    pub fn try_acquire() -> Result<Self, String> {
        if OBD_BUSY.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err("OBD is busy with another operation".to_string());
        }
        Ok(OBDBusyGuard)
    }

    /// Acquire the OBD lock with timeout (capped at 10s), retrying every 100ms
    pub fn acquire_with_wait(timeout_secs: u64) -> Result<Self, String> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs.min(10));

        loop {
            if OBD_BUSY.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                crate::obd::dev_log::log_debug("connection", "OBD lock acquired");
                return Ok(OBDBusyGuard);
            }

            if start.elapsed() > timeout {
                crate::obd::dev_log::log_warn("connection", &format!("OBD lock timeout after {} seconds", timeout_secs));
                return Err(format!("OBD lock timeout after {} seconds", timeout_secs));
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

impl Drop for OBDBusyGuard {
    fn drop(&mut self) {
        // Use try_lock to avoid deadlock if CONNECTION Mutex is already held (e.g. panic unwind)
        if let Ok(mut guard) = CONNECTION.try_lock() {
            if let ConnectionMode::Real(ref mut conn) = *guard {
                let _ = conn.reset_headers();
            }
        }
        set_obd_busy(false);
        crate::obd::dev_log::log_debug("connection", "OBD lock released");
    }
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
                    let vin = vin_parser::parse_vin_response(&vin_response);
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
    if let ConnectionMode::Real(mut conn) = prev {
        conn.disconnect();
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
    let status = match *guard {
        ConnectionMode::Disconnected => ConnectionStatus::Disconnected,
        ConnectionMode::Demo => ConnectionStatus::Demo,
        ConnectionMode::Real(_) => ConnectionStatus::Connected,
    };
    crate::obd::dev_log::log_debug("connection", &format!("Status requested: {:?}", status));
    status
}


/// Connect via WiFi (ELM327 WiFi adapter) — now fully integrated with ELM327 protocol
#[command]
pub async fn connect_wifi(host: String, port: u16) -> Result<VehicleInfo, String> {
    {
        let guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(*guard, ConnectionMode::Disconnected) {
            return Err(err_msg("Déjà connecté", "Already connected"));
        }
    }

    // Validate host is a private/link-local IP to prevent connecting to arbitrary servers
    if !is_private_ip(&host) {
        crate::obd::dev_log::log_warn("connection", &format!("WiFi connect denied: {} is not a private IP", host));
        return Err(err_msg("L'hôte WiFi doit être une adresse IP locale/privée", "WiFi host must be a local/private IP address"));
    }

    crate::obd::dev_log::log_info("connection", &format!("WiFi connect: {}:{}", host, port));

    let result: Result<(Elm327Connection, VehicleInfo), String> = tokio::task::spawn_blocking(move || {
        // Create WiFi transport with generous timeout (WiFi has latency)
        let wifi = transport::WiFiTransport::new(&host, port, 8000)?;
        crate::obd::dev_log::log_info("connection", "WiFi transport established, starting ELM327 init...");

        // Wire WiFi transport into the full ELM327 protocol layer
        let mut conn = Elm327Connection::new();
        conn.connect_transport(Box::new(wifi))?;

        crate::obd::dev_log::log_info("connection", &format!("WiFi ELM327 connected: {}", conn.elm_version));

        // Read VIN
        let vin_response = conn.send_command("0902").unwrap_or_default();
        let vin = vin_parser::parse_vin_response(&vin_response);
        let vin_info = crate::obd::vin::decode_vin(&vin);

        let model = super::database::find_vehicle_model_sync(&vin_info.make);
        let info = VehicleInfo {
            vin: vin_info.vin,
            make: vin_info.make,
            model: model.unwrap_or_default(),
            year: vin_info.year,
            protocol: conn.protocol.clone(),
            elm_version: conn.elm_version.clone(),
        };
        Ok((conn, info))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?;

    let (conn, info) = result?;
    let mut guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    *guard = ConnectionMode::Real(conn);
    Ok(info)
}

/// Scan for WiFi ELM327 adapters on common addresses
#[command]
pub async fn scan_wifi() -> Vec<serde_json::Value> {
    tokio::task::spawn_blocking(|| {
        let endpoints = transport::default_wifi_endpoints();
        let mut found = Vec::new();

        for (host, port) in &endpoints {
            crate::obd::dev_log::log_debug("connection", &format!("Probing WiFi {}:{}", host, port));
            match transport::WiFiTransport::new(host, *port, 1500) {
                Ok(mut t) => {
                    crate::obd::dev_log::log_info("connection", &format!("WiFi adapter found at {}:{}", host, port));
                    t.close();
                    found.push(serde_json::json!({
                        "host": host,
                        "port": port,
                        "name": format!("WiFi ELM327 ({}:{})", host, port),
                    }));
                }
                Err(_) => continue,
            }
        }

        found
    })
    .await
    .unwrap_or_else(|e| {
        crate::obd::dev_log::log_error("connection", &format!("scan_wifi task failed: {}", e));
        Vec::new()
    })
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


/// Set VIN manually and decode vehicle info from it
#[command]
pub fn set_manual_vin(vin: String) -> Result<VehicleInfo, String> {
    let vin = vin.trim().to_uppercase();

    // Validate: exactly 17 alphanumeric chars, no I/O/Q
    if !vin_parser::is_valid_vin(&vin) {
        return Err(err_msg(
            "Le VIN doit contenir 17 caractères alphanumériques valides (sans I, O, Q)",
            "VIN must be 17 valid alphanumeric characters (no I, O, Q)",
        ));
    }

    let vin_info = crate::obd::vin::decode_vin(&vin);

    // Preserve protocol/elm_version from current connection if available
    let (protocol, elm_version) = {
        let guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        match &*guard {
            ConnectionMode::Real(conn) => (conn.protocol.clone(), conn.elm_version.clone()),
            ConnectionMode::Demo => ("ISO 15765-4 CAN (Simulated)".to_string(), "Demo v1.0".to_string()),
            ConnectionMode::Disconnected => ("".to_string(), "".to_string()),
        }
    };

    crate::obd::dev_log::log_info("connection", &format!(
        "Manual VIN set: {}, Make: {}, Country: {}, Year: {}",
        vin_info.vin, vin_info.make, vin_info.country, vin_info.year
    ));

    let model = super::database::find_vehicle_model_sync(&vin_info.make);
    Ok(VehicleInfo {
        vin: vin_info.vin,
        make: vin_info.make,
        model: model.unwrap_or_default(),
        year: vin_info.year,
        protocol,
        elm_version,
    })
}

/// Check if an IP address is private/link-local (safe for local ELM327 adapters)
fn is_private_ip(host: &str) -> bool {
    if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
        ip.is_private() || ip.is_link_local() || ip.is_loopback()
    } else {
        // Allow "localhost" as special case
        host == "localhost"
    }
}

/// Check if VIN cache exists
#[command]
pub fn has_vin_cache(vin: String) -> bool {
    crate::obd::vin_cache::has_cache(&vin)
}

/// Clear VIN cache for a specific VIN
#[command]
pub fn clear_vin_cache(vin: String) -> Result<(), String> {
    crate::obd::vin_cache::clear_cache(&vin)?;
    super::dashboard::reset_discovered_params_inner();
    crate::obd::dev_log::log_info("connection", &format!("VIN cache cleared for {}", vin));
    Ok(())
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

    #[test]
    fn test_is_private_ip_localhost() {
        assert!(is_private_ip("localhost"));
    }

    #[test]
    fn test_is_private_ip_loopback() {
        assert!(is_private_ip("127.0.0.1"));
    }

    #[test]
    fn test_is_private_ip_private_range_192() {
        assert!(is_private_ip("192.168.1.1"));
    }

    #[test]
    fn test_is_private_ip_private_range_10() {
        assert!(is_private_ip("10.0.0.1"));
    }

    #[test]
    fn test_is_private_ip_private_range_172() {
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("172.31.255.255"));
    }

    #[test]
    fn test_is_private_ip_link_local() {
        assert!(is_private_ip("169.254.1.1"));
    }

    #[test]
    fn test_is_private_ip_public() {
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("1.1.1.1"));
    }


    #[test]
    fn test_obd_busy_guard_try_acquire() {
        set_obd_busy(false);
        let guard = OBDBusyGuard::try_acquire();
        assert!(guard.is_ok());
        assert!(is_obd_busy());
        drop(guard);
        assert!(!is_obd_busy());
    }

    #[test]
    fn test_obd_busy_guard_try_acquire_fails_when_busy() {
        set_obd_busy(false);
        let _guard1 = OBDBusyGuard::try_acquire().unwrap();
        assert!(is_obd_busy());
        let guard2 = OBDBusyGuard::try_acquire();
        assert!(guard2.is_err());
        assert!(is_obd_busy());
    }

    #[test]
    fn test_obd_busy_guard_auto_release_on_drop() {
        set_obd_busy(false);
        {
            let _guard = OBDBusyGuard::try_acquire().unwrap();
            assert!(is_obd_busy());
        }
        assert!(!is_obd_busy());
    }
}

