//! Transport layer abstraction – Serial USB, Bluetooth BLE, WiFi TCP
//! Allows the same ELM327 protocol to work over any physical link.

use std::io::{Read, Write};
use std::time::Duration;
use std::net::TcpStream;

use super::dev_log;

#[cfg(feature = "mobile")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "mobile")]
use std::collections::VecDeque;
#[cfg(feature = "mobile")]
use std::thread;
#[cfg(feature = "mobile")]
use btleplug::api::{Central, Peripheral, Characteristic, WriteType, CharPropFlags};
#[cfg(feature = "mobile")]
use btleplug::platform::Manager;
#[cfg(feature = "mobile")]
use futures::StreamExt;

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

#[cfg(feature = "mobile")]
const MAX_BLE_BUFFER: usize = 65536;

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
    /// Connect to WiFi ELM327 – typically 192.168.0.10:35000
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

#[cfg(feature = "mobile")]
pub struct BleDeviceInfo {
    pub name: String,
    pub address: String,
}

#[cfg(feature = "mobile")]
pub struct BleTransport {
    peripheral: btleplug::platform::Peripheral,
    tx_char: Characteristic,
    rx_buffer: Arc<Mutex<VecDeque<u8>>>,
    runtime: tokio::runtime::Handle,
    notification_task: Option<tokio::task::JoinHandle<()>>,
}

#[cfg(feature = "mobile")]
impl BleTransport {
    /// Scan and connect to a BLE ELM327 adapter
    pub fn new(device_name: &str, timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", &format!("BLE: scanning for device '{}'...", device_name));

        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| "No Tokio runtime available".to_string())?;

        let (peripheral, tx_char, rx_buffer, notification_task) = tokio::task::block_in_place(|| {
            runtime.block_on(async {
                Self::connect_async(device_name, timeout_ms).await
            })
        })?;

        dev_log::log_info("transport", "BLE connected and subscribed");
        Ok(Self {
            peripheral,
            tx_char,
            rx_buffer,
            runtime,
            notification_task: Some(notification_task),
        })
    }

    async fn connect_async(
        device_name: &str,
        timeout_ms: u64,
    ) -> Result<(btleplug::platform::Peripheral, Characteristic, Arc<Mutex<VecDeque<u8>>>, tokio::task::JoinHandle<()>), String> {
        let manager = Manager::new()
            .await
            .map_err(|e| format!("Failed to create BLE manager: {}", e))?;

        let adapters = manager.adapters()
            .await
            .map_err(|e| format!("Failed to get adapters: {}", e))?;
        let adapter = adapters.first()
            .ok_or("No BLE adapter found")?;

        dev_log::log_debug("transport", "Starting BLE scan...");
        adapter.start_scan(btleplug::api::ScanFilter::default())
            .await
            .map_err(|e| format!("Failed to start scan: {}", e))?;

        let scan_timeout = timeout_ms.min(5000);
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_timeout)).await;

        adapter.stop_scan()
            .await
            .map_err(|e| format!("Failed to stop scan: {}", e))?;

        dev_log::log_debug("transport", "BLE scan complete, searching for device...");
        let peripherals = adapter.peripherals()
            .await
            .map_err(|e| format!("Failed to get peripherals: {}", e))?;

        let device_name_lower = device_name.to_lowercase();
        let mut peripheral_found = None;
        for p in peripherals {
            if let Ok(Some(properties)) = p.properties().await {
                let name = properties.local_name.clone().unwrap_or_default().to_lowercase();
                if name.contains(&device_name_lower) || name.contains("elm") || name.contains("obd") {
                    peripheral_found = Some(p);
                    break;
                }
            }
        }
        let peripheral = peripheral_found
            .ok_or(format!("No BLE device matching '{}', 'ELM', or 'OBD' found", device_name))?;

        dev_log::log_info("transport", &format!("Found device: {:?}", peripheral.properties().await.ok().flatten()));

        peripheral.connect()
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        dev_log::log_debug("transport", "Connected, discovering services...");
        peripheral.discover_services()
            .await
            .map_err(|e| format!("Failed to discover services: {}", e))?;

        let services = peripheral.services();
        let uart_service = services.iter()
            .find(|svc| svc.uuid.to_string().to_lowercase().starts_with("6e400001"))
            .ok_or("Nordic UART service not found")?;

        let tx_char = uart_service.characteristics.iter()
            .find(|ch| ch.uuid.to_string().to_lowercase() == "6e400002-b5d3-4f47-8449-1934fe259dcc"
                || ch.uuid.to_string().to_lowercase().starts_with("6e400002"))
            .ok_or("TX characteristic not found")?
            .clone();

        let rx_char = uart_service.characteristics.iter()
            .find(|ch| ch.uuid.to_string().to_lowercase() == "6e400003-b5d3-4f47-8449-1934fe259dcc"
                || ch.uuid.to_string().to_lowercase().starts_with("6e400003"))
            .ok_or("RX characteristic not found")?
            .clone();

        peripheral.subscribe(&rx_char)
            .await
            .map_err(|e| format!("Failed to subscribe to RX: {}", e))?;

        let rx_buffer = Arc::new(Mutex::new(VecDeque::new()));
        let rx_buffer_clone = rx_buffer.clone();
        let peripheral_clone = peripheral.clone();

        let notification_task = tokio::spawn(async move {
            if let Ok(mut notifications) = peripheral_clone.notifications().await {
                while let Some(notification) = notifications.next().await {
                    let mut buf = rx_buffer_clone.lock().unwrap_or_else(|e| e.into_inner());
                    if buf.len() < MAX_BLE_BUFFER {
                        buf.extend_from_slice(&notification.value);
                    }
                }
            }
        });

        Ok((peripheral, tx_char, rx_buffer, notification_task))
    }
}

#[cfg(feature = "mobile")]
impl OBDTransport for BleTransport {
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), String> {
        tokio::task::block_in_place(|| {
            self.runtime.block_on(async {
                self.peripheral.write(&self.tx_char, data, WriteType::WithoutResponse)
                    .await
                    .map_err(|e| format!("BLE write failed: {}", e))
            })
        })
    }

    fn read_bytes(&mut self, buf: &mut [u8], timeout_ms: u64) -> Result<usize, String> {
        let start = std::time::Instant::now();
        let deadline = Duration::from_millis(timeout_ms);

        loop {
            {
                let mut buffer = self.rx_buffer.lock().unwrap_or_else(|e| e.into_inner());
                let count = buffer.drain(..buffer.len().min(buf.len())).zip(buf.iter_mut()).fold(0, |acc, (src, dst)| {
                    *dst = src;
                    acc + 1
                });
                if count > 0 {
                    return Ok(count);
                }
            }

            if start.elapsed() > deadline {
                return Ok(0);
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn flush(&mut self) -> Result<(), String> {
        let mut buffer = self.rx_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.clear();
        Ok(())
    }

    fn transport_type(&self) -> TransportType { TransportType::Bluetooth }

    fn close(&mut self) {
        if let Some(handle) = self.notification_task.take() {
            handle.abort();
        }
        tokio::task::block_in_place(|| {
            self.runtime.block_on(async {
                let _ = self.peripheral.disconnect().await;
            })
        });
    }
}

#[cfg(feature = "mobile")]
pub async fn scan_ble_devices(timeout_ms: u64) -> Result<Vec<BleDeviceInfo>, String> {
    let manager = Manager::new()
        .await
        .map_err(|e| format!("Failed to create BLE manager: {}", e))?;

    let adapters = manager.adapters()
        .await
        .map_err(|e| format!("Failed to get adapters: {}", e))?;
    let adapter = adapters.first()
        .ok_or("No BLE adapter found")?;

    adapter.start_scan(btleplug::api::ScanFilter::default())
        .await
        .map_err(|e| format!("Failed to start scan: {}", e))?;

    let scan_timeout = timeout_ms.min(5000);
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_timeout)).await;

    adapter.stop_scan()
        .await
        .map_err(|e| format!("Failed to stop scan: {}", e))?;

    let peripherals = adapter.peripherals()
        .await
        .map_err(|e| format!("Failed to get peripherals: {}", e))?;

    let mut devices = Vec::new();
    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            if let Some(name) = props.local_name {
                devices.push(BleDeviceInfo {
                    name,
                    address: p.address().to_string(),
                });
            }
        }
    }

    Ok(devices)
}

#[cfg(not(feature = "mobile"))]
pub struct BleTransport {
    _placeholder: (),
}

#[cfg(not(feature = "mobile"))]
impl BleTransport {
    pub fn new(_device_name: &str, _timeout_ms: u64) -> Result<Self, String> {
        dev_log::log_info("transport", "BLE transport: not available (desktop build)");
        Err("BLE transport not available on desktop – use WiFi or USB".to_string())
    }
}

#[cfg(not(feature = "mobile"))]
impl OBDTransport for BleTransport {
    fn write_bytes(&mut self, _data: &[u8]) -> Result<(), String> {
        Err("BLE not available".to_string())
    }
    fn read_bytes(&mut self, _buf: &mut [u8], _timeout_ms: u64) -> Result<usize, String> {
        Err("BLE not available".to_string())
    }
    fn flush(&mut self) -> Result<(), String> { Ok(()) }
    fn transport_type(&self) -> TransportType { TransportType::Bluetooth }
    fn close(&mut self) {}
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
