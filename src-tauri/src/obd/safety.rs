use crate::models::RiskLevel;
use tracing::warn;

/// Blocked UDS service IDs (never allowed)
const BLOCKED_SERVICES: &[u8] = &[
    0x2E, // WriteDataByIdentifier
    0x2F, // InputOutputControlByIdentifier
    0x31, // RoutineControl
    0x3D, // WriteMemoryByAddress
    0x34, // RequestDownload
    0x35, // RequestUpload
    0x36, // TransferData
    0x37, // RequestTransferExit
    0x11, // ECUReset
    0x27, // SecurityAccess
    0x28, // CommunicationControl
];

/// Safe UDS service IDs (always allowed)
const SAFE_SERVICES: &[u8] = &[
    0x10, // DiagnosticSessionControl
    0x19, // ReadDTCInformation
    0x22, // ReadDataByIdentifier
    0x3E, // TesterPresent
];

/// Service IDs requiring confirmation
const CAUTION_SERVICES: &[u8] = &[
    0x14, // ClearDiagnosticInformation
];

/// Safety guard for OBD/UDS operations
pub struct SafetyGuard;

impl SafetyGuard {
    /// Check if a command is safe to execute
    pub fn check_command(command: &str) -> RiskLevel {
        let trimmed = command.trim().to_uppercase();

        // AT command handling
        if trimmed.starts_with("AT") {
            return Self::classify_at_command(&trimmed);
        }

        let bytes: Vec<u8> = trimmed
            .split_whitespace()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();

        if bytes.is_empty() {
            return RiskLevel::Safe;
        }

        let service_id = bytes[0];
        Self::classify_service(service_id)
    }

    /// Classify a UDS service ID
    pub fn classify_service(service_id: u8) -> RiskLevel {
        if BLOCKED_SERVICES.contains(&service_id) {
            warn!("Blocked service 0x{:02X} attempted", service_id);
            return RiskLevel::Blocked;
        }

        if SAFE_SERVICES.contains(&service_id) {
            return RiskLevel::Safe;
        }

        if CAUTION_SERVICES.contains(&service_id) {
            return RiskLevel::Caution;
        }

        // OBD-II modes
        match service_id {
            0x01..=0x03 | 0x05..=0x07 | 0x09 | 0x0A => RiskLevel::Safe,
            0x04 => RiskLevel::Caution, // Clear DTC
            0x08 => RiskLevel::Blocked, // Control actuators
            _ => RiskLevel::Dangerous,
        }
    }

    /// Classify AT commands
    fn classify_at_command(cmd: &str) -> RiskLevel {
        let blocked = ["ATMA", "ATBD", "ATBI", "ATPP", "ATWS"];
        let caution = ["ATZ", "ATD", "ATRV"];

        for b in blocked {
            if cmd.starts_with(b) {
                return RiskLevel::Blocked;
            }
        }
        for c in caution {
            if cmd.starts_with(c) {
                return RiskLevel::Caution;
            }
        }
        RiskLevel::Safe
    }

    /// Check command in ADVANCED mode — allows write services (2E, 2F, 30, 31)
    /// but still blocks truly dangerous ones (ECUReset, SecurityAccess, Download/Upload)
    pub fn check_command_advanced(command: &str) -> RiskLevel {
        let trimmed = command.trim().to_uppercase();

        if trimmed.starts_with("AT") {
            return Self::classify_at_command(&trimmed);
        }

        let bytes: Vec<u8> = trimmed
            .split_whitespace()
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();

        if bytes.is_empty() {
            return RiskLevel::Safe;
        }

        let service_id = bytes[0];

        // Still blocked even in advanced mode — too dangerous
        const ALWAYS_BLOCKED: &[u8] = &[
            0x3D, // WriteMemoryByAddress
            0x34, // RequestDownload
            0x35, // RequestUpload
            0x36, // TransferData
            0x37, // RequestTransferExit
            0x11, // ECUReset
            0x27, // SecurityAccess
            0x28, // CommunicationControl
        ];

        if ALWAYS_BLOCKED.contains(&service_id) {
            warn!("Service 0x{:02X} blocked even in advanced mode", service_id);
            return RiskLevel::Blocked;
        }

        // These are allowed in advanced mode as Caution (need confirmation)
        const ADVANCED_ALLOWED: &[u8] = &[
            0x2E, // WriteDataByIdentifier
            0x2F, // InputOutputControlByIdentifier
            0x30, // InputOutputControl (actuator test)
            0x31, // RoutineControl
        ];

        if ADVANCED_ALLOWED.contains(&service_id) {
            return RiskLevel::Caution;
        }

        // Everything else: normal classification
        Self::classify_service(service_id)
    }

    /// Validate hex command string
    pub fn validate_hex(command: &str) -> Result<(), String> {
        for part in command.split_whitespace() {
            if part.len() > 2 {
                return Err(format!("Invalid hex byte: '{}'", part));
            }
            u8::from_str_radix(part, 16)
                .map_err(|_| format!("Invalid hex: '{}'", part))?;
        }
        Ok(())
    }
}
