use std::time::Duration;
use tracing::{debug, error, info, warn};
use crate::obd::dev_log;
use crate::obd::transport::{OBDTransport, TransportType};

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
    transport: Option<Box<dyn OBDTransport>>,
    pub config: ConnectionConfig,
    pub protocol: String,
    pub protocol_num: String,        // Raw protocol number (e.g. "6" for CAN 500k)
    pub elm_version: String,
    pub chip_type: ChipType,
    pub is_clone: bool,
    pub supported_pids: Vec<u8>,     // PIDs 0x01-0x60 supported bitmap
    pub supported_pids_ext: Vec<u8>, // PIDs 0x61-0xC0 if available
    pub voltage: Option<f64>,        // Battery voltage from ATRV
    headers_on: bool,                // Track ATH state
    last_command_time: std::time::Instant,
    consecutive_errors: u32,         // Track errors for adaptive recovery
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

    /// Connect to ELM327 via serial port with full resilience
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

    // ==================== INIT STRATEGIES ====================

    /// Multi-strategy initialization — tries 4 approaches, then bails
    fn init_with_resilience(&mut self) -> Result<(), String> {
        let strategies: Vec<(&str, fn(&mut Self) -> Result<(), String>)> = vec![
            ("standard", Self::try_standard_init),
            ("clone-compatible", Self::try_clone_init),
            ("aggressive", Self::try_aggressive_init),
            ("minimal", Self::try_minimal_init),
        ];

        for (name, strategy) in &strategies {
            dev_log::log_info("obd", &format!("Trying {} init strategy...", name));
            match strategy(self) {
                Ok(()) => {
                    dev_log::log_info("obd", &format!("Connected via {} init", name));
                    info!("Connected via {} init", name);
                    return Ok(());
                }
                Err(e) => {
                    dev_log::log_warn("obd", &format!("{} init failed: {}", name, e));
                    warn!("{} init failed: {}", name, e);
                    // Flush buffer between strategies
                    self.flush_buffer();
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }

        Err("All 4 connection strategies failed".to_string())
    }

    /// Strategy 1: Standard ELM327 initialization (genuine chips)
    fn try_standard_init(&mut self) -> Result<(), String> {
        // Hard reset
        let reset_response = self.send_command_timeout("ATZ", 4000)?;
        self.detect_chip_type(&reset_response);

        self.configure_adapter()?;
        self.configure_can_flow_control()?;
        self.detect_protocol()
    }

    /// Strategy 2: Clone adapter compatible (no ATZ reset — many clones hang on it)
    fn try_clone_init(&mut self) -> Result<(), String> {
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(200));

        // Use ATI instead of ATZ — clones handle it better
        let ati_response = self.send_command_timeout("ATI", 3000)?;
        self.detect_chip_type(&ati_response);
        self.is_clone = true;

        // Send ATE0 twice — some clones need a warm-up command before they respond properly
        let _ = self.send_command_timeout("ATE0", 1500);
        std::thread::sleep(Duration::from_millis(100));
        let _ = self.send_command_timeout("ATE0", 1500);

        self.configure_adapter()?;
        self.configure_can_flow_control()?;
        self.detect_protocol()
    }

    /// Strategy 3: Aggressive init (flush, delay, force wake-up, longer timeouts)
    fn try_aggressive_init(&mut self) -> Result<(), String> {
        // Hard flush
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(500));

        // Send multiple CR to wake up adapter from sleep/garbage state
        if let Some(ref mut transport) = self.transport {
            let _ = transport.write_bytes(b"\r\r\r");
            let _ = transport.flush();
        }
        std::thread::sleep(Duration::from_millis(300));
        self.flush_buffer();

        // Try ATD (set all defaults) instead of ATZ — lighter reset
        let _ = self.send_command_timeout("ATD", 3000);
        let _ = self.send_command_timeout("ATE0", 2000);

        // Detect version
        if let Ok(response) = self.send_command_timeout("ATI", 3000) {
            self.detect_chip_type(&response);
        }

        // Use aggressive adaptive timing + max timeout
        let _ = self.send_command("ATAT2");   // 2× adaptive timing
        let _ = self.send_command("ATST FF");  // Max timeout (255 × 4ms = 1.02s)

        self.configure_adapter()?;
        // Skip CAN flow control — might cause issues on problem adapters
        self.detect_protocol()
    }

    /// Strategy 4: Minimal init — absolute bare minimum, skip all optional AT commands
    fn try_minimal_init(&mut self) -> Result<(), String> {
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(300));

        // Only essential commands
        let _ = self.send_command_timeout("ATE0", 2000);
        let _ = self.send_command("ATS1");   // Spaces on (easier parsing)
        self.elm_version = "Minimal".to_string();
        self.chip_type = ChipType::Unknown;

        // Try protocols directly — no ATSP0 auto-detect (saves time on problem adapters)
        self.detect_protocol()
    }

    // ==================== CHIP DETECTION ====================

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

    // ==================== ADAPTER CONFIGURATION ====================

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

    // ==================== PROTOCOL DETECTION ====================

    /// Detect OBD protocol — tries auto-detect first, then manual cycling
    fn detect_protocol(&mut self) -> Result<(), String> {
        // First: wake up the ECU with TesterPresent (works even if protocol is wrong)
        // This ensures the ECU is not in sleep mode
        dev_log::log_debug("obd", "Sending wake-up TesterPresent before protocol detection");

        // Try auto-detect with ATSP0
        let _ = self.send_command("ATSP0");
        std::thread::sleep(Duration::from_millis(300));

        // Send 0100 with generous timeout — auto-detect can take 5-10s
        let response = match self.send_command_timeout("0100", 10000) {
            Ok(r) => r,
            Err(_) => String::new(),
        };

        if self.check_valid_pid_response(&response, "41 00") {
            let proto = self.send_command("ATDPN").unwrap_or_default();
            self.protocol_num = proto.trim().chars().last().map(|c| c.to_string()).unwrap_or_default();
            self.protocol = Self::decode_protocol(&proto);
            self.parse_supported_pids_from_response(&response);
            dev_log::log_info("obd", &format!("Auto-detect found protocol: {} ({})", self.protocol, self.protocol_num));
            return Ok(());
        }

        // Auto-detect failed — try manual protocol cycling
        dev_log::log_warn("obd", "Auto-detect failed, cycling protocols manually...");
        self.cycle_protocols()
    }

    /// Cycle through protocols — handles errors gracefully, tries multiple approaches per protocol
    fn cycle_protocols(&mut self) -> Result<(), String> {
        // Order: most common first (CAN 500k, CAN 250k, KWP fast, KWP slow, ISO, J1850)
        let protocols = [
            ("6", 6000,  "ISO 15765-4 CAN 11-bit 500k"),  // Most post-2008 vehicles
            ("8", 6000,  "ISO 15765-4 CAN 11-bit 250k"),
            ("7", 6000,  "ISO 15765-4 CAN 29-bit 500k"),
            ("9", 6000,  "ISO 15765-4 CAN 29-bit 250k"),
            ("5", 8000,  "ISO 14230-4 KWP fast init"),
            ("4", 14000, "ISO 14230-4 KWP 5-baud init"),  // Needs 10s+ for slow init
            ("3", 12000, "ISO 9141-2"),                    // Slow init
            ("1", 6000,  "SAE J1850 PWM"),
            ("2", 6000,  "SAE J1850 VPW"),
            ("A", 6000,  "SAE J1939 CAN 29-bit 250k"),    // Trucks/heavy vehicles
        ];

        for (proto_num, timeout_ms, proto_name) in protocols {
            dev_log::log_debug("obd", &format!("Trying protocol ATSP{} ({}) — timeout {}ms", proto_num, proto_name, timeout_ms));

            let _ = self.send_command(&format!("ATSP{}", proto_num));
            std::thread::sleep(Duration::from_millis(300));

            // For slow protocols (KWP/ISO), set max adapter timeout
            if timeout_ms > 6000 {
                let _ = self.send_command("ATST FF");
            }

            // Try 0100 first (standard PID request)
            let response = match self.send_command_timeout("0100", timeout_ms as u64) {
                Ok(r) => r,
                Err(e) => {
                    debug!("Protocol {} error: {}", proto_num, e);
                    // Restore normal timeout before trying next
                    if timeout_ms > 6000 {
                        let _ = self.send_command("ATST 64");
                    }
                    continue;
                }
            };

            // Check for valid response
            if self.check_valid_pid_response(&response, "41 00") {
                self.protocol_num = proto_num.to_string();
                self.protocol = proto_name.to_string();
                self.parse_supported_pids_from_response(&response);
                // Restore normal timeout
                let _ = self.send_command("ATST 64");
                dev_log::log_info("obd", &format!("Found working protocol: {} (ATSP{})", proto_name, proto_num));
                info!("Found working protocol: {} (ATSP{})", proto_name, proto_num);
                return Ok(());
            }

            // Check for partial success — ECU responded but with an error (communication exists)
            if response.contains("41") || response.contains("7F") || response.contains("7E") {
                warn!("Protocol {} got partial response: {}", proto_num, response);
                dev_log::log_warn("obd", &format!("Protocol {} partial response — using it: {}", proto_num, response));
                self.protocol_num = proto_num.to_string();
                self.protocol = proto_name.to_string();
                let _ = self.send_command("ATST 64");
                return Ok(());
            }

            // Restore normal timeout
            if timeout_ms > 6000 {
                let _ = self.send_command("ATST 64");
            }
        }

        // Last resort: try with headers ON — some vehicles only respond when headers are enabled
        dev_log::log_warn("obd", "All protocols failed with headers off, trying with headers on...");
        self.set_headers(true);

        for (proto_num, timeout_ms, proto_name) in &protocols[..4] {
            let _ = self.send_command(&format!("ATSP{}", proto_num));
            std::thread::sleep(Duration::from_millis(200));

            if let Ok(response) = self.send_command_timeout("0100", *timeout_ms as u64) {
                if response.contains("41 00") || response.contains("4100") {
                    self.protocol_num = proto_num.to_string();
                    self.protocol = proto_name.to_string();
                    self.set_headers(false);
                    dev_log::log_info("obd", &format!("Found protocol with headers: {} (ATSP{})", proto_name, proto_num));
                    return Ok(());
                }
            }
        }

        self.set_headers(false);
        Err("No compatible OBD protocol found after trying all 10 protocols".to_string())
    }

    /// Check if response contains a valid PID response (handles spaces/no-spaces, multi-line)
    fn check_valid_pid_response(&self, response: &str, expected_prefix: &str) -> bool {
        if response.contains(expected_prefix) {
            return true;
        }
        // Try without spaces (some adapters strip spaces)
        let no_space_prefix = expected_prefix.replace(" ", "");
        if response.replace(" ", "").contains(&no_space_prefix) {
            return true;
        }
        false
    }

    // ==================== SUPPORTED PID DISCOVERY ====================

    /// Discover all supported PID ranges: 0100, 0120, 0140, 0160
    fn discover_supported_pids(&mut self) {
        dev_log::log_debug("obd", "Discovering supported PID ranges...");

        // Range 0x01-0x20 (already read during protocol detect, but re-parse if needed)
        if self.supported_pids.is_empty() {
            if let Ok(response) = self.send_command("0100") {
                self.parse_supported_pids_from_response(&response);
            }
        }

        // Check if PID 0x20 is supported (means 0x21-0x40 range available)
        if self.supported_pids.contains(&0x20) {
            if let Ok(response) = self.send_command("0120") {
                let pids = Self::parse_pid_bitmap(&response, "41 20", 0x20);
                self.supported_pids.extend(pids);
            }
        }

        // Check if PID 0x40 is supported (means 0x41-0x60 range available)
        if self.supported_pids.contains(&0x40) {
            if let Ok(response) = self.send_command("0140") {
                let pids = Self::parse_pid_bitmap(&response, "41 40", 0x40);
                self.supported_pids.extend(pids);
            }
        }

        // Check if PID 0x60 is supported (means 0x61-0x80 range available)
        if self.supported_pids.contains(&0x60) {
            if let Ok(response) = self.send_command("0160") {
                let pids = Self::parse_pid_bitmap(&response, "41 60", 0x60);
                self.supported_pids_ext.extend(pids);
            }
        }

        self.supported_pids.sort();
        self.supported_pids.dedup();
        self.supported_pids_ext.sort();
        self.supported_pids_ext.dedup();

        dev_log::log_info("obd", &format!(
            "Supported PIDs: {} standard (0x01-0x60), {} extended (0x61+)",
            self.supported_pids.len(), self.supported_pids_ext.len()
        ));
    }

    /// Parse supported PIDs bitmask from any "41 XX" response
    fn parse_supported_pids_from_response(&mut self, response: &str) {
        let pids = Self::parse_pid_bitmap(response, "41 00", 0x00);
        if !pids.is_empty() {
            self.supported_pids = pids;
        }
    }

    /// Parse a PID bitmap from response — handles spaced and unspaced hex
    fn parse_pid_bitmap(response: &str, prefix: &str, base: u8) -> Vec<u8> {
        let mut pids = Vec::new();

        // Try with spaces first
        if let Some(line) = response.lines().find(|l| l.contains(prefix)) {
            let bytes: Vec<u8> = line
                .split_whitespace()
                .skip(2)  // Skip "41 XX"
                .take(4)
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            Self::decode_bitmap(&bytes, base, &mut pids);
        } else {
            // Try without spaces
            let no_space = prefix.replace(" ", "");
            let clean = response.replace(" ", "");
            if let Some(start) = clean.find(&no_space) {
                let data_start = start + no_space.len();
                if data_start + 8 <= clean.len() {
                    let bytes: Vec<u8> = (0..8)
                        .step_by(2)
                        .filter_map(|i| u8::from_str_radix(&clean[data_start+i..data_start+i+2], 16).ok())
                        .collect();
                    Self::decode_bitmap(&bytes, base, &mut pids);
                }
            }
        }

        pids
    }

    /// Decode 4-byte bitmask into PID list
    fn decode_bitmap(bytes: &[u8], base: u8, pids: &mut Vec<u8>) {
        for (byte_idx, &byte) in bytes.iter().enumerate() {
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    pids.push(base + (byte_idx * 8 + bit + 1) as u8);
                }
            }
        }
    }

    /// Check if a PID is supported by the vehicle
    pub fn is_pid_supported(&self, pid: u8) -> bool {
        self.supported_pids.contains(&pid) || self.supported_pids_ext.contains(&pid)
    }

    // ==================== BATTERY VOLTAGE ====================

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

    // ==================== HEADERS MANAGEMENT ====================

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

    // ==================== KEEP-ALIVE / HEALTH ====================

    /// Send TesterPresent to keep ECU session alive (call every 2-3s during idle)
    pub fn tester_present(&mut self) -> bool {
        match self.send_command_timeout("3E00", 2000) {
            Ok(r) => r.contains("7E") || r.contains("7e"),
            Err(_) => false,
        }
    }

    /// Check if the connection is still healthy
    pub fn health_check(&mut self) -> bool {
        // Quick check: read voltage (AT command, doesn't go to CAN bus)
        if let Ok(response) = self.send_command_timeout("ATRV", 2000) {
            if response.contains('.') && !response.contains("ERROR") {
                self.consecutive_errors = 0;
                return true;
            }
        }
        self.consecutive_errors += 1;
        false
    }

    /// Attempt to recover a broken connection (call when consecutive errors > 3)
    pub fn attempt_recovery(&mut self) -> Result<(), String> {
        dev_log::log_warn("obd", &format!("Attempting recovery (consecutive errors: {})", self.consecutive_errors));

        // Step 1: Flush buffer
        self.flush_buffer();
        std::thread::sleep(Duration::from_millis(300));

        // Step 2: Send CRs to reset adapter state
        if let Some(ref mut transport) = self.transport {
            let _ = transport.write_bytes(b"\r\r\r");
            let _ = transport.flush();
        }
        std::thread::sleep(Duration::from_millis(200));
        self.flush_buffer();

        // Step 3: Re-configure
        let _ = self.send_command("ATE0");
        let _ = self.send_command("ATS1");

        // Step 4: Verify with ATRV
        if let Ok(response) = self.send_command_timeout("ATRV", 2000) {
            if response.contains('.') {
                // Step 5: Restore protocol
                if !self.protocol_num.is_empty() {
                    let _ = self.send_command(&format!("ATSP{}", self.protocol_num));
                }
                self.consecutive_errors = 0;
                dev_log::log_info("obd", "Recovery successful");
                return Ok(());
            }
        }

        // Step 6: If still failing, try full re-detect
        let _ = self.send_command("ATSP0");
        std::thread::sleep(Duration::from_millis(300));
        if let Ok(response) = self.send_command_timeout("0100", 8000) {
            if self.check_valid_pid_response(&response, "41 00") {
                let proto = self.send_command("ATDPN").unwrap_or_default();
                self.protocol_num = proto.trim().chars().last().map(|c| c.to_string()).unwrap_or_default();
                self.protocol = Self::decode_protocol(&proto);
                self.consecutive_errors = 0;
                dev_log::log_info("obd", "Recovery via re-detect successful");
                return Ok(());
            }
        }

        Err("Recovery failed — adapter may need physical reconnection".to_string())
    }

    // ==================== SEND / RECEIVE ====================

    /// Send command with default timeout
    pub fn send_command(&mut self, cmd: &str) -> Result<String, String> {
        self.send_command_timeout(cmd, self.config.timeout_ms)
    }

    /// Send command with custom timeout — core communication method
    /// Works over any OBDTransport (serial, WiFi, BLE)
    pub fn send_command_timeout(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, String> {
        let transport = self.transport.as_mut().ok_or("Not connected")?;

        // Log TX
        dev_log::log_tx(cmd);

        // Inter-command delay — clones need more time to process
        let delay = match self.chip_type {
            ChipType::CloneV15 | ChipType::CloneV21 => 50,
            ChipType::Stn1110 | ChipType::Stn2120 => 15,
            _ => 30,
        };
        // WiFi adds latency — increase delay
        let delay = if transport.transport_type() == TransportType::WiFi {
            delay.max(80)
        } else {
            delay
        };

        let elapsed = self.last_command_time.elapsed().as_millis() as u64;
        if elapsed < delay {
            std::thread::sleep(Duration::from_millis(delay - elapsed));
        }

        // Send command
        let cmd_bytes = format!("{}\r", cmd);
        transport.write_bytes(cmd_bytes.as_bytes())
            .map_err(|e| { dev_log::log_error("obd", &format!("Write error: {}", e)); e })?;
        transport.flush().map_err(|e| format!("Flush error: {}", e))?;

        self.last_command_time = std::time::Instant::now();

        // Read response (wait for prompt '>')
        let mut response = String::new();
        let mut buf = [0u8; 256]; // Larger buffer for multi-frame responses
        let timeout = std::time::Instant::now();

        // WiFi needs more generous timeout due to TCP latency
        let effective_timeout = if transport.transport_type() == TransportType::WiFi {
            timeout_ms.max(3000)
        } else {
            timeout_ms
        };

        while timeout.elapsed() < Duration::from_millis(effective_timeout) {
            match transport.read_bytes(&mut buf, 100) {
                Ok(n) if n > 0 => {
                    for &byte in &buf[..n] {
                        let ch = byte as char;
                        if ch == '>' {
                            let clean = Self::clean_response(&response);
                            dev_log::log_rx(cmd, &clean);
                            self.consecutive_errors = 0;
                            return Ok(clean);
                        }
                        response.push(ch);
                    }
                }
                Ok(_) => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => {
                    self.consecutive_errors += 1;
                    return Err("Transport read error".to_string());
                }
            }
        }

        // Timeout — return what we have if it looks useful
        if !response.is_empty() {
            let clean = Self::clean_response(&response);
            // If we got a partial valid response, use it
            if clean.contains("41") || clean.contains("62") || clean.contains("7E") || clean.contains("59") {
                dev_log::log_rx(cmd, &format!("(partial) {}", clean));
                return Ok(clean);
            }
            dev_log::log_rx(cmd, &format!("(timeout) {}", clean));
            Ok(clean)
        } else {
            self.consecutive_errors += 1;
            dev_log::log_error("obd", &format!("Timeout: {} ({}ms)", cmd, timeout_ms));
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
                && !l.starts_with("LP ALERT")
                && !l.starts_with("LV RESET")
                && !l.starts_with("ACT ALERT")
                && !l.starts_with("ERR")
                // NOTE: "NO DATA" is NOT filtered — it carries semantic meaning
                // (PID not supported, ECU not responding) that callers check for
                && !l.starts_with("TIMEOUT")
                && *l != "?"
                // NOTE: "OK" is kept for AT commands (only valid response),
                // but filtered from OBD data responses where it's noise
                // Filter echo residues (AT commands echoed back)
                && !l.starts_with("AT")
                && !l.starts_with("at")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Flush input buffer (discard any pending data)
    fn flush_buffer(&mut self) {
        if let Some(ref mut transport) = self.transport {
            let mut buf = [0u8; 1024];
            // Read and discard all pending data (up to 10 rounds to be sure)
            for _ in 0..10 {
                match transport.read_bytes(&mut buf, 50) {
                    Ok(0) => break,
                    Ok(_) => continue,
                    Err(_) => break,
                }
            }
        }
    }

    // ==================== PID QUERY WITH RESILIENCE ====================

    /// Send OBD-II PID request with retry logic and error recovery
    pub fn query_pid(&mut self, mode: u8, pid: u8) -> Result<Vec<u8>, String> {
        let cmd = format!("{:02X}{:02X}", mode, pid);
        let expected_prefix = format!("{:02X} {:02X}", mode + 0x40, pid);

        // If we know supported PIDs and this one isn't listed, skip it
        if mode == 0x01 && !self.supported_pids.is_empty() && !self.is_pid_supported(pid) {
            return Err(format!("PID {:02X} not in supported list", pid));
        }

        // Try up to 3 times with escalating recovery
        for attempt in 0..3 {
            let response = match self.send_command(&cmd) {
                Ok(r) => r,
                Err(e) => {
                    if attempt < 2 {
                        debug!("PID {:02X} attempt {} failed: {}, retrying...", pid, attempt + 1, e);
                        // Escalating recovery
                        if attempt == 0 {
                            std::thread::sleep(Duration::from_millis(100));
                        } else {
                            // On second retry, try recovery sequence
                            dev_log::log_warn("obd", &format!("PID {:02X} retry with recovery", pid));
                            let _ = self.send_command("3E00"); // TesterPresent wake-up
                            std::thread::sleep(Duration::from_millis(200));
                        }
                        continue;
                    }
                    return Err(e);
                }
            };

            // Check for "NO DATA" — PID not supported by this ECU
            if response.contains("NO DATA") {
                return Err(format!("PID {:02X} not supported (NO DATA)", pid));
            }

            // Check for negative response (7F = error)
            if response.contains("7F") {
                // 7F XX 31 = serviceNotSupported, 7F XX 12 = subFunctionNotSupported
                return Err(format!("PID {:02X} negative response: {}", pid, response));
            }

            // Parse hex response: "41 0C 1A F8" (with spaces)
            if let Some(data_str) = response.lines().find(|l| l.contains(&expected_prefix)) {
                let bytes: Vec<u8> = data_str
                    .split_whitespace()
                    .skip(2)
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect();
                if !bytes.is_empty() {
                    return Ok(bytes);
                }
            }

            // Try without spaces (some adapters strip spaces)
            let no_space_prefix = format!("{:02X}{:02X}", mode + 0x40, pid);
            let hex_str = response.replace(" ", "").replace("\n", "");
            if let Some(start) = hex_str.find(&no_space_prefix) {
                let data_start = start + no_space_prefix.len();
                if data_start < hex_str.len() {
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

            // Multi-line response: some vehicles return data on a different line
            for line in response.lines() {
                let trimmed = line.trim();
                if trimmed.len() >= 4 {
                    let bytes: Vec<u8> = trimmed
                        .split_whitespace()
                        .filter_map(|s| u8::from_str_radix(s, 16).ok())
                        .collect();
                    if bytes.len() >= 3 && bytes[0] == mode + 0x40 && bytes[1] == pid {
                        return Ok(bytes[2..].to_vec());
                    }
                }
            }

            if attempt < 2 {
                debug!("PID {:02X} parse failed, retrying...", pid);
                std::thread::sleep(Duration::from_millis(50 * (attempt as u64 + 1)));
            }
        }

        Err(format!("Invalid response for PID {:02X} after 3 attempts", pid))
    }

    /// Query a UDS DID (Service 0x22) with multi-frame support
    pub fn query_did(&mut self, did: u16) -> Result<Vec<u8>, String> {
        let cmd = format!("22{:04X}", did);
        let response = self.send_command_timeout(&cmd, 3000)?;

        if response.contains("NO DATA") {
            return Err(format!("DID {:04X} not supported", did));
        }

        if response.contains("7F") {
            return Err(format!("DID {:04X} negative response", did));
        }

        // Parse with spaces — match "62" as UDS positive response token (not as data byte)
        let did_high = format!("{:02X}", (did >> 8) & 0xFF);
        let did_low = format!("{:02X}", did & 0xFF);
        if let Some(line) = response.lines().find(|l| {
            let tokens: Vec<&str> = l.split_whitespace().collect();
            tokens.len() >= 3 && tokens.iter().position(|t| *t == "62")
                .map(|p| p + 2 < tokens.len() && tokens[p+1] == did_high && tokens[p+2] == did_low)
                .unwrap_or(false)
        }) {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let pos = tokens.iter().position(|t| *t == "62").unwrap();
            let bytes: Vec<u8> = tokens[pos+3..]
                .iter()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            if !bytes.is_empty() {
                return Ok(bytes);
            }
        }

        // Parse without spaces
        let no_space_prefix = format!("62{:04X}", did);
        let hex_str = response.replace(" ", "").replace("\n", "");
        if let Some(start) = hex_str.find(&no_space_prefix) {
            let data_start = start + no_space_prefix.len();
            if data_start < hex_str.len() {
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

        Err(format!("Invalid response for DID {:04X}", did))
    }

    // ==================== DISCONNECT ====================

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

    /// Decode protocol number to human-readable name
    fn decode_protocol(proto_str: &str) -> String {
        let num = proto_str.trim().chars().last().unwrap_or('?');
        match num {
            '0' => "Auto",
            '1' => "SAE J1850 PWM",
            '2' => "SAE J1850 VPW",
            '3' => "ISO 9141-2",
            '4' => "ISO 14230-4 KWP (5 baud init)",
            '5' => "ISO 14230-4 KWP (fast init)",
            '6' => "ISO 15765-4 CAN 11-bit 500k",
            '7' => "ISO 15765-4 CAN 29-bit 500k",
            '8' => "ISO 15765-4 CAN 11-bit 250k",
            '9' => "ISO 15765-4 CAN 29-bit 250k",
            'A' | 'a' => "SAE J1939 CAN 29-bit 250k",
            'B' | 'b' => "User CAN 1 (11-bit, user baud)",
            'C' | 'c' => "User CAN 2 (29-bit, user baud)",
            _ => "Unknown",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_response_filters_searching() {
        let raw = "SEARCHING\n41 0C 10 20\nSEARCHING";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("SEARCHING"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_bus_init() {
        let raw = "BUS INIT\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("BUS INIT"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_can_error() {
        let raw = "CAN ERROR\n41 0C 10 20\nCAN ERROR";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("CAN ERROR"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_buffer_full() {
        let raw = "BUFFER FULL\nDATA ERROR\n41 0C";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("BUFFER FULL"));
        assert!(!cleaned.contains("DATA ERROR"));
        assert!(cleaned.contains("41 0C"));
    }

    #[test]
    fn test_clean_response_filters_at_echo() {
        let raw = "ATE0\nOK\nATE1\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("ATE0"));
        assert!(!cleaned.contains("ATE1"));
        // OK is kept from AT commands but filtered from data responses
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_preserves_no_data() {
        // NO DATA is semantic information — should NOT be filtered
        let raw = "41 0C\nNO DATA\n41 0D";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.contains("NO DATA"));
    }

    #[test]
    fn test_clean_response_removes_empty_lines() {
        let raw = "41 0C 10 20\n\n\n41 0D 05 10\n";
        let cleaned = Elm327Connection::clean_response(raw);
        // Should have two lines, no empties
        let lines: Vec<&str> = cleaned.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| !l.is_empty()));
    }

    #[test]
    fn test_clean_response_normalizes_line_endings() {
        let raw = "41 0C 10 20\r\n41 0D 05 10\r\n";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.contains("41 0C 10 20"));
        assert!(cleaned.contains("41 0D 05 10"));
        assert!(!cleaned.contains("\r"));
    }

    #[test]
    fn test_clean_response_trims_whitespace() {
        let raw = "  41 0C 10 20  \n  41 0D 05 10  ";
        let cleaned = Elm327Connection::clean_response(raw);
        let lines: Vec<&str> = cleaned.lines().collect();
        assert_eq!(lines[0], "41 0C 10 20");
        assert_eq!(lines[1], "41 0D 05 10");
    }

    #[test]
    fn test_clean_response_filters_unable_to_connect() {
        let raw = "UNABLE TO CONNECT\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("UNABLE TO CONNECT"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_bus_busy() {
        let raw = "BUS BUSY\nFB ERROR\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("BUS BUSY"));
        assert!(!cleaned.contains("FB ERROR"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_filters_err() {
        let raw = "ERR\n41 0C 10 20\nERROR";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("ERR"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

    #[test]
    fn test_clean_response_mixed_noise() {
        let raw = "SEARCHING\nBUS INIT\n41 0C 10 20\nCAN ERROR\nSEARCHING\n\nNO DATA";
        let cleaned = Elm327Connection::clean_response(raw);

        // Should keep valid data and NO DATA
        assert!(cleaned.contains("41 0C 10 20"));
        assert!(cleaned.contains("NO DATA"));

        // Should remove noise
        assert!(!cleaned.contains("SEARCHING"));
        assert!(!cleaned.contains("BUS INIT"));
        assert!(!cleaned.contains("CAN ERROR"));
    }

    #[test]
    fn test_clean_response_empty_input() {
        let raw = "";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_response_only_noise() {
        let raw = "SEARCHING\nBUS INIT\nCAN ERROR";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_response_question_mark_filtered() {
        let raw = "?\n41 0C 10 20";
        let cleaned = Elm327Connection::clean_response(raw);
        assert!(!cleaned.contains("?"));
        assert!(cleaned.contains("41 0C 10 20"));
    }

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
