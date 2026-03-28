use tauri::command;
use crate::models::MilStatus;
use crate::obd::dev_log;
use crate::commands::connection::{is_demo, with_real_connection};

/// Get MIL status from PID 01 byte A — indicates if check engine light is on
#[command]
pub async fn get_mil_status() -> Result<MilStatus, String> {
    tokio::task::spawn_blocking(move || {
        if is_demo() {
            dev_log::log_debug("mil", "Demo mode: returning simulated MIL status");
            return Ok(MilStatus { mil_on: false, dtc_count: 0 });
        }

        dev_log::log_info("mil", "Real mode: reading MIL status from Mode 01 PID 01");

        // Ensure headers are reset to broadcast
        let _ = with_real_connection(|conn| conn.reset_headers());

        let response = match with_real_connection(|conn| conn.query_pid(0x01, 0x01)) {
            Ok(bytes) if !bytes.is_empty() => {
                dev_log::log_rx("0101", &format!("{:02X?}", bytes));
                bytes
            },
            _ => {
                // Retry after wake-up
                dev_log::log_warn("mil", "PID 01 failed, trying wake-up + retry...");
                let _ = with_real_connection(|conn| { conn.tester_present(); Ok(()) });
                std::thread::sleep(std::time::Duration::from_millis(300));

                match with_real_connection(|conn| conn.query_pid(0x01, 0x01)) {
                    Ok(bytes) if !bytes.is_empty() => bytes,
                    _ => {
                        dev_log::log_warn("mil", "Mode 01 PID 01 read failed after retry");
                        return Ok(MilStatus { mil_on: false, dtc_count: 0 });
                    }
                }
            }
        };

        let byte_a = response[0];
        Ok(MilStatus {
            mil_on: byte_a & 0x80 != 0,
            dtc_count: byte_a & 0x7F,
        })
    }).await.map_err(|e| format!("Task error: {}", e))?
}
