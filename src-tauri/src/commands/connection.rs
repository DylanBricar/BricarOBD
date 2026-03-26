use tauri::command;
use crate::models::{ConnectionStatus, PortInfo, VehicleInfo};
use crate::obd::Elm327Connection;
use crate::obd::transport::{self, OBDTransport};
use std::sync::Mutex;

enum ConnectionMode {
    Disconnected,
    Demo,
    Real(crate::obd::Elm327Connection),
}

static CONNECTION: Mutex<ConnectionMode> = Mutex::new(ConnectionMode::Disconnected);
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
                    let vin = parse_vin_response(&vin_response);
                    let vin_info = crate::obd::vin::decode_vin(&vin);

                    crate::obd::dev_log::log_info("connection", &format!("VIN decoded: {}, Make: {}, Year: {}", vin_info.vin, vin_info.make, vin_info.year));

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
        crate::obd::dev_log::log_error("connection", &format!("Connection failed at all baud rates: {}", last_error));
        Err(format!("Connection failed at all baud rates: {}", last_error))
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

/// Strategy 1: Standard multi-frame "49 02 XX <data>"
fn parse_vin_multiframe(response: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for line in response.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 3 && parts[0] == "49" && parts[1] == "02" {
            // Skip "49 02 XX" (service + PID + message counter)
            bytes.extend(
                parts[3..].iter()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
            );
        }
    }
    bytes
}

/// Strategy 2: Single-line response without frame counter "49 02 <all data>"
fn parse_vin_singleline(response: &str) -> Vec<u8> {
    let all_parts: Vec<&str> = response.split_whitespace().collect();
    if all_parts.len() > 2 && all_parts[0] == "49" && all_parts[1] == "02" {
        return all_parts[2..].iter()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
    }
    Vec::new()
}

/// Strategy 3: No-space hex "4902XXXXXXXXXXXX..."
fn parse_vin_nospace(response: &str) -> Vec<u8> {
    let hex_str = response.replace(" ", "").replace("\n", "");
    if let Some(start) = hex_str.find("4902") {
        let data_start = start + 4;
        // Skip message counter byte (2 chars)
        let data_start = if data_start + 2 <= hex_str.len() { data_start + 2 } else { data_start };
        return (data_start..hex_str.len()).step_by(2)
            .filter_map(|i| {
                if i + 2 <= hex_str.len() {
                    u8::from_str_radix(&hex_str[i..i+2], 16).ok()
                } else {
                    None
                }
            })
            .collect();
    }
    Vec::new()
}

/// Strategy 4: CAN multi-frame with headers "7E8 10 14 49 02 01 XX XX XX..."
fn parse_vin_can_header(response: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for line in response.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Look for "49 02" anywhere in the line (skip header bytes)
        if let Some(pos) = parts.iter().position(|&p| p == "49") {
            if pos + 1 < parts.len() && parts[pos + 1] == "02" {
                let data_start = pos + 3; // Skip "49 02 XX"
                if data_start < parts.len() {
                    bytes.extend(
                        parts[data_start..].iter()
                            .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    );
                }
            }
        }
    }
    bytes
}

/// Strategy 5: Raw fallback — try to parse all hex bytes and find ASCII VIN pattern
fn parse_vin_raw_fallback(response: &str) -> Vec<u8> {
    response
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect()
}

/// Convert raw bytes to a validated 17-char VIN string
fn validate_vin(bytes: Vec<u8>) -> String {
    // Convert to ASCII, filter to alphanumeric only
    let vin: String = String::from_utf8(bytes)
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    // VIN must be exactly 17 characters
    if vin.len() == 17 {
        vin
    } else if vin.len() > 17 {
        // Some adapters pad with extra bytes — try to extract 17-char VIN
        // VIN never contains I, O, Q
        for start in 0..=(vin.len() - 17) {
            let candidate = &vin[start..start + 17];
            if candidate.chars().all(|c| c.is_ascii_alphanumeric() && c != 'I' && c != 'O' && c != 'Q') {
                return candidate.to_string();
            }
        }
        String::new()
    } else {
        crate::obd::dev_log::log_warn("connection", &format!("VIN parse: got {} chars instead of 17: {}", vin.len(), vin));
        String::new()
    }
}

/// Parse VIN from Mode 09 response (multi-frame compatible, handles many adapter formats)
fn parse_vin_response(response: &str) -> String {
    if response.is_empty() || response.contains("NO DATA") || response.contains("ERROR") {
        return String::new();
    }
    let strategies: &[fn(&str) -> Vec<u8>] = &[
        parse_vin_multiframe,
        parse_vin_singleline,
        parse_vin_nospace,
        parse_vin_can_header,
        parse_vin_raw_fallback,
    ];
    for strategy in strategies {
        let bytes = strategy(response);
        if !bytes.is_empty() {
            return validate_vin(bytes);
        }
    }
    String::new()
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
        let vin = parse_vin_response(&vin_response);
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
    .unwrap_or_default()
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
    if vin.len() != 17 {
        return Err(err_msg(
            &format!("Le VIN doit contenir 17 caractères (reçu {})", vin.len()),
            &format!("VIN must be 17 characters (got {})", vin.len()),
        ));
    }
    if !vin.chars().all(|c| c.is_ascii_alphanumeric() && c != 'I' && c != 'O' && c != 'Q') {
        return Err(err_msg(
            "Le VIN contient des caractères invalides (I, O, Q interdits)",
            "VIN contains invalid characters (I, O, Q not allowed)",
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
