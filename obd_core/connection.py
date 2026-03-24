"""ELM327 connection manager with auto-detection and safety."""

from __future__ import annotations

import serial
import serial.tools.list_ports
import threading
import logging
import time
import re
from enum import Enum

logger = logging.getLogger(__name__)

MAX_RESPONSE_SIZE = 4096

PROTOCOL_NAMES = {
    "0": "Auto", "1": "SAE J1850 PWM", "2": "SAE J1850 VPW",
    "3": "ISO 9141-2", "4": "ISO 14230-4 KWP", "5": "ISO 15765-4 CAN 11-bit 500k",
    "6": "ISO 15765-4 CAN 29-bit 500k", "7": "ISO 15765-4 CAN 11-bit 250k",
    "8": "ISO 15765-4 CAN 29-bit 250k", "9": "ISO 15765-4 CAN 11-bit 500k",
    "A": "SAE J1939 CAN 29-bit 250k",
}


class ConnectionState(Enum):
    """ELM327 connection states."""
    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    CONNECTED = "connected"
    ERROR = "error"


class ELM327Connection:
    """Manages communication with ELM327 OBD adapter."""

    def __init__(self):
        """Initialize connection parameters."""
        self.port = None
        self.baud_rate = 38400
        self.serial_conn = None
        self.state = ConnectionState.DISCONNECTED
        self.protocol_name = ""
        self.elm_version = ""
        self.lock = threading.RLock()
        self._last_command_time = 0
        self.P3_DELAY = 0.055  # 55ms minimum between commands (ISO standard)

    @staticmethod
    def available_ports() -> list[dict]:
        """Get list of available serial ports."""
        ports = []
        for port_info in serial.tools.list_ports.comports():
            ports.append({
                "port": port_info.device,
                "description": port_info.description,
                "hwid": port_info.hwid
            })
        return ports

    def connect(self, port: str, baud_rate: int = 38400, protocol: str = None) -> bool:
        """
        Connect to ELM327 device.

        Args:
            port: Serial port name (e.g., 'COM3' or '/dev/ttyUSB0')
            baud_rate: Baud rate (default 38400)
            protocol: Force protocol number (e.g., "6" for CAN 11/500).
                      If set, skips ATZ reset (faster reconnect for clone ELM327).

        Returns:
            True if successful, False otherwise
        """
        with self.lock:
            try:
                self.state = ConnectionState.CONNECTING
                self.serial_conn = serial.Serial(port, baud_rate, timeout=1)
                self.port = port
                self.baud_rate = baud_rate

                time.sleep(0.5)

                # Flush any garbage and wake up ELM327
                self.serial_conn.reset_input_buffer()
                self.serial_conn.reset_output_buffer()
                self.serial_conn.write(b"\r\r")
                time.sleep(0.5)
                self.serial_conn.reset_input_buffer()

                if protocol:
                    # Fast reconnect: skip ATZ, just configure and force protocol
                    init_commands = [
                        ("ATE0", "echo off"),
                        ("ATL0", "linefeeds off"),
                        ("ATS1", "spaces on"),
                        ("ATH0", "headers off"),
                        (f"ATSP{protocol}", f"set protocol {protocol}"),
                    ]
                    logger.info(f"Fast reconnect with protocol {protocol}")
                else:
                    init_commands = [
                        ("ATZ", "reset"),
                        ("ATE0", "echo off"),
                        ("ATL0", "linefeeds off"),
                        ("ATS1", "spaces on"),
                        ("ATH0", "headers off"),
                        ("ATSP0", "protocol auto"),
                    ]

                for cmd, desc in init_commands:
                    response = self.send_command(cmd, timeout=3)
                    if cmd == "ATZ":
                        if "ELM" not in response.upper():
                            logger.warning(f"ATZ unexpected: {response}")
                        time.sleep(1)  # Extra delay after reset for clone ELM327
                    elif "OK" not in response and response.strip():
                        logger.warning(f"Command {desc} ({cmd}) unexpected response: {response}")

                self.elm_version = self._parse_elm_version(self.send_command("ATI", timeout=2))
                self.protocol_name = self._detect_protocol()

                self.state = ConnectionState.CONNECTED
                logger.info(f"Connected to ELM327 at {port} ({self.elm_version})")
                return True

            except serial.SerialException as e:
                self.state = ConnectionState.ERROR
                logger.error(f"Serial connection failed: {e}")
                return False
            except Exception as e:
                self.state = ConnectionState.ERROR
                logger.error(f"Connection error: {e}")
                return False

    def connect_with_retry(self, port: str, baud_rate: int = 38400, max_retries: int = 3) -> bool:
        """Connect with automatic retry on failure.

        Args:
            port: Serial port
            baud_rate: Baud rate
            max_retries: Maximum connection attempts

        Returns:
            True if connected
        """
        for attempt in range(max_retries):
            if self.connect(port, baud_rate):
                return True
            logger.warning(f"Connection attempt {attempt + 1}/{max_retries} failed, retrying...")
            time.sleep(1 + attempt)  # Exponential backoff
        return False

    def disconnect(self) -> None:
        """Disconnect from ELM327 device."""
        with self.lock:
            try:
                if self.serial_conn and self.serial_conn.is_open:
                    self.serial_conn.close()
            except Exception as e:
                logger.error(f"Error closing connection: {e}")
            finally:
                self.serial_conn = None
                self.state = ConnectionState.DISCONNECTED
                logger.info("Disconnected from ELM327")

    def send_command(self, cmd: str, timeout: int = 5) -> str:
        """
        Send AT command and get response.

        Args:
            cmd: Command to send
            timeout: Response timeout in seconds

        Returns:
            Response string
        """
        with self.lock:
            if not self.is_connected():
                return ""

            try:
                # P3 timing: wait minimum delay between commands
                elapsed = time.time() - self._last_command_time
                if elapsed < self.P3_DELAY:
                    time.sleep(self.P3_DELAY - elapsed)

                self.serial_conn.write((cmd + "\r").encode())
                response = self._read_until_prompt(timeout)
                self._last_command_time = time.time()
                return response.strip()
            except serial.SerialTimeoutException:
                logger.error(f"Timeout on command: {cmd}")
                return ""
            except Exception as e:
                logger.error(f"Error sending command {cmd}: {e}")
                return ""

    def send_obd(self, mode: str, pid: str = None, _internal: bool = False) -> str:
        """Send OBD-II command.

        Args:
            mode: OBD mode (e.g., "01")
            pid: Parameter ID (e.g., "0D")
            _internal: If True, allow write modes (used by DTCManager only)

        Returns:
            Response string
        """
        # Block write-capable modes unless explicitly allowed
        if mode in ("04",) and not _internal:
            logger.error("Mode 04 (clear DTC) blocked - use DTCManager.clear_all_dtcs()")
            return ""

        if pid:
            cmd = f"{mode}{pid}"
        else:
            cmd = mode

        # DTC modes need longer timeout
        timeout = 5 if mode in ("03", "07", "0A") else 2

        from utils.dev_console import log_obd_command
        log_obd_command("TX", cmd)
        response = self.send_command(cmd, timeout=timeout)
        log_obd_command("RX", cmd, response.strip()[:80] if response else "(no response)")
        return response

    def send_raw(self, hex_string: str) -> str:
        """Send raw hex command for UDS.

        Args:
            hex_string: Hex string (only 0-9, A-F allowed, no AT commands)

        Returns:
            Response string

        Raises:
            ValueError: If hex_string contains invalid characters
        """
        clean = hex_string.replace(" ", "").upper()
        if not clean:
            return ""
        if not re.match(r'^[0-9A-F]+$', clean):
            raise ValueError(f"Invalid hex string: {hex_string}")
        if clean.startswith("AT"):
            raise ValueError("AT commands not allowed via send_raw()")
        if len(clean) > 16:
            logger.warning(f"Long hex command ({len(clean)} chars): {clean[:8]}...")

        from utils.dev_console import log_obd_command
        log_obd_command("TX", f"RAW:{clean}")
        response = self.send_command(clean, timeout=5)
        log_obd_command("RX", f"RAW:{clean}", response.strip()[:80] if response else "(no response)")
        return response

    def is_connected(self) -> bool:
        """Check if currently connected."""
        return self.state == ConnectionState.CONNECTED and \
               self.serial_conn is not None and \
               self.serial_conn.is_open

    def _read_until_prompt(self, timeout: int) -> str:
        """
        Read serial data until prompt character is received.

        Args:
            timeout: Timeout in seconds

        Returns:
            Response string
        """
        response = ""
        start_time = time.time()

        while time.time() - start_time < timeout:
            try:
                if self.serial_conn.in_waiting > 0:
                    char = self.serial_conn.read(1).decode('utf-8', errors='ignore')
                    response += char
                    if char == '>':
                        return response[:-1]
                    if len(response) > MAX_RESPONSE_SIZE:
                        logger.warning("Response exceeded max size, truncating")
                        return response
                time.sleep(0.01)
            except (serial.SerialException, OSError) as e:
                logger.error(f"Serial error during read: {e}")
                self.state = ConnectionState.ERROR
                return response

        return response

    def _parse_elm_version(self, response: str) -> str:
        """
        Extract ELM327 version from ATI response.

        Args:
            response: Response from ATI command

        Returns:
            Version string
        """
        lines = response.split('\n')
        for line in lines:
            if 'ELM327' in line or 'v' in line.lower():
                return line.strip()
        return "Unknown"

    def _detect_protocol(self) -> str:
        """
        Detect and return active protocol.

        Returns:
            Protocol name
        """
        try:
            response = self.send_command("ATDPN", timeout=2)
            protocol_num = response.replace("DPN", "").strip()
            # Strip auto-detect prefix (e.g., "A6" -> "6")
            protocol_num = protocol_num.lstrip('A')
            return PROTOCOL_NAMES.get(protocol_num, f"Unknown ({protocol_num})")
        except Exception as e:
            logger.error(f"Protocol detection failed: {e}")
            return "Unknown"
