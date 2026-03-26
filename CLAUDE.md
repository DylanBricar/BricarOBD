# BricarOBD — Project Rules

## Architecture
- **Frontend**: React 18 + TypeScript + Vite 5 + Tailwind CSS 3 (NOT Next.js — this is a Tauri desktop app)
- **Backend**: Rust + Tauri 2.x + rusqlite + serialport
- **IPC**: Frontend ↔ Backend via `invoke()` from `@tauri-apps/api/core`
- **Database**: Pre-built SQLite 503MB (3.17M operations, 90 vehicle profiles, 4866 ECUs)
- **i18n**: react-i18next (FR/EN), all strings must use `t()` — never hardcode French or English

## Critical Safety Rules
- **ALL OBD commands outside Advanced page MUST be read-only** (Mode 01, 03, 07, 09, 0A, Service 22, 3E)
- **Write services (2E, 2F, 30, 31) are ONLY allowed via `send_raw_command`** which uses `SafetyGuard::check_command_advanced`
- **Always blocked everywhere**: 0x11 (ECUReset), 0x27 (SecurityAccess), 0x34-0x37 (Download/Upload), 0x3D (WriteMemory), 0x28 (CommControl)
- **Mode 04 (Clear DTCs)** requires SafetyGuard check + frontend confirmation dialog
- **Never trust frontend input** for file paths — sanitize filenames, validate paths stay within BricarOBD_Exports/
- **AT commands**: Block ATMA, ATBD, ATBI, ATPP, ATWS

## Connection Modes
- `ConnectionMode::Demo` — all data comes from `DemoConnection` (JS + Rust demo generators)
- `ConnectionMode::Real` — all data comes from the real ELM327 via serial port
- **NEVER mix modes**: if `is_demo()` → use DemoConnection, if real → use `with_real_connection()`
- **NEVER return demo data in real mode** — return empty Vec if real read fails

## File Structure
```
src/                    # React frontend
  pages/                # One file per tab
  components/           # Reusable UI components
  stores/               # State management (connection.ts, vehicle.ts)
  lib/                  # Utils, i18n
  styles/               # Tailwind globals

src-tauri/src/          # Rust backend
  obd/                  # OBD protocol layer (connection, pid, dtc, vin, safety, demo, dev_log)
  commands/             # Tauri command handlers (connection, dashboard, dtc, ecu, database, settings)
  db/                   # SQLite database layer
  models/               # Shared data types
```

## Rust Rules
- Use `unwrap_or_else(|e| e.into_inner())` on Mutex locks — never bare `unwrap()`
- All OBD commands go through `send_command()` which auto-logs TX/RX to dev_log
- PID IDs are `u16` (supports both standard 0x00-0xFF and manufacturer DIDs 0x0100-0xFFFF)
- Use `#[serde(rename_all = "camelCase")]` on structs sent to frontend
- Connection resilience: 4 init strategies, protocol cycling with per-protocol timeouts, retry on PID reads
- All functions in commands/ must log to `dev_log` for the Dev Console

## React Rules
- All user-visible strings use `t("key")` from react-i18next
- Toast notifications: use the shared pattern (green success / red error, X button, auto-dismiss 5s)
- Console logs for Dev Console: use `console.log("[BricarOBD] message")` prefix
- `useEffect` cleanup: always clean up intervals, listeners, subscriptions
- Connection store uses `useEffect` for subscription (not `useState`) to avoid memory leaks
- `pollingModeRef` tracks demo vs real — `changeRefreshRate` respects the current mode

## Performance
- SQLite DB opened once at startup via `setup()` — never re-opened
- PRAGMA: `journal_mode=DELETE`, `synchronous=NORMAL`, `cache_size=-32000`
- PID history buffer capped at 120 entries per PID
- Dev log buffer capped at 5000 entries
- Serial reads in 256-byte chunks (not byte-by-byte)
- 30ms inter-command delay for ELM327 recovery time

## Common Pitfalls to Avoid
1. Blob URL downloads don't work in Tauri webview — use `invoke("save_csv_file")` instead
2. `window.open()` doesn't work in Tauri — use anchor tag with `target="_blank"`
3. Module-level constants with `t()` don't re-render on language switch — move inside component
4. `selectedPids` must reset when `pidData` transitions from 0→N (reconnect scenario)
5. `stopPolling()` must clear `pidData` to avoid stale data display
6. Dates must use `i18n.language` for locale, not hardcoded `"fr-FR"`
