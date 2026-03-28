use tauri::command;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};

/// Get extended vehicle information (CalID, CVN, ECU Name) via Mode 09 PIDs
#[command]
pub async fn get_vehicle_info_extended() -> Result<crate::models::VehicleInfoExtended, String> {
    tokio::task::spawn_blocking(move || {
        let mut result = crate::models::VehicleInfoExtended { calid: None, cvn: None, ecu_name: None };

        if is_demo() {
            result.calid = Some("DEMO_CALID_001".to_string());
            result.cvn = Some("A1B2C3D4".to_string());
            result.ecu_name = Some("Demo ECU".to_string());
            return Ok(result);
        }

        // Mode 09 PID 04 - Calibration ID
        if let Ok(resp) = with_real_connection(|conn| {
            conn.send_command_timeout("0904", 3000)
        }) {
            let cleaned = resp.lines()
                .filter(|l| !l.starts_with("SEARCHING") && !l.contains("NO DATA"))
                .collect::<Vec<_>>().join(" ");
            let bytes: Vec<u8> = cleaned.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            // Skip response header bytes (49 04 XX)
            let data_bytes: Vec<u8> = bytes.iter().copied()
                .skip_while(|&b| b != 0x49).skip(3)
                .filter(|&b| b >= 0x20 && b <= 0x7E)
                .collect();
            if !data_bytes.is_empty() {
                result.calid = Some(String::from_utf8_lossy(&data_bytes).trim().to_string());
            }
        }

        // Mode 09 PID 06 - CVN
        if let Ok(resp) = with_real_connection(|conn| {
            conn.send_command_timeout("0906", 3000)
        }) {
            let bytes: Vec<u8> = resp.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            let hex: Vec<String> = bytes.iter().skip(3).map(|b| format!("{:02X}", b)).collect();
            if !hex.is_empty() {
                result.cvn = Some(hex.join(""));
            }
        }

        // Mode 09 PID 0A - ECU Name
        if let Ok(resp) = with_real_connection(|conn| {
            conn.send_command_timeout("090A", 3000)
        }) {
            let bytes: Vec<u8> = resp.split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            let data_bytes: Vec<u8> = bytes.iter().copied()
                .skip(3)
                .filter(|&b| b >= 0x20 && b <= 0x7E)
                .collect();
            if !data_bytes.is_empty() {
                result.ecu_name = Some(String::from_utf8_lossy(&data_bytes).trim().to_string());
            }
        }

        dev_log::log_info("ecu", &format!("Vehicle info extended: calid={:?}, cvn={:?}, ecu_name={:?}", result.calid, result.cvn, result.ecu_name));
        Ok(result)
    }).await.unwrap_or_else(|_| Ok(crate::models::VehicleInfoExtended { calid: None, cvn: None, ecu_name: None }))
}
