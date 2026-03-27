use std::time::Duration;
use tracing::{info, warn};
use crate::obd::dev_log;
use super::{Elm327Connection, ChipType};

impl Elm327Connection {
    // ==================== INIT STRATEGIES ====================

    /// Multi-strategy initialization — tries 4 approaches, then bails
    pub(super) fn init_with_resilience(&mut self) -> Result<(), String> {
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
        std::thread::sleep(Duration::from_millis(500));

        // Use ATI instead of ATZ — clones handle it better
        let ati_response = self.send_command_timeout("ATI", 3000)?;
        self.detect_chip_type(&ati_response);
        self.is_clone = true;

        // Send ATE0 twice — some clones need a warm-up command before they respond properly
        let _ = self.send_command_timeout("ATE0", 1500);
        std::thread::sleep(Duration::from_millis(100));
        let _ = self.send_command_timeout("ATE0", 1500);

        self.configure_adapter()?;
        // Verify adapter responds — some clones appear to init but are actually unresponsive
        match self.send_command_timeout("ATRV", 2000) {
            Ok(r) if r.contains('.') || r.contains("OK") => {},
            _ => return Err("Clone adapter did not respond to validation (ATRV)".to_string()),
        }
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
            let _ = transport.write_bytes(b"\r\r\r\r\r");
            let _ = transport.flush();
        }
        std::thread::sleep(Duration::from_millis(800));
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

        std::thread::sleep(Duration::from_millis(300));
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
}
