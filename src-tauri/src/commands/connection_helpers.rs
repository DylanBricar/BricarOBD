use std::sync::atomic::{AtomicBool, Ordering};

static OBD_BUSY: AtomicBool = AtomicBool::new(false);

/// Set OBD busy state
pub fn set_obd_busy(busy: bool) {
    OBD_BUSY.store(busy, Ordering::SeqCst);
}

/// Check if OBD is busy
pub fn is_obd_busy() -> bool {
    OBD_BUSY.load(Ordering::SeqCst)
}

/// Guard to manage OBD busy state with RAII semantics
pub struct OBDBusyGuard;

impl OBDBusyGuard {
    /// Try to acquire the OBD lock, fail immediately if already busy
    pub fn try_acquire() -> Result<Self, String> {
        if OBD_BUSY.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err("OBD is busy with another operation".to_string());
        }
        Ok(OBDBusyGuard)
    }

    /// Acquire the OBD lock with timeout (capped at 10s), retrying every 100ms
    pub fn acquire_with_wait(timeout_secs: u64) -> Result<Self, String> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs.min(10));

        loop {
            if OBD_BUSY.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                crate::obd::dev_log::log_debug("connection", "OBD lock acquired");
                return Ok(OBDBusyGuard);
            }

            if start.elapsed() > timeout {
                crate::obd::dev_log::log_warn("connection", &format!("OBD lock timeout after {} seconds", timeout_secs));
                return Err(format!("OBD lock timeout after {} seconds", timeout_secs));
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

impl Drop for OBDBusyGuard {
    fn drop(&mut self) {
        // Use try_lock to avoid deadlock if CONNECTION Mutex is already held (e.g. panic unwind)
        if let Ok(mut guard) = super::connection::CONNECTION.try_lock() {
            if let super::connection::ConnectionMode::Real(ref mut conn) = *guard {
                let _ = conn.reset_headers();
            }
        }
        set_obd_busy(false);
        crate::obd::dev_log::log_debug("connection", "OBD lock released");
    }
}

/// Check if an IP address is private/link-local (safe for local ELM327 adapters)
pub fn is_private_ip(host: &str) -> bool {
    if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
        ip.is_private() || ip.is_link_local() || ip.is_loopback()
    } else {
        // Allow "localhost" as special case
        host == "localhost"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static BUSY_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_obd_busy_guard_try_acquire() {
        let _lock = BUSY_TEST_LOCK.lock().unwrap();
        set_obd_busy(false);
        let guard = OBDBusyGuard::try_acquire();
        assert!(guard.is_ok());
        assert!(is_obd_busy());
        drop(guard);
        assert!(!is_obd_busy());
    }

    #[test]
    fn test_obd_busy_guard_try_acquire_fails_when_busy() {
        let _lock = BUSY_TEST_LOCK.lock().unwrap();
        set_obd_busy(false);
        let _guard1 = OBDBusyGuard::try_acquire().unwrap();
        assert!(is_obd_busy());
        let guard2 = OBDBusyGuard::try_acquire();
        assert!(guard2.is_err());
        assert!(is_obd_busy());
    }

    #[test]
    fn test_obd_busy_guard_auto_release_on_drop() {
        let _lock = BUSY_TEST_LOCK.lock().unwrap();
        set_obd_busy(false);
        {
            let _guard = OBDBusyGuard::try_acquire().unwrap();
            assert!(is_obd_busy());
        }
        assert!(!is_obd_busy());
    }

    #[test]
    fn test_is_private_ip_localhost() {
        assert!(is_private_ip("localhost"));
    }

    #[test]
    fn test_is_private_ip_loopback() {
        assert!(is_private_ip("127.0.0.1"));
    }

    #[test]
    fn test_is_private_ip_private_range_192() {
        assert!(is_private_ip("192.168.1.1"));
    }

    #[test]
    fn test_is_private_ip_private_range_10() {
        assert!(is_private_ip("10.0.0.1"));
    }

    #[test]
    fn test_is_private_ip_private_range_172() {
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("172.31.255.255"));
    }

    #[test]
    fn test_is_private_ip_link_local() {
        assert!(is_private_ip("169.254.1.1"));
    }

    #[test]
    fn test_is_private_ip_public() {
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("1.1.1.1"));
    }
}
