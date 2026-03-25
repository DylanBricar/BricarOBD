---
description: Rust backend rules for BricarOBD Tauri app — OBD safety, performance, serial port handling
globs: src-tauri/**/*.rs
---

# Rust Backend Rules

## Safety — Non-Negotiable
- NEVER send write commands (2E, 2F, 30, 31, 34-37, 3D, 11, 27, 28) outside `send_raw_command`
- ALWAYS check `is_demo()` before accessing the real serial connection
- ALWAYS use `with_real_connection()` to access the Elm327Connection — never access the static directly
- ALWAYS use `SafetyGuard::check_command()` for normal commands, `check_command_advanced()` only for Advanced page
- NEVER return DemoConnection data when `is_demo()` returns false

## Error Handling
- Use `unwrap_or_else(|e| e.into_inner())` on all Mutex locks
- Propagate errors with `?` and `map_err()` — never use bare `unwrap()` or `expect()` in commands
- Log errors to `dev_log::log_error()` before returning Err
- Serial port errors should `continue` (skip PID) not abort the entire scan

## Performance
- Serial reads: 64-byte buffer chunks, not byte-by-byte
- PID history: max 120 entries per PID (VecDeque preferred over Vec for O(1) pop_front)
- Database queries: always use LIMIT parameter
- Avoid holding Mutex locks across `.await` points
- Use `tokio::task::spawn_blocking()` for serial I/O (blocking calls)

## Serialport Gotchas
- Always set 8N1 (8 data bits, no parity, 1 stop bit) for OBD
- Timeout: 5s default, 10-12s for KWP 5-baud init
- Flush buffer before each init strategy
- 30ms inter-command delay (ELM327 recovery)
- Filter response noise: SEARCHING, BUS INIT, NO DATA, CAN ERROR, BUFFER FULL, BUS BUSY

## Logging
- Every command must log to `dev_log` with appropriate source name
- TX/RX: use `dev_log::log_tx()` and `dev_log::log_rx()`
- Use `log_info` for successful operations, `log_warn` for fallbacks, `log_error` for failures
- Never log sensitive data (VIN is OK, but not security keys)

## Data Types
- PID ID: `u16` (supports 0x00-0xFFFF)
- Structs for frontend: use `#[serde(rename_all = "camelCase")]`
- Optional fields: use `#[serde(skip_serializing_if = "Option::is_none")]`
- DTC status enum: lowercase serialization `#[serde(rename_all = "lowercase")]`

## Top 15 Common Rust Mistakes to Avoid
1. Bare `unwrap()` on Mutex — causes panic on poison
2. Holding Mutex across async `.await` — causes deadlock
3. `Vec::remove(0)` in a hot loop — use VecDeque
4. Not filtering ELM327 noise from responses
5. Single byte reads on serial port — use 64-byte chunks
6. Not resetting CAN headers after ECU-specific commands (`ATSH7DF`)
7. Using `?` in protocol cycling loop — kills the loop on first error
8. Timeout too short for KWP 5-baud (needs 10s+)
9. Not flushing serial buffer before init
10. Returning demo data in real mode
11. Not sanitizing user-provided filenames for file I/O
12. Not validating path stays within allowed directory
13. Sending write commands without SafetyGuard check
14. Not logging TX/RX for debugging
15. Creating new DemoConnection on every call — use static Mutex singleton
