use tauri::command;
use crate::models::{VehicleInfo};
use crate::obd::transport::{self, OBDTransport};
use super::vin_parser;
use super::connection_helpers::is_private_ip;

/// Connect via WiFi (ELM327 WiFi adapter) — now fully integrated with ELM327 protocol
#[command]
pub async fn connect_wifi(host: String, port: u16) -> Result<VehicleInfo, String> {
    {
        let guard = super::connection::CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(*guard, super::connection::ConnectionMode::Disconnected) {
            return Err(super::connection::err_msg("Déjà connecté", "Already connected"));
        }
    }

    // Validate host is a private/link-local IP to prevent connecting to arbitrary servers
    if !is_private_ip(&host) {
        crate::obd::dev_log::log_warn("connection", &format!("WiFi connect denied: {} is not a private IP", host));
        return Err(super::connection::err_msg("L'hôte WiFi doit être une adresse IP locale/privée", "WiFi host must be a local/private IP address"));
    }

    crate::obd::dev_log::log_info("connection", &format!("WiFi connect: {}:{}", host, port));

    let result: Result<(crate::obd::Elm327Connection, VehicleInfo), String> = tokio::task::spawn_blocking(move || {
        // Create WiFi transport with generous timeout (WiFi has latency)
        let wifi = transport::WiFiTransport::new(&host, port, 8000)?;
        crate::obd::dev_log::log_info("connection", "WiFi transport established, starting ELM327 init...");

        // Wire WiFi transport into the full ELM327 protocol layer
        let mut conn = crate::obd::Elm327Connection::new();
        conn.connect_transport(Box::new(wifi))?;

        crate::obd::dev_log::log_info("connection", &format!("WiFi ELM327 connected: {}", conn.elm_version));

        // Read VIN
        let vin_response = conn.send_command("0902").unwrap_or_default();
        let vin = vin_parser::parse_vin_response(&vin_response);
        let vin_info = crate::obd::vin::decode_vin(&vin);
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
        Ok((conn, info))
    })
    .await
    .map_err(|e| {
        crate::obd::dev_log::log_error("connection", &format!("WiFi task error: {}", e));
        format!("Task error: {}", e)
    })?;

    let (conn, info) = result?;
    let mut guard = super::connection::CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
    *guard = super::connection::ConnectionMode::Real(conn);
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

/// Set VIN manually and decode vehicle info from it
#[command]
pub fn set_manual_vin(vin: String) -> Result<VehicleInfo, String> {
    let vin = vin.trim().to_uppercase();

    // Validate: exactly 17 alphanumeric chars, no I/O/Q
    if !vin_parser::is_valid_vin(&vin) {
        return Err(super::connection::err_msg(
            "Le VIN doit contenir 17 caractères alphanumériques valides (sans I, O, Q)",
            "VIN must be 17 valid alphanumeric characters (no I, O, Q)",
        ));
    }

    let vin_info = crate::obd::vin::decode_vin(&vin);

    // Preserve protocol/elm_version from current connection if available
    let (protocol, elm_version) = {
        let guard = super::connection::CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        match &*guard {
            super::connection::ConnectionMode::Real(conn) => (conn.protocol.clone(), conn.elm_version.clone()),
            super::connection::ConnectionMode::Demo => ("ISO 15765-4 CAN (Simulated)".to_string(), "Demo v1.0".to_string()),
            super::connection::ConnectionMode::Disconnected => ("".to_string(), "".to_string()),
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

/// Check if VIN cache exists
#[command]
pub fn has_vin_cache(vin: String) -> bool {
    crate::obd::vin_cache::has_cache(&vin)
}

/// Clear VIN cache for a specific VIN
#[command]
pub fn clear_vin_cache(vin: String) -> Result<(), String> {
    crate::obd::vin_cache::clear_cache(&vin)?;
    super::dashboard_discovery::reset_discovered_params_inner();
    crate::obd::dev_log::log_info("connection", &format!("VIN cache cleared for {}", vin));
    Ok(())
}
