pub mod obd;
pub mod db;
pub mod commands;
pub mod models;

use commands::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    obd::dev_log::init_log_file();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Auto-initialize the SQLite database at app startup
            let resource_dir = app.path().resource_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            // Try multiple paths to find the DB
            let possible_paths = vec![
                resource_dir.join("data").join("bricarobd.db"),
                resource_dir.join("bricarobd.db"),
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data").join("bricarobd.db"),
            ];

            for db_path in &possible_paths {
                if db_path.exists() {
                    match database::init_database_internal(db_path) {
                        Ok(stats) => {
                            tracing::info!("Database auto-initialized: {:?} from {}", stats, db_path.display());
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to init DB from {}: {}", db_path.display(), e);
                        }
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection
            connection::list_serial_ports,
            connection::connect_obd,
            connection_wifi_vin::connect_wifi,
            connection_wifi_vin::scan_wifi,
            connection::get_connection_types,
            connection::disconnect_obd,
            connection::connect_demo,
            connection::get_connection_status,
            connection_wifi_vin::set_manual_vin,
            connection::set_language,
            connection_wifi_vin::has_vin_cache,
            connection_wifi_vin::clear_vin_cache,
            // Dashboard
            dashboard::get_pid_data,
            dashboard::get_all_pids,
            dashboard::get_pid_data_extended,
            dashboard::reset_pid_blacklist,
            discover_vehicle_params,
            dashboard::get_battery_voltage,
            get_discovery_progress,
            // DTC
            dtc::read_all_dtcs,
            dtc::clear_dtcs,
            dtc::export_dtcs,
            // ECU
            ecu::scan_ecus,
            ecu::read_did,
            ecu::get_monitors,
            ecu::check_anomalies,
            ecu::get_generic_ecus,
            ecu::get_manufacturer_dids,
            ecu::get_all_manufacturer_dids,
            ecu::get_vehicle_info_extended,
            // Advanced
            ecu::send_raw_command,
            ecu::get_advanced_categories,
            ecu::get_advanced_manufacturer_groups,
            // Diagnostic (Mode 06, Mode 02)
            commands::diagnostic::get_mode06_results,
            commands::diagnostic::get_freeze_frame,
            // Database (3.27M operations pre-built)
            database::get_database_stats,
            database::search_operations,
            database::get_vehicle_operations,
            database::get_read_operations,
            database::get_write_operations,
            database::get_vehicle_profiles,
            database::get_profile_ecus,
            database::search_ecu_catalog,
            database::get_sessions_cmd,
            database::delete_session_cmd,
            database::save_session_cmd,
            database::find_vehicle_model,
            // Settings + File export
            settings::get_settings,
            settings::save_settings,
            settings::get_dev_logs,
            settings::get_dev_log_count,
            settings::clear_dev_logs,
            settings::add_dev_log,
            settings::add_dev_logs_batch,
            settings::save_csv_file,
            settings::read_csv_file,
            settings::list_exports,
            settings::open_exports_folder,
            settings::get_log_dir,
            settings::open_log_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BricarOBD");
}
