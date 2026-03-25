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
        ConnectionMode::Demo => Err("Demo mode – no real connection".to_string()),
        ConnectionMode::Disconnected => Err("Not connected".to_string()),
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
            return Err("Already connected".to_string());
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

        let mut last_error = String::new();
        for &baud in &baud_rates {
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

                    let info = VehicleInfo {
                        vin: vin_info.vin,
                        make: vin_info.make,
                        model: String::new(),
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
    let mut guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    if let ConnectionMode::Real(ref mut conn) = *guard {
        conn.disconnect();
    }
    *guard = ConnectionMode::Disconnected;
    crate::obd::dev_log::log_info("connection", "Disconnected from OBD adapter");
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
        ConnectionMode::Demo => ConnectionStatus::Connected,
        ConnectionMode::Real(_) => ConnectionStatus::Connected,
    };
    crate::obd::dev_log::log_debug("connection", &format!("Status requested: {:?}", status));
    status
}

/// Parse VIN from Mode 09 response (multi-frame compatible)
fn parse_vin_response(response: &str) -> String {
    let bytes: Vec<u8> = response
        .lines()
        .flat_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Multi-frame: each line starts with "49 02 XX" — skip those 3 tokens
            if parts.len() > 3 && parts[0] == "49" && parts[1] == "02" {
                parts[3..].iter()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect::<Vec<u8>>()
            } else {
                // Fallback: try to parse all hex bytes
                parts.iter()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect::<Vec<u8>>()
            }
        })
        .collect();

    let vin: String = String::from_utf8(bytes)
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    // VIN must be exactly 17 characters
    if vin.len() == 17 { vin } else { String::new() }
}

/// Connect via WiFi (ELM327 WiFi adapter)
#[command]
pub async fn connect_wifi(host: String, port: u16) -> Result<VehicleInfo, String> {
    {
        let guard = CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(*guard, ConnectionMode::Disconnected) {
            return Err("Already connected".to_string());
        }
    }

    crate::obd::dev_log::log_info("connection", &format!("WiFi connect: {}:{}", host, port));

    let result: Result<(Elm327Connection, VehicleInfo), String> = tokio::task::spawn_blocking(move || {
        // Create WiFi transport
        let _wifi = transport::WiFiTransport::new(&host, port, 5000)?;
        crate::obd::dev_log::log_info("connection", "WiFi transport established");

        // For now, use the standard ELM327 connection over serial
        // TODO: refactor Elm327Connection to accept OBDTransport trait
        // Workaround: many WiFi ELM327s also appear as a virtual serial port
        Err("WiFi direct connection not yet integrated with ELM327 protocol layer – use the virtual serial port (e.g. /dev/cu.OBDII-WiFi)".to_string())
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
        "available": false, // Not yet implemented
    }));

    types
}
