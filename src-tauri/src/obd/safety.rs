use crate::models::RiskLevel;
use tracing::warn;

/// Blocked UDS service IDs (never allowed in normal mode)
const BLOCKED_SERVICES: &[u8] = &[
    0x08, // RequestControlOfOnBoardSystem
    0x2E, // WriteDataByIdentifier
    0x2F, // InputOutputControlByIdentifier
    0x30, // InputOutputControl (actuator test)
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
    0x19, // ReadDTCInformation
    0x22, // ReadDataByIdentifier
    0x3E, // TesterPresent
];

/// Service IDs requiring confirmation
const CAUTION_SERVICES: &[u8] = &[
    0x10, // DiagnosticSessionControl (can open programming sessions)
    0x14, // ClearDiagnosticInformation
];

/// Safety guard for OBD/UDS operations
pub struct SafetyGuard;

impl SafetyGuard {
    /// Extract service ID from a hex command string (handles both spaced and unspaced formats)
    /// "2E F1 90" → 0x2E, "2EF190" → 0x2E, "22 F1 90" → 0x22
    fn parse_service_id(command: &str) -> Option<u8> {
        let trimmed = command.trim().to_uppercase();
        if trimmed.is_empty() {
            return None;
        }

        // Try spaced format first: "2E F1 90"
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts[0].len() == 2 {
            return u8::from_str_radix(parts[0], 16).ok();
        }

        // Unspaced format: "2EF190" — first 2 chars are the service ID
        if trimmed.len() >= 2 {
            return u8::from_str_radix(&trimmed[..2], 16).ok();
        }

        None
    }

    /// Check if a command is safe to execute (normal mode — blocks write services)
    pub fn check_command(command: &str) -> RiskLevel {
        let trimmed = command.trim().to_uppercase();

        // AT command handling
        if trimmed.starts_with("AT") {
            return Self::classify_at_command(&trimmed);
        }

        // OBD-II Mode commands (single digit modes): "03", "07", "0A", "04"
        // Also handles "0100", "0902", etc.
        match Self::parse_service_id(&trimmed) {
            Some(service_id) => Self::classify_service(service_id),
            None => RiskLevel::Blocked, // Default-deny: unparseable commands are blocked
        }
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

        let service_id = match Self::parse_service_id(&trimmed) {
            Some(id) => id,
            None => return RiskLevel::Blocked, // Default-deny: unparseable commands are blocked
        };

        // Still blocked even in advanced mode — too dangerous
        const ALWAYS_BLOCKED: &[u8] = &[
            0x08, // RequestControlOfOnBoardSystem
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

    /// Validate hex command string — supports both "2E F1 90" and "2EF190" formats
    pub fn validate_hex(command: &str) -> Result<(), String> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err("Empty command".to_string());
        }

        // Check if spaced format
        if trimmed.contains(' ') {
            for part in trimmed.split_whitespace() {
                if part.len() > 2 {
                    return Err(format!("Invalid hex byte: '{}'", part));
                }
                u8::from_str_radix(part, 16)
                    .map_err(|_| format!("Invalid hex: '{}'", part))?;
            }
        } else {
            // Unspaced format: validate all chars are hex and length is even
            if trimmed.len() % 2 != 0 {
                return Err(format!("Odd-length hex string: '{}'", trimmed));
            }
            for i in (0..trimmed.len()).step_by(2) {
                u8::from_str_radix(&trimmed[i..i+2], 16)
                    .map_err(|_| format!("Invalid hex at position {}: '{}'", i, &trimmed[i..i+2]))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_services_always_blocked() {
        // Always blocked in normal mode
        assert_eq!(SafetyGuard::check_command("11"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("27"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("28"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("34"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("35"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("36"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("37"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("3D"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("08"), RiskLevel::Blocked);

        // Always blocked in advanced mode too
        assert_eq!(SafetyGuard::check_command_advanced("11"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("27"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("28"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("34"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("35"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("36"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("37"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("3D"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("08"), RiskLevel::Blocked);
    }

    #[test]
    fn test_read_services_allowed() {
        // OBD-II read modes should be safe in normal mode
        assert_eq!(SafetyGuard::check_command("01"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("02"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("03"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("07"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("09"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("0A"), RiskLevel::Safe);

        // UDS read services
        assert_eq!(SafetyGuard::check_command("22"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("3E"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("19"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("10"), RiskLevel::Caution);
    }

    #[test]
    fn test_write_services_blocked_in_normal() {
        // Write services should be blocked in normal mode
        assert_eq!(SafetyGuard::check_command("2E"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("2F"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("30"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("31"), RiskLevel::Blocked);

        // But allowed (as Caution) in advanced mode
        assert_eq!(SafetyGuard::check_command_advanced("2E"), RiskLevel::Caution);
        assert_eq!(SafetyGuard::check_command_advanced("2F"), RiskLevel::Caution);
        assert_eq!(SafetyGuard::check_command_advanced("30"), RiskLevel::Caution);
        assert_eq!(SafetyGuard::check_command_advanced("31"), RiskLevel::Caution);
    }

    #[test]
    fn test_mode04_clear_dtc() {
        // Mode 04 (Clear DTCs) should be Caution in normal mode
        assert_eq!(SafetyGuard::check_command("04"), RiskLevel::Caution);
        assert_eq!(SafetyGuard::check_command_advanced("04"), RiskLevel::Caution);
    }

    #[test]
    fn test_validate_hex_valid() {
        // Spaced format
        assert!(SafetyGuard::validate_hex("01 0C").is_ok());
        assert!(SafetyGuard::validate_hex("22 F1 90").is_ok());
        assert!(SafetyGuard::validate_hex("2E 01 23 AB").is_ok());

        // Unspaced format
        assert!(SafetyGuard::validate_hex("010C").is_ok());
        assert!(SafetyGuard::validate_hex("22F190").is_ok());
        assert!(SafetyGuard::validate_hex("2E0123AB").is_ok());

        // With leading/trailing whitespace
        assert!(SafetyGuard::validate_hex("  01 0C  ").is_ok());
        assert!(SafetyGuard::validate_hex("  010C  ").is_ok());
    }

    #[test]
    fn test_validate_hex_invalid() {
        // Non-hex characters
        assert!(SafetyGuard::validate_hex("hello").is_err());
        assert!(SafetyGuard::validate_hex("GG").is_err());
        assert!(SafetyGuard::validate_hex("ZZ FF").is_err());

        // Odd-length unspaced
        assert!(SafetyGuard::validate_hex("010").is_err());
        assert!(SafetyGuard::validate_hex("1A2").is_err());

        // Empty
        assert!(SafetyGuard::validate_hex("").is_err());
        assert!(SafetyGuard::validate_hex("   ").is_err());

        // Byte too long in spaced format
        assert!(SafetyGuard::validate_hex("010 0C").is_err());
        assert!(SafetyGuard::validate_hex("01 0CD").is_err());
    }

    #[test]
    fn test_blocked_at_commands() {
        assert_eq!(SafetyGuard::check_command("ATMA"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("ATBD"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("ATBI"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("ATPP"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("ATWS"), RiskLevel::Blocked);

        // Should be blocked in advanced mode too
        assert_eq!(SafetyGuard::check_command_advanced("ATMA"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command_advanced("ATWS"), RiskLevel::Blocked);
    }

    #[test]
    fn test_safe_at_commands() {
        assert_eq!(SafetyGuard::check_command("ATE0"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("ATH1"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("ATS0"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("ATSP6"), RiskLevel::Safe);
    }

    #[test]
    fn test_caution_at_commands() {
        assert_eq!(SafetyGuard::check_command("ATZ"), RiskLevel::Caution);
        assert_eq!(SafetyGuard::check_command("ATD"), RiskLevel::Caution);
        assert_eq!(SafetyGuard::check_command("ATRV"), RiskLevel::Caution);
    }

    #[test]
    fn test_parse_service_id_spaced() {
        assert_eq!(SafetyGuard::check_command("2E F1 90"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("22 F1 90"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("01 0C"), RiskLevel::Safe);
    }

    #[test]
    fn test_parse_service_id_unspaced() {
        assert_eq!(SafetyGuard::check_command("2EF190"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("22F190"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("010C"), RiskLevel::Safe);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(SafetyGuard::check_command("2e"), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("22f190"), RiskLevel::Safe);
        assert_eq!(SafetyGuard::check_command("atz"), RiskLevel::Caution);
    }

    #[test]
    fn test_unparseable_commands() {
        assert_eq!(SafetyGuard::check_command(""), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("   "), RiskLevel::Blocked);
        assert_eq!(SafetyGuard::check_command("INVALID"), RiskLevel::Blocked);
    }
}
