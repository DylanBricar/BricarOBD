use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::collections::VecDeque;
use std::io::Write;
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
static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
static EVICTED_COUNT: AtomicU64 = AtomicU64::new(0);

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

/// Push a log entry to the buffer and optionally write to file.
/// File write is decoupled from buffer lock to reduce contention.
fn push(entry: LogEntry) {
    // Clone entry for file write before taking the buffer lock
    let file_entry = entry.clone();

    {
        let mut guard = get_buffer();
        let buf = guard.get_or_insert_with(|| VecDeque::with_capacity(MAX_ENTRIES));
        if buf.len() >= MAX_ENTRIES {
            buf.pop_front();
            EVICTED_COUNT.fetch_add(1, Relaxed);
        }
        buf.push_back(entry);
    }
    // Buffer lock released — now write to file without holding it
    if let Ok(mut file_guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *file_guard {
            let _ = writeln!(file, "[{}] {} {} — {}", file_entry.timestamp, file_entry.level, file_entry.source, file_entry.message);
        }
    }
}

/// Get all logs (for the dev console)
pub fn get_all_logs() -> Vec<LogEntry> {
    let guard = get_buffer();
    guard.as_ref().map(|b| b.iter().cloned().collect()).unwrap_or_default()
}

/// Get logs since a given index (for incremental polling)
pub fn get_logs_since(since_index: usize) -> Vec<LogEntry> {
    let guard = get_buffer();
    let evicted = EVICTED_COUNT.load(Relaxed) as usize;
    let adjusted_index = if since_index > evicted {
        since_index - evicted
    } else {
        0
    };
    guard.as_ref()
        .map(|b| {
            b.iter()
                .skip(adjusted_index)
                .cloned()
                .collect()
        })
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

/// Initialize the log file for the session
pub fn init_log_file() {
    if let Some(log_dir) = crate::obd::vin_cache::get_log_dir_path() {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("bricarobd_{}.log", timestamp);
        let path = log_dir.join(&filename);

        match std::fs::File::create(&path) {
            Ok(mut file) => {
                let _ = writeln!(file, "BricarOBD Log Session Started — {}", chrono::Local::now());
                let _ = writeln!(file, "---");
                if let Ok(mut file_guard) = LOG_FILE.lock() {
                    *file_guard = Some(file);
                }
            }
            Err(e) => {
                eprintln!("Failed to create log file: {}", e);
            }
        }
    }
}

/// Get the log directory path
pub fn get_log_dir_path() -> Option<std::path::PathBuf> {
    crate::obd::vin_cache::get_log_dir_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    // Serialize test access to shared state
    static TEST_LOCK: StdMutex<()> = StdMutex::new(());

    #[test]
    fn test_log_tx() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_tx("0100");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "TX");
        assert_eq!(logs[0].source, "obd");
        assert!(logs[0].message.contains("0100"));
    }

    #[test]
    fn test_log_rx() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_rx("0100", "41 00 FF FF FF FF");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "RX");
        assert_eq!(logs[0].source, "obd");
        assert!(logs[0].message.contains("0100"));
        assert!(logs[0].message.contains("41 00"));
    }

    #[test]
    fn test_log_info() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_info("test_src", "test message");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "INFO");
        assert_eq!(logs[0].source, "test_src");
        assert_eq!(logs[0].message, "test message");
    }

    #[test]
    fn test_log_warn() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_warn("warning_src", "be careful");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "WARN");
        assert_eq!(logs[0].source, "warning_src");
        assert_eq!(logs[0].message, "be careful");
    }

    #[test]
    fn test_log_error() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_error("error_src", "something failed");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "ERROR");
        assert_eq!(logs[0].source, "error_src");
        assert_eq!(logs[0].message, "something failed");
    }

    #[test]
    fn test_log_debug() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_debug("debug_src", "debug info");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "DEBUG");
        assert_eq!(logs[0].source, "debug_src");
        assert_eq!(logs[0].message, "debug info");
    }

    #[test]
    fn test_log_count() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        assert_eq!(log_count(), 0);
        log_info("src1", "msg1");
        assert_eq!(log_count(), 1);
        log_info("src2", "msg2");
        assert_eq!(log_count(), 2);
    }

    #[test]
    fn test_clear_logs() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_info("src", "msg");
        assert_eq!(log_count(), 1);
        clear_logs();
        assert_eq!(log_count(), 0);
    }

    #[test]
    fn test_get_all_logs() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_info("src1", "msg1");
        log_warn("src2", "msg2");
        let logs = get_all_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].level, "INFO");
        assert_eq!(logs[1].level, "WARN");
    }

    #[test]
    fn test_log_entry_has_timestamp() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_info("src", "msg");
        let logs = get_all_logs();
        assert!(!logs[0].timestamp.is_empty());
    }

    #[test]
    fn test_max_entries_capped() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        for i in 0..MAX_ENTRIES + 100 {
            log_info("src", &format!("msg {}", i));
        }
        let logs = get_all_logs();
        assert!(logs.len() <= MAX_ENTRIES);
        assert_eq!(logs.len(), MAX_ENTRIES);
    }

    #[test]
    fn test_get_logs_since() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_info("src1", "msg1");
        log_info("src2", "msg2");
        log_info("src3", "msg3");
        let logs = get_logs_since(1);
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "msg2");
        assert_eq!(logs[1].message, "msg3");
    }

    #[test]
    fn test_get_logs_since_beyond_length() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_logs();
        log_info("src1", "msg1");
        log_info("src2", "msg2");
        let logs = get_logs_since(100);
        assert_eq!(logs.len(), 0);
    }
}
