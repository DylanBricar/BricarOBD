pub mod catalog;
pub mod connection;
pub mod connection_ble;
pub mod connection_helpers;
pub mod connection_wifi_vin;
pub mod dashboard;
pub mod dashboard_did;
pub mod dashboard_discovery;
pub mod database;
pub mod diagnostic;
pub mod dtc;
pub mod dtc_clear;
pub mod ecu;
pub mod ecu_scan;
pub mod mil;
pub mod mode06_names;
pub mod settings;
pub mod vehicle_info;
pub mod vin_parser;

/// Common UDS ECU addresses for DTC operations
pub const UDS_ECU_ADDRESSES: &[&str] = &[
    "7E0", "7E1", "7E2", "7E3", "7E4", "75D", "7C0", "7C1", "7A0", "740", "710", "714",
];

// Re-export discovery commands so they're available for tauri handler
pub use dashboard_discovery::{discover_vehicle_params, get_discovery_progress};
// Re-export connection helpers for convenience
pub use connection_helpers::{is_obd_busy, is_private_ip, set_obd_busy, OBDBusyGuard};
// Re-export WiFi/VIN commands for tauri handler
pub use connection_wifi_vin::{
    clear_vin_cache, connect_wifi, has_vin_cache, scan_wifi, set_manual_vin,
};
