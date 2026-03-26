use std::time::Duration;
use tracing::{error, info};
use crate::obd::dev_log;
use crate::obd::transport::OBDTransport;
use crate::models::PortInfo;

mod init_strategies;
mod protocol;
mod pid_discovery;
mod elm_io;
mod queries;
mod health;

/// ELM327 connection configuration
pub struct ConnectionConfig {
    pub port: String,
    pub baud_rate: u32,
    pub timeout_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 38400,
            timeout_ms: 5000,
        }
    }
}

/// Detected chip type — affects timing and command compatibility
#[derive(Debug, Clone, PartialEq)]
pub enum ChipType {
    GenuineElm327,   // Real ELM327 v1.4b/v1.5b
    Stn1110,         // OBDLink / STN1110 (faster, more reliable)
    Stn2120,         // STN2120 (BLE capable)
    CloneV15,        // Chinese ELM327 v1.5 clone (common, limited)
    CloneV21,        // Chinese ELM327 v2.1 clone (even more limited)
    Unknown,
}

/// ELM327 OBD-II adapter connection with resilience layer
/// Works over any transport: Serial USB, WiFi TCP, or BLE
pub struct Elm327Connection {
    pub(super) transport: Option<Box<dyn OBDTransport>>,
    pub config: ConnectionConfig,
    pub protocol: String,
    pub protocol_num: String,        // Raw protocol number (e.g. "6" for CAN 500k)
    pub elm_version: String,
    pub chip_type: ChipType,
    pub is_clone: bool,
    pub supported_pids: Vec<u8>,     // PIDs 0x01-0x60 supported bitmap
    pub supported_pids_ext: Vec<u8>, // PIDs 0x61-0xC0 if available
    pub voltage: Option<f64>,        // Battery voltage from ATRV
    pub(super) headers_on: bool,                // Track ATH state
    pub(super) last_command_time: std::time::Instant,
    pub(super) consecutive_errors: u32,         // Track errors for adaptive recovery
}

impl Elm327Connection {
    pub fn new() -> Self {
        Self {
            transport: None,
            config: ConnectionConfig::default(),
            protocol: String::new(),
            protocol_num: String::new(),
            elm_version: String::new(),
            chip_type: ChipType::Unknown,
            is_clone: false,
            supported_pids: Vec::new(),
            supported_pids_ext: Vec::new(),
            voltage: None,
            headers_on: false,
            last_command_time: std::time::Instant::now(),
            consecutive_errors: 0,
        }
    }

    /// List available serial ports (desktop only — mobile uses BLE/WiFi)
    #[cfg(feature = "desktop")]
    pub fn list_ports() -> Vec<PortInfo> {
        match serialport::available_ports() {
            Ok(ports) => ports
                .into_iter()
                .map(|p| PortInfo {
                    description: match &p.port_type {
                        serialport::SerialPortType::UsbPort(usb) => {
                            format!("{} {}",
                                usb.manufacturer.as_deref().unwrap_or(""),
                                usb.product.as_deref().unwrap_or("")
                            ).trim().to_string()
                        }
                        serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                        _ => "Serial".to_string(),
                    },
                    name: p.port_name,
                })
                .collect(),
            Err(e) => {
                error!("Failed to list serial ports: {}", e);
                Vec::new()
            }
        }
    }

    /// List available serial ports — stub for mobile (no serial ports)
    #[cfg(not(feature = "desktop"))]
    pub fn list_ports() -> Vec<PortInfo> {
        Vec::new()
    }

    /// Connect to ELM327 via serial port with full resilience (desktop only)
    #[cfg(feature = "desktop")]
    pub fn connect(&mut self, port: &str, baud_rate: u32) -> Result<(), String> {
        info!("Connecting to {} at {} baud", port, baud_rate);
        dev_log::log_info("obd", &format!("Opening serial port {} @ {} baud", port, baud_rate));

        let serial_transport = crate::obd::transport::SerialTransport::new(port, baud_rate, self.config.timeout_ms)?;
        self.transport = Some(Box::new(serial_transport));
        self.config.port = port.to_string();
        self.config.baud_rate = baud_rate;

        // Try multiple init strategies
        if let Err(e) = self.init_with_resilience() {
            self.transport = None;
            return Err(e);
        }

        // Read battery voltage (non-critical)
        self.read_voltage();

        // Discover all supported PID ranges
        self.discover_supported_pids();

        dev_log::log_info("obd", &format!(
            "Connected: ELM={}, Chip={:?}, Protocol={} ({}), Clone={}, Voltage={:.1}V, PIDs={}",
            self.elm_version, self.chip_type, self.protocol, self.protocol_num,
            self.is_clone, self.voltage.unwrap_or(0.0), self.supported_pids.len()
        ));
        info!("Connected: ELM={}, Protocol={}, Clone={}",
            self.elm_version, self.protocol, self.is_clone);
        Ok(())
    }

    /// Connect to ELM327 via serial port — stub for mobile
    #[cfg(not(feature = "desktop"))]
    pub fn connect(&mut self, _port: &str, _baud_rate: u32) -> Result<(), String> {
        Err("Serial port not available on mobile — use WiFi or BLE".to_string())
    }

    /// Connect to ELM327 via any pre-built transport (WiFi, BLE, etc.)
    pub fn connect_transport(&mut self, transport: Box<dyn OBDTransport>) -> Result<(), String> {
        let transport_type = transport.transport_type();
        info!("Connecting via {:?} transport", transport_type);
        dev_log::log_info("obd", &format!("Connecting via {:?} transport", transport_type));

        self.transport = Some(transport);

        // Try init
        if let Err(e) = self.init_with_resilience() {
            self.transport = None;
            return Err(e);
        }

        self.read_voltage();
        self.discover_supported_pids();

        dev_log::log_info("obd", &format!(
            "Connected via {:?}: ELM={}, Protocol={} ({}), PIDs={}",
            transport_type, self.elm_version, self.protocol, self.protocol_num, self.supported_pids.len()
        ));
        Ok(())
    }

    /// Detect chip type from ATZ/ATI response — affects timing and feature support
    fn detect_chip_type(&mut self, response: &str) {
        let upper = response.to_uppercase();

        if upper.contains("STN2120") || upper.contains("STN 2120") {
            self.chip_type = ChipType::Stn2120;
            self.elm_version = response.lines().find(|l| l.contains("STN")).unwrap_or("STN2120").trim().to_string();
        } else if upper.contains("STN1110") || upper.contains("STN 1110") || upper.contains("OBDLINK") {
            self.chip_type = ChipType::Stn1110;
            self.elm_version = response.lines().find(|l| l.contains("STN") || l.contains("OBD")).unwrap_or("STN1110").trim().to_string();
        } else if upper.contains("ELM327 V1.5") || upper.contains("ELM327 V1.4") {
            // Could be genuine or clone — check response timing/quality
            if upper.contains("V1.5A") || upper.contains("V1.4B") || upper.contains("V1.5B") {
                self.chip_type = ChipType::GenuineElm327;
            } else {
                // v1.5 without letter suffix is almost always a clone
                self.chip_type = ChipType::CloneV15;
                self.is_clone = true;
            }
            self.elm_version = response.lines().find(|l| l.to_uppercase().contains("ELM")).unwrap_or("ELM327").trim().to_string();
        } else if upper.contains("ELM327 V2.1") || upper.contains("ELM327 V2.2") {
            // v2.1/v2.2 are ALWAYS clones — real ELM327 never went past v2.0
            self.chip_type = ChipType::CloneV21;
            self.is_clone = true;
            self.elm_version = response.lines().find(|l| l.to_uppercase().contains("ELM")).unwrap_or("ELM327 Clone").trim().to_string();
        } else if upper.contains("ELM") || upper.contains("OBD") {
            self.chip_type = ChipType::Unknown;
            self.elm_version = response.lines().find(|l| !l.is_empty()).unwrap_or("Unknown").trim().to_string();
        } else {
            self.chip_type = ChipType::Unknown;
            self.elm_version = "Unknown".to_string();
        }

        dev_log::log_info("obd", &format!("Chip detected: {:?} — {}", self.chip_type, self.elm_version));
    }

    /// Configure adapter with optimal settings
    fn configure_adapter(&mut self) -> Result<(), String> {
        self.send_command("ATE0")?;    // Echo off
        self.send_command("ATL0")?;    // Linefeeds off
        self.send_command("ATS1")?;    // Spaces on (easier parsing)
        self.send_command("ATH0")?;    // Headers off (default)
        self.headers_on = false;

        // Adaptive timing — depends on chip type
        match self.chip_type {
            ChipType::Stn1110 | ChipType::Stn2120 => {
                // STN chips: aggressive adaptive timing, they handle it well
                let _ = self.send_command("ATAT2");
                let _ = self.send_command("ATST 32"); // 50 × 4ms = 200ms (STN is fast)
            }
            ChipType::GenuineElm327 => {
                let _ = self.send_command("ATAT1");   // Normal adaptive
                let _ = self.send_command("ATST 64"); // 100 × 4ms = 400ms
            }
            ChipType::CloneV15 | ChipType::CloneV21 => {
                // Clones: be conservative — some ignore ATAT, cap ATST at ~0x14
                let _ = self.send_command("ATAT1");
                let _ = self.send_command("ATST 96"); // 150 × 4ms = 600ms (generous for clones)
            }
            ChipType::Unknown => {
                let _ = self.send_command("ATAT1");
                let _ = self.send_command("ATST 64");
            }
        }

        Ok(())
    }

    /// Configure CAN flow control — critical for multi-frame responses
    fn configure_can_flow_control(&mut self) -> Result<(), String> {
        // CAN Auto-Format: handles multi-frame ISO-TP transparently
        let _ = self.send_command("ATCAF1");

        // CAN Flow Control: sends ISO-TP consecutive frame requests automatically
        let _ = self.send_command("ATCFC1");

        // Set flow control timing based on chip type
        match self.chip_type {
            ChipType::Stn1110 | ChipType::Stn2120 => {
                // STN: tight timing, handles it well
                // BS=0 (unlimited frames), STmin=10ms
                let _ = self.send_command("ATFCSM1"); // Flow control mode 1 (user defined)
                let _ = self.send_command("ATFCSD300000"); // BS=0, STmin=0 (let ECU decide)
            }
            _ => {
                // Genuine/Clone: let adapter decide flow control params
                // Don't set ATFCSM — use default auto-respond
            }
        }

        Ok(())
    }

    /// Set headers on/off — needed for ECU-specific communication
    pub fn set_headers(&mut self, on: bool) -> bool {
        let cmd = if on { "ATH1" } else { "ATH0" };
        if self.send_command(cmd).is_ok() {
            self.headers_on = on;
            true
        } else {
            false
        }
    }

    /// Set CAN header for a specific ECU address (e.g. "7E0")
    pub fn set_ecu_header(&mut self, address: &str) -> Result<(), String> {
        // Validate address is strictly hex (3-8 chars) to prevent AT command injection via \r
        if address.is_empty() || address.len() > 8 || !address.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("Invalid ECU address: {}", address));
        }
        if !self.headers_on {
            self.set_headers(true);
        }
        self.send_command(&format!("ATSH{}", address))?;
        Ok(())
    }

    /// Reset headers to broadcast mode
    pub fn reset_headers(&mut self) -> Result<(), String> {
        self.send_command("ATSH7DF")?;
        self.set_headers(false);
        Ok(())
    }

    /// Read battery voltage from adapter (ATRV)
    fn read_voltage(&mut self) {
        if let Ok(response) = self.send_command("ATRV") {
            // Response like "12.6V" or "14.1V"
            let cleaned: String = response.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            if let Ok(v) = cleaned.parse::<f64>() {
                if (6.0..=20.0).contains(&v) {
                    self.voltage = Some(v);
                    dev_log::log_info("obd", &format!("Battery voltage: {:.1}V", v));
                }
            }
        }
    }

    /// Get battery voltage (re-reads from adapter)
    pub fn get_voltage(&mut self) -> Option<f64> {
        self.read_voltage();
        self.voltage
    }

    /// Disconnect cleanly
    pub fn disconnect(&mut self) {
        if let Some(ref mut transport) = self.transport {
            // Send CR to cancel any pending command
            let _ = transport.write_bytes(b"\r");
            std::thread::sleep(Duration::from_millis(100));
            // Close protocol
            let _ = transport.write_bytes(b"ATPC\r");
            std::thread::sleep(Duration::from_millis(50));
            // Reset adapter
            let _ = transport.write_bytes(b"ATZ\r");
            std::thread::sleep(Duration::from_millis(100));
            transport.close();
        }
        self.transport = None;
        self.protocol.clear();
        self.protocol_num.clear();
        self.elm_version.clear();
        self.supported_pids.clear();
        self.supported_pids_ext.clear();
        self.voltage = None;
        self.consecutive_errors = 0;
        info!("Disconnected");
    }

    pub fn is_connected(&self) -> bool {
        self.transport.is_some()
    }

    /// Get consecutive error count (for frontend to display warning)
    pub fn error_count(&self) -> u32 {
        self.consecutive_errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip_type_detection() {
        // ChipType enum exists and can be matched
        let chip = ChipType::GenuineElm327;
        assert_eq!(chip, ChipType::GenuineElm327);

        let chip = ChipType::Stn1110;
        assert_eq!(chip, ChipType::Stn1110);

        let chip = ChipType::CloneV15;
        assert_eq!(chip, ChipType::CloneV15);
    }

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.baud_rate, 38400);
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.port.is_empty());
    }

    #[test]
    fn test_elm327_connection_new() {
        let conn = Elm327Connection::new();
        assert_eq!(conn.protocol, "");
        assert_eq!(conn.elm_version, "");
        assert_eq!(conn.chip_type, ChipType::Unknown);
        assert!(!conn.is_clone);
        assert!(conn.supported_pids.is_empty());
        assert_eq!(conn.consecutive_errors, 0);
    }
}
