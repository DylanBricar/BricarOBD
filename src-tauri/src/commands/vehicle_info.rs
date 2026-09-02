use crate::commands::connection::{is_demo, with_real_connection};
use crate::commands::OBDBusyGuard;
use crate::obd::dev_log;
use tauri::command;

fn text_mode09_payload(bytes: &[u8]) -> Option<String> {
    let value = bytes
        .iter()
        .copied()
        .skip_while(|byte| !byte.is_ascii_graphic() && *byte != b' ')
        .filter(|byte| byte.is_ascii_graphic() || *byte == b' ')
        .collect::<Vec<_>>();
    let value = String::from_utf8_lossy(&value).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn cvn_mode09_payload(bytes: &[u8]) -> Option<String> {
    let value = if bytes.len() >= 5 && (bytes.len() - 1).is_multiple_of(4) && bytes[0] <= 0x20 {
        &bytes[1..]
    } else {
        bytes
    };
    (!value.is_empty()).then(|| {
        value
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>()
    })
}

/// Get extended vehicle information (CalID, CVN, ECU Name) via Mode 09 PIDs
#[command]
pub async fn get_vehicle_info_extended() -> Result<crate::models::VehicleInfoExtended, String> {
    tokio::task::spawn_blocking(move || {
        let mut result = crate::models::VehicleInfoExtended {
            calid: None,
            cvn: None,
            ecu_name: None,
        };

        if is_demo() {
            result.calid = Some("DEMO_CALID_001".to_string());
            result.cvn = Some("A1B2C3D4".to_string());
            result.ecu_name = Some("Demo ECU".to_string());
            return Ok(result);
        }

        let _guard = OBDBusyGuard::acquire_with_wait(10)?;
        with_real_connection(|conn| conn.reset_headers())?;

        // Mode 09 PID 04 - Calibration ID
        if let Ok(bytes) = with_real_connection(|conn| conn.query_pid(0x09, 0x04)) {
            result.calid = text_mode09_payload(&bytes);
        }

        // Mode 09 PID 06 - CVN
        if let Ok(bytes) = with_real_connection(|conn| conn.query_pid(0x09, 0x06)) {
            result.cvn = cvn_mode09_payload(&bytes);
        }

        // Mode 09 PID 0A - ECU Name
        if let Ok(bytes) = with_real_connection(|conn| conn.query_pid(0x09, 0x0A)) {
            result.ecu_name = text_mode09_payload(&bytes);
        }

        dev_log::log_info(
            "ecu",
            &format!(
                "Vehicle info extended: calid={:?}, cvn={:?}, ecu_name={:?}",
                result.calid, result.cvn, result.ecu_name
            ),
        );
        Ok(result)
    })
    .await
    .unwrap_or(Ok(crate::models::VehicleInfoExtended {
        calid: None,
        cvn: None,
        ecu_name: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_mode09_record_number_from_text() {
        assert_eq!(
            text_mode09_payload(b"\x01BOSCH EDC17   "),
            Some("BOSCH EDC17".to_string())
        );
    }

    #[test]
    fn strips_mode09_record_number_from_cvn() {
        assert_eq!(
            cvn_mode09_payload(&[1, 0xA1, 0xB2, 0xC3, 0xD4]),
            Some("A1B2C3D4".to_string())
        );
    }
}
