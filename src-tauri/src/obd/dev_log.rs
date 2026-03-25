use std::sync::Mutex;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

/// A single dev log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,    // "TX", "RX", "INFO", "WARN", "ERROR", "DEBUG"
    pub source: String,   // "obd", "dtc", "ecu", "db", "ui", "safety"
    pub message: String,
}

const MAX_ENTRIES: usize = 5000;

static LOG_BUFFER: Mutex<Option<VecDeque<LogEntry>>> = Mutex::new(None);

fn get_buffer() -> std::sync::MutexGuard<'static, Option<VecDeque<LogEntry>>> {
    LOG_BUFFER.lock().unwrap_or_else(|e| e.into_inner())
}

fn now_str() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

/// Log an OBD TX command
pub fn log_tx(cmd: &str) {
    push(LogEntry {
        timestamp: now_str(),
        level: "TX".into(),
        source: "obd".into(),
        message: format!(">>> {}", cmd),
    });
}

/// Log an OBD RX response
pub fn log_rx(cmd: &str, response: &str) {
    push(LogEntry {
        timestamp: now_str(),
        level: "RX".into(),
        source: "obd".into(),
        message: format!("<<< {} → {}", cmd, response),
    });
}

/// Log info
pub fn log_info(source: &str, message: &str) {
    push(LogEntry {
        timestamp: now_str(),
        level: "INFO".into(),
        source: source.into(),
        message: message.into(),
    });
}

/// Log warning
pub fn log_warn(source: &str, message: &str) {
    push(LogEntry {
        timestamp: now_str(),
        level: "WARN".into(),
        source: source.into(),
        message: message.into(),
    });
}

/// Log error
pub fn log_error(source: &str, message: &str) {
    push(LogEntry {
        timestamp: now_str(),
        level: "ERROR".into(),
        source: source.into(),
        message: message.into(),
    });
}

/// Log debug
pub fn log_debug(source: &str, message: &str) {
    push(LogEntry {
        timestamp: now_str(),
        level: "DEBUG".into(),
        source: source.into(),
        message: message.into(),
    });
}

fn push(entry: LogEntry) {
    let mut guard = get_buffer();
    let buf = guard.get_or_insert_with(|| VecDeque::with_capacity(MAX_ENTRIES));
    if buf.len() >= MAX_ENTRIES {
        buf.pop_front();
    }
    buf.push_back(entry);
}

/// Get all logs (for the dev console)
pub fn get_all_logs() -> Vec<LogEntry> {
    let guard = get_buffer();
    guard.as_ref().map(|b| b.iter().cloned().collect()).unwrap_or_default()
}

/// Get logs since a given index (for incremental polling)
pub fn get_logs_since(since_index: usize) -> Vec<LogEntry> {
    let guard = get_buffer();
    guard.as_ref()
        .map(|b| b.iter().skip(since_index).cloned().collect())
        .unwrap_or_default()
}

/// Get total log count
pub fn log_count() -> usize {
    let guard = get_buffer();
    guard.as_ref().map(|b| b.len()).unwrap_or(0)
}

/// Clear all logs
pub fn clear_logs() {
    let mut guard = get_buffer();
    if let Some(buf) = guard.as_mut() {
        buf.clear();
    }
}
