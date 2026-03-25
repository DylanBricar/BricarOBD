# BricarOBD

Professional OBD-II diagnostic tool built with **Rust** (Tauri 2.x) and **React** (TypeScript).

## Features

- **3.17M operations** from DDT2000 database (PSA, VAG, BMW, Mercedes, Renault, Toyota, Honda, Hyundai/Kia, Fiat, Ford, Mazda, Subaru, Volvo)
- **8,807 DTC codes** with French descriptions + 415 repair tips
- **70 standard OBD-II PIDs** with real-time gauges and charts
- **391 manufacturer-specific DIDs** (PSA 204, Mercedes 30, Hyundai 29, Ford 20, etc.)
- **83 WMI codes** for VIN decoding across 39 manufacturers
- **9 OBD-II protocols** (CAN 11/29-bit 500k/250k, KWP2000, ISO 9141, J1850 PWM/VPW)
- **4 connection strategies** for maximum ELM327 compatibility (genuine + clones)
- **Auto baud rate detection** (9600, 38400, 115200, 230400, 500000)
- **Safety guard** with 11 blocked UDS services — write operations only in Advanced mode
- **Bilingual FR/EN** interface with glassmorphic automotive theme
- **Developer console** with real-time TX/RX serial logs
- **CSV export** for live data recordings and DTC reports
- **Demo mode** (Peugeot 207) for testing without adapter

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 18 + TypeScript + Vite + Tailwind CSS |
| Backend | Rust + Tauri 2.x |
| Database | SQLite (503 MB, pre-built) |
| Serial | serialport crate (ELM327) |
| i18n | react-i18next (FR/EN) |

## Development

```bash
# Install dependencies
pnpm install

# Run in dev mode
cargo tauri dev

# Build for production
cargo tauri build
```

## Requirements

- Node.js 20+
- Rust 1.80+
- pnpm 9+

## License

See [LICENSE](LICENSE).
