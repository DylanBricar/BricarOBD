#[cfg(feature = "desktop")]
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use crate::obd::dev_log;

use crate::models::PortInfo;

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

/// ELM327 OBD-II adapter connection with resilience layer
pub struct Elm327Connection {
    port: Option<Box<dyn SerialPort>>,
    pub config: ConnectionConfig,
    pub protocol: String,
    pub elm_version: String,
    pub is_clone: bool,           // Detected clone adapter
    pub supported_pids: Vec<u8>,  // Cached supported PIDs bitmap
}

impl Elm327Connection {
    pub fn new() -> Self {
        Self {
            port: None,
            config: ConnectionConfig::default(),
            protocol: String::new(),
            elm_version: String::new(),
            is_clone: false,
            supported_pids: Vec::new(),
        }
    }

    /// List available serial ports
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

    /// Connect to ELM327 adapter with full resilience
    pub fn connect(&mut self, port: &str, baud_rate: u32) -> Result<(), String> {
        info!("Connecting to {} at {} baud", port, baud_rate);

        let serial = serialport::new(port, baud_rate)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .open()
            .map_err(|e| format!("Failed to open port: {}", e))?;

        self.port = Some(serial);
        self.config.port = port.to_string();
        self.config.baud_rate = baud_rate;

        // Try multiple init strategies
        if let Err(e) = self.init_with_resilience() {
            self.port = None;
            return Err(e);
        }

        dev_log::log_info("obd", &format!("Connected: ELM={}, Protocol={}, Clone={}",
            self.elm_version, self.protocol, self.is_clone));
        info!("Connected: ELM={}, Protocol={}, Clone={}",
            self.elm_version, self.protocol, self.is_clone);
        Ok(())
    }

    /// Multi-strategy initialization – tries several approaches
    fn init_with_resilience(&mut self) -> Result<(), String> {
        // Strategy 1: Standard init (works for genuine ELM327)
        if self.try_standard_init().is_ok() {
            info!("Connected via standard init");
            return Ok(());
        }
        warn!("Standard init failed, trying clone-compatible init...");

        // Strategy 2: Clone-compatible init (skip ATZ, use ATI)
        if self.try_clone_init().is_ok() {
            self.is_clone = true;
            info!("Connected via clone init");
            return Ok(());
        }
        warn!("Clone init failed, trying aggressive init...");

        // Strategy 3: Aggressive init (flush buffer, force reset, longer timeouts)
        if self.try_aggressive_init().is_ok() {
            info!("Connected via aggressive init");
            return Ok(());
        }
        warn!("Aggressive init failed, trying minimal init...");

        // Strategy 4: Minimal init (bare minimum commands)
        if self.try_minimal_init().is_ok() {
            info!("Connected via minimal init");
            return Ok(());
        }

        Err("All connection strategies failed".to_string())
    }

    // ==================== INIT STRATEGIES ====================

    /// Strategy 1: Standard ELM327 initialization
    fn try_standard_init(&mut self) -> Result<(), String> {
        // Reset
        let reset_response = self.send_command("ATZ")?;
        if let Some(ver) = reset_response.lines().find(|l| l.contains("ELM")) {
            self.elm_version = ver.trim().to_string();
        }

        self.configure_adapter()?;
        self.detect_protocol()
    }

    /// Strategy 2: Clone adapter compatible (no ATZ reset)
    fn try_clone_init(&mut self) -> Result<(), String> {
        // Flush any garbage in the buffer
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(200));

        // Skip ATZ – many clones hang on it. Use ATI instead.
        let ati_response = self.send_command_timeout("ATI", 3000)?;
        if ati_response.contains("ELM") || ati_response.contains("OBD") {
            self.elm_version = ati_response.lines()
                .find(|l| l.contains("ELM") || l.contains("OBD"))
                .unwrap_or("Clone v1.5")
                .trim().to_string();
        } else {
            self.elm_version = "Unknown Clone".to_string();
        }

        // Send ATE0 twice — some clones need a warm-up command
        let _ = self.send_command("ATE0");
        let _ = self.send_command("ATE0");

        self.configure_adapter()?;
        self.detect_protocol()
    }

    /// Strategy 3: Aggressive init (flush, delay, force)
    fn try_aggressive_init(&mut self) -> Result<(), String> {
        // Hard flush
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(500));

        // Send multiple CR to wake up
        if let Some(ref mut port) = self.port {
            let _ = port.write_all(b"\r\r\r");
            let _ = port.flush();
        }
        std::thread::sleep(Duration::from_millis(300));
        self.flush_buffer();

        // Try warm reset (ATD = set all defaults) instead of ATZ
        let _ = self.send_command_timeout("ATD", 3000);
        let _ = self.send_command("ATE0");

        // Detect version
        if let Ok(response) = self.send_command("ATI") {
            self.elm_version = response.lines()
                .find(|l| !l.is_empty())
                .unwrap_or("Unknown")
                .trim().to_string();
        }

        // Set longer adaptive timing
        let _ = self.send_command("ATAT2"); // Aggressive adaptive timing
        let _ = self.send_command("ATST FF"); // Max timeout (255 × 4ms = 1.02s)

        self.configure_adapter()?;
        self.detect_protocol()
    }

    /// Strategy 4: Minimal init (absolute bare minimum)
    fn try_minimal_init(&mut self) -> Result<(), String> {
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(300));

        // Just echo off and go
        let _ = self.send_command_timeout("ATE0", 2000);
        let _ = self.send_command("ATS1");
        self.elm_version = "Minimal".to_string();

        // Try each protocol directly without ATSP0 auto-detect
        self.detect_protocol()
    }

    // ==================== SHARED HELPERS ====================

    /// Configure adapter with optimal settings
    fn configure_adapter(&mut self) -> Result<(), String> {
        self.send_command("ATE0")?;   // Echo off
        self.send_command("ATL0")?;   // Linefeeds off
        self.send_command("ATS1")?;   // Spaces on (easier parsing)
        self.send_command("ATH0")?;   // Headers off
        let _ = self.send_command("ATAT1"); // Adaptive timing (normal)
        let _ = self.send_command("ATST 64"); // Timeout 100 × 4ms = 400ms
        Ok(())
    }

    /// Detect OBD protocol — tries auto-detect then manual cycling
    fn detect_protocol(&mut self) -> Result<(), String> {
        // First try: auto-detect with ATSP0
        let _ = self.send_command("ATSP0");
        std::thread::sleep(Duration::from_millis(200));

        let response = match self.send_command_timeout("0100", 8000) {
            Ok(r) => r,
            Err(_) => String::new(),
        };

        if response.contains("41 00") {
            let proto = self.send_command("ATDPN").unwrap_or_default();
            self.protocol = Self::decode_protocol(&proto);
            self.parse_supported_pids(&response);
            return Ok(());
        }

        // Second try: manual protocol cycling with extended timeout
        self.cycle_protocols()
    }

    /// Cycle through protocols — handles errors gracefully
    fn cycle_protocols(&mut self) -> Result<(), String> {
        // Order: most common first (CAN 500k, CAN 250k, KWP fast, KWP slow, ISO, J1850)
        let protocols = [
            ("6", 6000),   // ISO 15765-4 CAN 11-bit 500k (most post-2008 vehicles)
            ("8", 6000),   // ISO 15765-4 CAN 11-bit 250k
            ("7", 6000),   // ISO 15765-4 CAN 29-bit 500k
            ("9", 6000),   // ISO 15765-4 CAN 29-bit 250k
            ("5", 8000),   // ISO 14230-4 KWP fast init
            ("4", 12000),  // ISO 14230-4 KWP 5-baud init (needs 10s+)
            ("3", 10000),  // ISO 9141-2 (slow init)
            ("1", 6000),   // SAE J1850 PWM
            ("2", 6000),   // SAE J1850 VPW
        ];

        for (proto_num, timeout_ms) in protocols {
            debug!("Trying protocol ATSP{} (timeout {}ms)", proto_num, timeout_ms);

            let _ = self.send_command(&format!("ATSP{}", proto_num));
            std::thread::sleep(Duration::from_millis(300));

            // For KWP/ISO protocols, set longer timeout on the adapter too
            if timeout_ms > 6000 {
                let _ = self.send_command("ATST FF"); // Max adapter timeout
            }

            let response = match self.send_command_timeout("0100", timeout_ms as u64) {
                Ok(r) => r,
                Err(e) => {
                    debug!("Protocol {} error: {}", proto_num, e);
                    continue;
                }
            };

            // Check for valid response
            if response.contains("41 00") {
                let proto = self.send_command("ATDPN").unwrap_or_default();
                self.protocol = Self::decode_protocol(&proto);
                self.parse_supported_pids(&response);
                // Restore normal timeout
                let _ = self.send_command("ATST 64");
                info!("Found working protocol: {} (ATSP{})", self.protocol, proto_num);
                return Ok(());
            }

            // Check for partial success indicators
            if response.contains("41") || response.contains("7F") {
                // Got a response (maybe error), but communication exists
                warn!("Protocol {} got partial response: {}", proto_num, response);
                let proto = self.send_command("ATDPN").unwrap_or_default();
                self.protocol = Self::decode_protocol(&proto);
                let _ = self.send_command("ATST 64");
                return Ok(());
            }
        }

        Err("No compatible OBD protocol found".to_string())
    }

    /// Parse supported PIDs from Mode 01 PID 00 response
    fn parse_supported_pids(&mut self, response: &str) {
        // Response "41 00 BE 3E B0 13" — bytes encode a bitmask
        let bytes: Vec<u8> = response
            .split_whitespace()
            .skip(2) // Skip "41 00"
            .take(4)
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();

        self.supported_pids.clear();
        for (byte_idx, &byte) in bytes.iter().enumerate() {
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    self.supported_pids.push((byte_idx * 8 + bit + 1) as u8);
                }
            }
        }
        debug!("Supported PIDs: {:?}", self.supported_pids);
    }

    // ==================== SEND / RECEIVE ====================

    /// Send command with default timeout
    pub fn send_command(&mut self, cmd: &str) -> Result<String, String> {
        self.send_command_timeout(cmd, self.config.timeout_ms)
    }

    /// Send command with custom timeout
    pub fn send_command_timeout(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, String> {
        let port = self.port.as_mut().ok_or("Not connected")?;

        // Log TX
        dev_log::log_tx(cmd);

        // Small delay between commands (ELM327 needs recovery time)
        std::thread::sleep(Duration::from_millis(30));

        // Send command
        let cmd_bytes = format!("{}\r", cmd);
        port.write_all(cmd_bytes.as_bytes())
            .map_err(|e| { dev_log::log_error("obd", &format!("Write error: {}", e)); format!("Write error: {}", e) })?;
        port.flush().map_err(|e| format!("Flush error: {}", e))?;

        // Read response (wait for prompt '>')
        let mut response = String::new();
        let mut buf = [0u8; 64]; // Read in larger chunks for performance
        let timeout = std::time::Instant::now();

        while timeout.elapsed() < Duration::from_millis(timeout_ms) {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    for &byte in &buf[..n] {
                        let ch = byte as char;
                        if ch == '>' {
                            let clean = Self::clean_response(&response);
                            dev_log::log_rx(cmd, &clean);
                            return Ok(clean);
                        }
                        response.push(ch);
                    }
                }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(format!("Read error: {}", e)),
            }
        }

        // Timeout — return what we have (may be partial)
        if !response.is_empty() {
            let clean = Self::clean_response(&response);
            dev_log::log_rx(cmd, &format!("(timeout) {}", clean));
            Ok(clean)
        } else {
            dev_log::log_error("obd", &format!("Timeout: {}", cmd));
            Err(format!("Timeout on command: {}", cmd))
        }
    }

    /// Clean ELM327 response — filter noise, errors, control chars
    fn clean_response(raw: &str) -> String {
        raw.replace('\r', "\n")
            .lines()
            .map(|l| l.trim())
            .filter(|l| {
                !l.is_empty()
                && !l.starts_with("SEARCHING")
                && !l.starts_with("BUS INIT")
                && !l.starts_with("UNABLE TO CONNECT")
                && !l.starts_with("CAN ERROR")
                && !l.starts_with("FB ERROR")
                && !l.starts_with("DATA ERROR")
                && !l.starts_with("BUFFER FULL")
                && !l.starts_with("STOPPED")
                && !l.starts_with("BUS BUSY")
                && *l != "?"
                && *l != "OK"
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Flush input buffer (discard any pending data)
    fn flush_buffer(&mut self) {
        if let Some(ref mut port) = self.port {
            let mut buf = [0u8; 512];
            // Read and discard all pending data
            loop {
                match port.read(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        }
    }

    /// Send OBD-II PID request with retry logic
    pub fn query_pid(&mut self, mode: u8, pid: u8) -> Result<Vec<u8>, String> {
        let cmd = format!("{:02X}{:02X}", mode, pid);
        let expected_prefix = format!("{:02X} {:02X}", mode + 0x40, pid);

        // Try up to 2 times
        for attempt in 0..2 {
            let response = match self.send_command(&cmd) {
                Ok(r) => r,
                Err(e) => {
                    if attempt == 0 {
                        debug!("PID {:02X} attempt 1 failed: {}, retrying...", pid, e);
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    return Err(e);
                }
            };

            // Check for "NO DATA" — PID not supported
            if response.contains("NO DATA") {
                return Err(format!("PID {:02X} not supported", pid));
            }

            // Parse hex response: "41 0C 1A F8"
            if let Some(data_str) = response.lines().find(|l| l.starts_with(&expected_prefix)) {
                let bytes: Vec<u8> = data_str
                    .split_whitespace()
                    .skip(2)
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect();
                return Ok(bytes);
            }

            // Try without spaces (some adapters)
            let no_space_prefix = format!("{:02X}{:02X}", mode + 0x40, pid);
            if response.contains(&no_space_prefix) {
                // Parse packed hex
                let hex_str = response.replace(" ", "");
                if let Some(start) = hex_str.find(&no_space_prefix) {
                    let data_start = start + no_space_prefix.len();
                    let data_hex = &hex_str[data_start..];
                    let bytes: Vec<u8> = (0..data_hex.len())
                        .step_by(2)
                        .filter_map(|i| {
                            if i + 2 <= data_hex.len() {
                                u8::from_str_radix(&data_hex[i..i+2], 16).ok()
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !bytes.is_empty() {
                        return Ok(bytes);
                    }
                }
            }

            if attempt == 0 {
                debug!("PID {:02X} parse failed, retrying...", pid);
                std::thread::sleep(Duration::from_millis(50));
            }
        }

        Err(format!("Invalid response for PID {:02X}", pid))
    }

    /// Disconnect
    pub fn disconnect(&mut self) {
        if let Some(ref mut port) = self.port {
            let _ = port.write_all(b"\r");
            std::thread::sleep(Duration::from_millis(100));
            let _ = port.write_all(b"ATZ\r");
        }
        self.port = None;
        self.protocol.clear();
        self.elm_version.clear();
        self.supported_pids.clear();
        info!("Disconnected");
    }

    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }

    /// Decode protocol number to name
    fn decode_protocol(proto_str: &str) -> String {
        let num = proto_str.trim().chars().last().unwrap_or('?');
        match num {
            '1' => "SAE J1850 PWM",
            '2' => "SAE J1850 VPW",
            '3' => "ISO 9141-2",
            '4' => "ISO 14230-4 KWP (5 baud init)",
            '5' => "ISO 14230-4 KWP (fast init)",
            '6' => "ISO 15765-4 CAN 11-bit 500k",
            '7' => "ISO 15765-4 CAN 29-bit 500k",
            '8' => "ISO 15765-4 CAN 11-bit 250k",
            '9' => "ISO 15765-4 CAN 29-bit 250k",
            'A' => "SAE J1939 CAN 29-bit 250k",
            _ => "Unknown",
        }
        .to_string()
    }
}
