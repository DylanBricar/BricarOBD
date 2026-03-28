//! Bluetooth BLE transport layer — Mobile only
//! Provides BLE-specific implementation for ELM327 adapters

use super::dev_log;
use super::transport::{OBDTransport, TransportType};

// ==================== BLUETOOTH BLE ====================

#[cfg(feature = "mobile")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "mobile")]
use std::collections::VecDeque;
#[cfg(feature = "mobile")]
use std::io::Read;
#[cfg(feature = "mobile")]
use std::time::Duration;
#[cfg(feature = "mobile")]
use std::thread;
#[cfg(feature = "mobile")]
use btleplug::api::{Central, Peripheral, Characteristic, WriteType, CharPropFlags};
#[cfg(feature = "mobile")]
use btleplug::platform::Manager;
#[cfg(feature = "mobile")]
use futures::StreamExt;

#[cfg(feature = "mobile")]
const MAX_BLE_BUFFER: usize = 65536;

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
                    // If buffer would overflow, drain oldest data to make room (FIFO)
                    let incoming = notification.value.len();
                    if buf.len() + incoming > MAX_BLE_BUFFER {
                        dev_log::log_warn("transport", &format!("BLE buffer overflow: draining {}B to make room for {}B", buf.len() + incoming - MAX_BLE_BUFFER, incoming));
                        let drain_count = (buf.len() + incoming).saturating_sub(MAX_BLE_BUFFER);
                        let drain_count = drain_count.min(buf.len());
                        buf.drain(..drain_count);
                    }
                    buf.extend_from_slice(&notification.value);
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

            // Poll interval: 20ms balances latency vs CPU usage.
            // TODO: Replace with Condvar for zero-CPU-cost blocking reads.
            thread::sleep(Duration::from_millis(20));
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
pub struct BleDeviceInfo {
    pub name: String,
    pub address: String,
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
