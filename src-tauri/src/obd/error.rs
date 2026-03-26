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
