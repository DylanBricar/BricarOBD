use tauri::command;
use crate::models::VehicleInfo;

/// Scan for BLE ELM327 devices
#[command]
pub async fn scan_ble(timeout_ms: Option<u64>) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(feature = "mobile")]
    {
        let timeout = timeout_ms.unwrap_or(5000);
        let devices = crate::obd::transport_ble::scan_ble_devices(timeout).await?;
        Ok(devices
            .into_iter()
            .map(|d| {
                serde_json::json!({
                    "name": d.name,
                    "address": d.address,
                })
            })
            .collect())
    }
    #[cfg(not(feature = "mobile"))]
    {
        let _ = timeout_ms;
        Err("BLE scanning is only available on mobile".to_string())
    }
}

/// Connect to an ELM327 adapter via Bluetooth BLE
#[command]
pub async fn connect_ble(device_name: String) -> Result<VehicleInfo, String> {
    {
        let guard = super::connection::CONNECTION.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(*guard, super::connection::ConnectionMode::Disconnected) {
            return Err(super::connection::err_msg("Déjà connecté", "Already connected"));
        }
    }

    crate::obd::dev_log::log_info("connection", &format!("BLE connect: {}", device_name));

    let result: Result<(crate::obd::Elm327Connection, VehicleInfo), String> =
        tokio::task::spawn_blocking(move || {
            #[cfg(feature = "mobile")]
            {
                let ble = crate::obd::transport::BleTransport::new(&device_name, 5000)?;
                crate::obd::dev_log::log_info(
                    "connection",
                    "BLE transport established, starting ELM327 init...",
                );

                let mut conn = crate::obd::Elm327Connection::new();
                conn.connect_transport(Box::new(ble))?;

                crate::obd::dev_log::log_info(
                    "connection",
                    &format!("BLE ELM327 connected: {}", conn.elm_version),
                );

                let vin_response = conn.send_command("0902").unwrap_or_default();
                let vin = super::vin_parser::parse_vin_response(&vin_response);
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
            }
            #[cfg(not(feature = "mobile"))]
            {
                let _ = device_name;
                Err("BLE connection is only available on mobile".to_string())
            }
        })
        .await
        .map_err(|e| {
            crate::obd::dev_log::log_error("connection", &format!("BLE task error: {}", e));
            format!("Task error: {}", e)
        })?;

    let (conn, info) = result?;
    let mut guard = super::connection::CONNECTION
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *guard = super::connection::ConnectionMode::Real(conn);
    Ok(info)
}
