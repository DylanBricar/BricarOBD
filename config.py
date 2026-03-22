"""Application configuration and constants."""

import os
from pathlib import Path

# Application
APP_NAME = "BricarOBD"
APP_VERSION = "1.0.0"
APP_WINDOW_SIZE = "1400x860"
APP_MIN_SIZE = (1100, 700)

# Paths
APP_DIR = Path(__file__).parent
DATA_DIR = APP_DIR / "data"
SESSIONS_DIR = DATA_DIR / "sessions"
LOG_DIR = APP_DIR / "logs"

# Ensure directories exist
SESSIONS_DIR.mkdir(parents=True, exist_ok=True)
LOG_DIR.mkdir(parents=True, exist_ok=True)

# Connection defaults
DEFAULT_BAUD_RATE = 38400
SUPPORTED_BAUD_RATES = [9600, 38400, 115200, 230400, 500000]
CONNECTION_TIMEOUT = 10  # seconds
COMMAND_TIMEOUT = 5  # seconds

# ELM327
ELM327_INIT_COMMANDS = [
    "ATZ",    # Reset
    "ATE0",   # Echo off
    "ATL0",   # Linefeeds off
    "ATS1",   # Spaces on
    "ATH0",   # Headers off
    "ATSP0",  # Auto-detect protocol
]

# Safety
MAX_FAILED_SECURITY_ATTEMPTS = 3
DTC_CLEAR_COOLDOWN_SECONDS = 5
REQUIRE_DOUBLE_CONFIRMATION_FOR = [
    "clear_dtc",
    "ecu_reset",
]

# Operations that are BLOCKED (never allowed)
BLOCKED_UDS_SERVICES = [
    0x2E,  # WriteDataByIdentifier
    0x2F,  # InputOutputControlByIdentifier
    0x31,  # RoutineControl
    0x3D,  # WriteMemoryByAddress
    0x34,  # RequestDownload
    0x35,  # RequestUpload
    0x36,  # TransferData
    0x37,  # RequestTransferExit
    0x11,  # ECUReset
    0x27,  # SecurityAccess
    0x28,  # CommunicationControl
]

# Safe UDS services (allowed)
SAFE_UDS_SERVICES = [
    0x10,  # DiagnosticSessionControl
    0x19,  # ReadDTCInformation
    0x22,  # ReadDataByIdentifier
    0x3E,  # TesterPresent
]

# Allowed with confirmation
CONFIRMED_UDS_SERVICES = [
    0x14,  # ClearDiagnosticInformation
]

# OBD-II Standard CAN IDs
OBD_BROADCAST_ID = 0x7DF
OBD_ECU_REQUEST_IDS = {
    "Engine (ECM)": 0x7E0,
    "Transmission (TCM)": 0x7E1,
    "ABS/ESP": 0x7E2,
    "Airbag (SRS)": 0x7E3,
    "Body Control (BCM)": 0x7E4,
    "Instrument Cluster": 0x7E5,
    "Steering/ADAS": 0x7E6,
}

# Session export format
SESSION_FILE_EXTENSION = ".json"
SESSION_TEXT_EXTENSION = ".txt"

# Web search
DTC_SEARCH_URL = "https://www.google.com/search?q={code}+{vehicle}+diagnostic"

# Refresh rates (ms)
LIVE_DATA_REFRESH_MS = 500
DASHBOARD_REFRESH_MS = 1000
CONNECTION_CHECK_MS = 3000

# Live graphs
GRAPH_HISTORY_SAMPLES = 60  # Number of samples to display in graphs

# CSV recording
CSV_DIR = DATA_DIR / "recordings"
CSV_DIR.mkdir(parents=True, exist_ok=True)
