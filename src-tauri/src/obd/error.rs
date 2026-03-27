use std::fmt;

/// Unified error type for all OBD operations
#[derive(Debug, Clone)]
pub enum ObdError {
    // Transport layer
    SerialPort(String),
    WiFi(String),
    Ble(String),
    TransportRead(String),
    TransportWrite(String),

    // Connection state
    NotConnected,
    AlreadyConnected,
    DemoMode,

    // OBD protocol
    AllInitFailed,
    NoProtocolFound,
    CommandTimeout(String),
    InvalidResponse(String),
    NoData(String),
    NegativeResponse { pid: String, raw: String },

    // Safety
    CommandBlocked(String),
    ConfirmationRequired(String),

    // Database
    Database(String),

    // File I/O
    Io(String),
    PathTraversal(String),

    // Parsing
    InvalidHex(String),
    InvalidFormat(String),

    // Task/async
    TaskError(String),
}

impl fmt::Display for ObdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerialPort(e) => write!(f, "Serial port error: {}", e),
            Self::WiFi(e) => write!(f, "WiFi error: {}", e),
            Self::Ble(e) => write!(f, "BLE error: {}", e),
            Self::TransportRead(e) => write!(f, "Transport read error: {}", e),
            Self::TransportWrite(e) => write!(f, "Transport write error: {}", e),
            Self::NotConnected => write!(f, "Not connected"),
            Self::AlreadyConnected => write!(f, "Already connected"),
            Self::DemoMode => write!(f, "Operation not available in demo mode"),
            Self::AllInitFailed => write!(f, "All 4 connection strategies failed"),
            Self::NoProtocolFound => write!(f, "No compatible OBD protocol found"),
            Self::CommandTimeout(cmd) => write!(f, "Timeout on command: {}", cmd),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::NoData(pid) => write!(f, "{} not supported (NO DATA)", pid),
            Self::NegativeResponse { pid, raw } => write!(f, "{} negative response: {}", pid, raw),
            Self::CommandBlocked(cmd) => write!(f, "Command blocked: {}", cmd),
            Self::ConfirmationRequired(cmd) => write!(f, "Confirmation required for: {}", cmd),
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::PathTraversal(path) => write!(f, "Path traversal blocked: {}", path),
            Self::InvalidHex(s) => write!(f, "Invalid hex: {}", s),
            Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            Self::TaskError(e) => write!(f, "Task error: {}", e),
        }
    }
}

impl std::error::Error for ObdError {}

/// Convert ObdError to String for Tauri command compatibility
/// This allows gradual migration — new code uses ObdError, Tauri commands convert to String
impl From<ObdError> for String {
    fn from(err: ObdError) -> String {
        err.to_string()
    }
}

impl From<std::io::Error> for ObdError {
    fn from(err: std::io::Error) -> Self {
        ObdError::Io(err.to_string())
    }
}

impl From<rusqlite::Error> for ObdError {
    fn from(err: rusqlite::Error) -> Self {
        ObdError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for ObdError {
    fn from(err: serde_json::Error) -> Self {
        ObdError::InvalidFormat(err.to_string())
    }
}

impl From<tokio::task::JoinError> for ObdError {
    fn from(err: tokio::task::JoinError) -> Self {
        ObdError::TaskError(err.to_string())
    }
}

#[cfg(feature = "desktop")]
impl From<serialport::Error> for ObdError {
    fn from(err: serialport::Error) -> Self {
        ObdError::SerialPort(err.to_string())
    }
}

/// Convenience type alias
pub type ObdResult<T> = Result<T, ObdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_serial_port() {
        let err = ObdError::SerialPort("connection failed".to_string());
        assert_eq!(err.to_string(), "Serial port error: connection failed");
    }

    #[test]
    fn test_display_wifi() {
        let err = ObdError::WiFi("network error".to_string());
        assert_eq!(err.to_string(), "WiFi error: network error");
    }

    #[test]
    fn test_display_ble() {
        let err = ObdError::Ble("pairing failed".to_string());
        assert_eq!(err.to_string(), "BLE error: pairing failed");
    }

    #[test]
    fn test_display_transport_read() {
        let err = ObdError::TransportRead("timeout".to_string());
        assert_eq!(err.to_string(), "Transport read error: timeout");
    }

    #[test]
    fn test_display_transport_write() {
        let err = ObdError::TransportWrite("failed".to_string());
        assert_eq!(err.to_string(), "Transport write error: failed");
    }

    #[test]
    fn test_display_not_connected() {
        let err = ObdError::NotConnected;
        assert_eq!(err.to_string(), "Not connected");
    }

    #[test]
    fn test_display_already_connected() {
        let err = ObdError::AlreadyConnected;
        assert_eq!(err.to_string(), "Already connected");
    }

    #[test]
    fn test_display_demo_mode() {
        let err = ObdError::DemoMode;
        assert_eq!(err.to_string(), "Operation not available in demo mode");
    }

    #[test]
    fn test_display_all_init_failed() {
        let err = ObdError::AllInitFailed;
        assert_eq!(err.to_string(), "All 4 connection strategies failed");
    }

    #[test]
    fn test_display_no_protocol_found() {
        let err = ObdError::NoProtocolFound;
        assert_eq!(err.to_string(), "No compatible OBD protocol found");
    }

    #[test]
    fn test_display_command_timeout() {
        let err = ObdError::CommandTimeout("0100".to_string());
        assert_eq!(err.to_string(), "Timeout on command: 0100");
    }

    #[test]
    fn test_display_invalid_response() {
        let err = ObdError::InvalidResponse("bad format".to_string());
        assert_eq!(err.to_string(), "Invalid response: bad format");
    }

    #[test]
    fn test_display_no_data() {
        let err = ObdError::NoData("0x0C".to_string());
        assert_eq!(err.to_string(), "0x0C not supported (NO DATA)");
    }

    #[test]
    fn test_display_negative_response() {
        let err = ObdError::NegativeResponse {
            pid: "0x05".to_string(),
            raw: "7F 01 31".to_string(),
        };
        assert_eq!(err.to_string(), "0x05 negative response: 7F 01 31");
    }

    #[test]
    fn test_display_command_blocked() {
        let err = ObdError::CommandBlocked("0x11 ECU Reset".to_string());
        assert_eq!(err.to_string(), "Command blocked: 0x11 ECU Reset");
    }

    #[test]
    fn test_display_confirmation_required() {
        let err = ObdError::ConfirmationRequired("Mode 04 Clear DTC".to_string());
        assert_eq!(
            err.to_string(),
            "Confirmation required for: Mode 04 Clear DTC"
        );
    }

    #[test]
    fn test_display_database() {
        let err = ObdError::Database("query failed".to_string());
        assert_eq!(err.to_string(), "Database error: query failed");
    }

    #[test]
    fn test_display_io() {
        let err = ObdError::Io("file not found".to_string());
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn test_display_path_traversal() {
        let err = ObdError::PathTraversal("../../../etc".to_string());
        assert_eq!(err.to_string(), "Path traversal blocked: ../../../etc");
    }

    #[test]
    fn test_display_invalid_hex() {
        let err = ObdError::InvalidHex("0xZZ".to_string());
        assert_eq!(err.to_string(), "Invalid hex: 0xZZ");
    }

    #[test]
    fn test_display_invalid_format() {
        let err = ObdError::InvalidFormat("malformed json".to_string());
        assert_eq!(err.to_string(), "Invalid format: malformed json");
    }

    #[test]
    fn test_display_task_error() {
        let err = ObdError::TaskError("join failed".to_string());
        assert_eq!(err.to_string(), "Task error: join failed");
    }

    #[test]
    fn test_from_obd_error_to_string() {
        let err = ObdError::NotConnected;
        let s: String = err.into();
        assert_eq!(s, "Not connected");
    }

    #[test]
    fn test_from_std_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let obd_err: ObdError = io_err.into();
        match obd_err {
            ObdError::Io(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let obd_err: ObdError = json_err.into();
        match obd_err {
            ObdError::InvalidFormat(_) => (),
            _ => panic!("Expected InvalidFormat variant"),
        }
    }

    #[test]
    fn test_from_rusqlite_error() {
        let db_err = rusqlite::Error::QueryReturnedNoRows;
        let obd_err: ObdError = db_err.into();
        match obd_err {
            ObdError::Database(_) => (),
            _ => panic!("Expected Database variant"),
        }
    }

    #[test]
    fn test_implements_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(ObdError::NotConnected);
        assert_eq!(err.to_string(), "Not connected");
    }
}
