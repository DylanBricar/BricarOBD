//! Transport layer abstraction — Serial USB, Bluetooth BLE, WiFi TCP
//! Allows the same ELM327 protocol to work over any physical link.

use std::io::{Read, Write};
use std::time::Duration;
use std::net::TcpStream;
use tracing::{debug, info, warn};

use super::dev_log;

/// Transport type identifier
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TransportType {
    Serial,
    Bluetooth,
    WiFi,
}

/// Unified transport trait — any backend that can send/receive bytes
pub trait OBDTransport: Send {
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), String>;
    fn read_bytes(&mut self, buf: &mut [u8], timeout_ms: u64) -> Result<usize, String>;
    fn flush(&mut self) -> Result<(), String>;
    fn transport_type(&self) -> TransportType;
    fn close(&mut self);
}

// ==================== SERIAL USB (Desktop only) ====================

#[cfg(feature = "desktop")]
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

#[cfg(feature = "desktop")]
impl SerialTransport {
    pub fn new(port_name: &str, baud_rate: u32, timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", &format!("Opening serial: {} @ {} baud", port_name, baud_rate));
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(timeout_ms))
            .open()
            .map_err(|e| format!("Serial open failed: {}", e))?;
        Ok(Self { port })
    }
}

#[cfg(feature = "desktop")]
impl OBDTransport for SerialTransport {
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), String> {
        self.port.write_all(data).map_err(|e| format!("Serial write: {}", e))
    }

    fn read_bytes(&mut self, buf: &mut [u8], _timeout_ms: u64) -> Result<usize, String> {
        match self.port.read(buf) {
            Ok(n) => Ok(n),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(format!("Serial read: {}", e)),
        }
    }

    fn flush(&mut self) -> Result<(), String> {
        self.port.flush().map_err(|e| format!("Serial flush: {}", e))
    }

    fn transport_type(&self) -> TransportType { TransportType::Serial }

    fn close(&mut self) {
        // SerialPort closes on drop
    }
}

// ==================== WIFI TCP (ELM327 WiFi adapters) ====================

pub struct WiFiTransport {
    stream: TcpStream,
}

impl WiFiTransport {
    /// Connect to WiFi ELM327 — typically 192.168.0.10:35000
    pub fn new(host: &str, port: u16, timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", &format!("Connecting WiFi: {}:{}", host, port));
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
            Duration::from_millis(timeout_ms),
        ).map_err(|e| format!("WiFi connect failed: {}", e))?;

        stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)))
            .map_err(|e| format!("Set timeout: {}", e))?;
        stream.set_nodelay(true).ok();

        dev_log::log_info("transport", "WiFi connected");
        Ok(Self { stream })
    }
}

impl OBDTransport for WiFiTransport {
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream.write_all(data).map_err(|e| format!("WiFi write: {}", e))
    }

    fn read_bytes(&mut self, buf: &mut [u8], _timeout_ms: u64) -> Result<usize, String> {
        match self.stream.read(buf) {
            Ok(n) => Ok(n),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(format!("WiFi read: {}", e)),
        }
    }

    fn flush(&mut self) -> Result<(), String> {
        self.stream.flush().map_err(|e| format!("WiFi flush: {}", e))
    }

    fn transport_type(&self) -> TransportType { TransportType::WiFi }

    fn close(&mut self) {
        self.stream.shutdown(std::net::Shutdown::Both).ok();
    }
}

// ==================== BLUETOOTH BLE ====================

pub struct BleTransport {
    // BLE uses btleplug for cross-platform BLE communication
    // ELM327 BLE adapters expose a UART service (Nordic UART or similar)
    // For now, this is a stub that will be implemented when btleplug compiles for the target
    _placeholder: (),
}

impl BleTransport {
    /// Scan and connect to a BLE ELM327 adapter
    pub fn new(_device_name: &str, _timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", "BLE transport: scanning for devices...");
        // TODO: Full btleplug implementation
        // 1. Create manager: btleplug::platform::Manager::new()
        // 2. Get adapter: manager.adapters().first()
        // 3. Start scan: adapter.start_scan(ScanFilter::default())
        // 4. Find device by name containing "ELM" or "OBD"
        // 5. Connect to device
        // 6. Discover services — look for Nordic UART Service (6E400001-...)
        // 7. Get TX characteristic (6E400002-...) and RX characteristic (6E400003-...)
        // 8. Subscribe to RX notifications
        Err("BLE transport not yet fully implemented — use WiFi or USB".to_string())
    }
}

impl OBDTransport for BleTransport {
    fn write_bytes(&mut self, _data: &[u8]) -> Result<(), String> {
        Err("BLE write not implemented".to_string())
    }
    fn read_bytes(&mut self, _buf: &mut [u8], _timeout_ms: u64) -> Result<usize, String> {
        Err("BLE read not implemented".to_string())
    }
    fn flush(&mut self) -> Result<(), String> { Ok(()) }
    fn transport_type(&self) -> TransportType { TransportType::Bluetooth }
    fn close(&mut self) {}
}

// ==================== FACTORY ====================

/// List available WiFi ELM327 endpoints (common defaults)
pub fn default_wifi_endpoints() -> Vec<(String, u16)> {
    vec![
        ("192.168.0.10".to_string(), 35000),  // Most common WiFi ELM327
        ("192.168.1.10".to_string(), 35000),  // Alternative
        ("192.168.4.1".to_string(), 35000),   // Some clones
        ("10.0.0.1".to_string(), 35000),      // Some newer adapters
    ]
}

/// Try to connect via WiFi by probing common addresses
pub fn auto_connect_wifi(timeout_ms: u64) -> Result<WiFiTransport, String> {
    for (host, port) in default_wifi_endpoints() {
        dev_log::log_debug("transport", &format!("Probing WiFi {}:{}", host, port));
        match WiFiTransport::new(&host, port, timeout_ms.min(2000)) {
            Ok(transport) => {
                dev_log::log_info("transport", &format!("WiFi connected at {}:{}", host, port));
                return Ok(transport);
            }
            Err(_) => continue,
        }
    }
    Err("No WiFi ELM327 found at common addresses".to_string())
}
