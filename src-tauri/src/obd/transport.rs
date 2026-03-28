//! Transport layer abstraction – Serial USB, Bluetooth BLE, WiFi TCP
//! Allows the same ELM327 protocol to work over any physical link.

use std::io::{Read, Write};
use std::time::Duration;
use std::net::TcpStream;

use super::dev_log;
pub use super::transport_ble::{BleTransport, BleDeviceInfo};

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
    current_timeout_ms: u64,
}

#[cfg(feature = "desktop")]
impl SerialTransport {
    pub fn new(port_name: &str, baud_rate: u32, timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", &format!("Opening serial: {} @ {} baud", port_name, baud_rate));
        let port = serialport::new(port_name, baud_rate)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_millis(timeout_ms))
            .open()
            .map_err(|e| {
                let msg = e.to_string();
                let detail = match e.kind() {
                    serialport::ErrorKind::Io(io_kind) => match io_kind {
                        std::io::ErrorKind::PermissionDenied => {
                            #[cfg(target_os = "linux")]
                            { format!("Permission denied on {}. Run: sudo usermod -aG dialout $USER — then log out and back in.", port_name) }
                            #[cfg(target_os = "macos")]
                            { format!("Permission denied on {}. The port may be in use by another app, or the adapter driver is not installed.", port_name) }
                            #[cfg(target_os = "windows")]
                            { format!("Access denied on {}. The port may be in use by another application (close other OBD software first).", port_name) }
                            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                            { format!("Permission denied on {}: {}", port_name, msg) }
                        }
                        std::io::ErrorKind::NotFound => {
                            #[cfg(target_os = "linux")]
                            { format!("Port {} not found. The adapter may be unplugged or the driver is missing (check: ls /dev/ttyUSB* /dev/ttyACM*).", port_name) }
                            #[cfg(target_os = "macos")]
                            { format!("Port {} not found. The adapter may be unplugged. If using a CH340/CH341 adapter, install the macOS driver from the manufacturer.", port_name) }
                            #[cfg(target_os = "windows")]
                            { format!("Port {} not found. The adapter may be unplugged or the driver is missing (install CH340 or FTDI driver from Device Manager).", port_name) }
                            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                            { format!("Port {} not found: {}", port_name, msg) }
                        }
                        _ => {
                            if msg.contains("Resource busy") {
                                format!("Port {} is busy — close any other application using this port.", port_name)
                            } else {
                                format!("Serial open failed on {}: {}", port_name, msg)
                            }
                        }
                    }
                    _ => format!("Serial open failed on {}: {}", port_name, msg),
                };
                dev_log::log_error("transport", &detail);
                detail
            })?;
        Ok(Self { port, current_timeout_ms: timeout_ms })
    }
}

#[cfg(feature = "desktop")]
impl OBDTransport for SerialTransport {
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), String> {
        self.port.write_all(data).map_err(|e| format!("Serial write: {}", e))
    }

    fn read_bytes(&mut self, buf: &mut [u8], timeout_ms: u64) -> Result<usize, String> {
        if timeout_ms != self.current_timeout_ms {
            self.port.set_timeout(Duration::from_millis(timeout_ms))
                .map_err(|e| format!("Failed to set timeout: {}", e))?;
            self.current_timeout_ms = timeout_ms;
        }
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
    /// Connect to WiFi ELM327 – typically 192.168.0.10:35000
    pub fn new(host: &str, port: u16, timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", &format!("Connecting WiFi: {}:{}", host, port));
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
            Duration::from_millis(timeout_ms),
        ).map_err(|e| {
            let msg = format!("WiFi connect to {}:{} failed: {}. Check that the ELM327 WiFi adapter is powered on and your device is connected to its WiFi network.", host, port, e);
            dev_log::log_error("transport", &msg);
            msg
        })?;

        stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)))
            .map_err(|e| format!("Set read timeout: {}", e))?;
        stream.set_write_timeout(Some(Duration::from_millis(timeout_ms)))
            .map_err(|e| format!("Set write timeout: {}", e))?;
        stream.set_nodelay(true).ok();

        dev_log::log_info("transport", "WiFi connected");
        Ok(Self { stream })
    }
}

impl OBDTransport for WiFiTransport {
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream.write_all(data).map_err(|e| format!("WiFi write: {}", e))
    }

    fn read_bytes(&mut self, buf: &mut [u8], timeout_ms: u64) -> Result<usize, String> {
        let orig = self.stream.read_timeout().ok().flatten();
        let _ = self.stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)));

        let result = match self.stream.read(buf) {
            Ok(n) => Ok(n),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(format!("WiFi read: {}", e)),
        };

        if let Some(d) = orig {
            let _ = self.stream.set_read_timeout(Some(d));
        } else {
            let _ = self.stream.set_read_timeout(None);
        }

        result
    }

    fn flush(&mut self) -> Result<(), String> {
        self.stream.flush().map_err(|e| format!("WiFi flush: {}", e))
    }

    fn transport_type(&self) -> TransportType { TransportType::WiFi }

    fn close(&mut self) {
        self.stream.shutdown(std::net::Shutdown::Both).ok();
    }
}

// ==================== FACTORY ====================

/// List available WiFi ELM327 endpoints (common defaults)
pub fn default_wifi_endpoints() -> Vec<(String, u16)> {
    vec![
        ("192.168.0.10".to_string(), 35000),  // Most common WiFi ELM327 (Viecar, generic)
        ("192.168.0.10".to_string(), 23),     // Some adapters use telnet port
        ("192.168.4.1".to_string(), 35000),   // Konnwei / hotspot-mode adapters
        ("192.168.1.10".to_string(), 35000),  // Alternative subnet
        ("192.168.1.1".to_string(), 3333),    // Older clones (non-standard port)
        ("10.0.0.1".to_string(), 35000),      // Some newer adapters
        ("192.168.2.10".to_string(), 35000),  // Rare subnet variant
    ]
}

/// Try to connect via WiFi by probing common addresses in parallel
pub fn auto_connect_wifi(timeout_ms: u64) -> Result<WiFiTransport, String> {
    use std::sync::{Mutex, Arc};
    let endpoints = default_wifi_endpoints();
    let found: Arc<Mutex<Option<WiFiTransport>>> = Arc::new(Mutex::new(None));

    std::thread::scope(|s| {
        let mut handles = vec![];
        for (host, port) in endpoints {
            let found = Arc::clone(&found);
            let handle = s.spawn(move || {
                dev_log::log_debug("transport", &format!("Probing WiFi {}:{}", host, port));
                if let Ok(transport) = WiFiTransport::new(&host, port, timeout_ms.min(2000)) {
                    let mut result = found.lock().unwrap_or_else(|e| e.into_inner());
                    if result.is_none() {
                        dev_log::log_info("transport", &format!("WiFi connected at {}:{}", host, port));
                        *result = Some(transport);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    });

    let mut guard = found.lock().unwrap_or_else(|e| e.into_inner());
    guard.take().ok_or_else(|| "No WiFi ELM327 found at common addresses".to_string())
}
