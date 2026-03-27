pub mod connection;
pub mod connection_helpers;
pub mod connection_wifi_vin;
pub mod dashboard;
pub mod dashboard_did;
pub mod dashboard_discovery;
pub mod dtc;
pub mod ecu;
pub mod ecu_scan;
pub mod database;
pub mod settings;
pub mod diagnostic;
pub mod mode06_names;
pub mod vin_parser;

// Re-export discovery commands so they're available for tauri handler
pub use dashboard_discovery::{discover_vehicle_params, get_discovery_progress};
// Re-export connection helpers for convenience
pub use connection_helpers::{OBDBusyGuard, set_obd_busy, is_obd_busy, is_private_ip};
// Re-export WiFi/VIN commands for tauri handler
pub use connection_wifi_vin::{connect_wifi, scan_wifi, set_manual_vin, has_vin_cache, clear_vin_cache};
