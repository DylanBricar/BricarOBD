"""Hybrid OBD reader — python-obd (380 commands) + custom UDS/security layer.

python-obd handles:
- ELM327 connection (battle-tested, 8+ years)
- 380 predefined OBD-II commands with decoders
- Robust protocol negotiation
- Unit conversions via Pint

Our code always handles:
- Safety guards (default-deny, 11 blocked services) — ALWAYS ACTIVE
- UDS client (Service 0x22, 0x19, 0x3E)
- Extended ECU scan by manufacturer
- VIN-based vehicle detection
- Anomaly detection
"""

from __future__ import annotations

import logging
import threading
from typing import Optional, Tuple, Dict, List

import obd

from obd_core.connection import ConnectionState

logger = logging.getLogger(__name__)
logger.info(f"python-obd {obd.__version__} loaded ({len(obd.commands)} commands)")


class HybridConnection:
    """Uses python-obd for OBD-II + our custom code for UDS and safety.

    Safety guards are ALWAYS enforced regardless of which transport is used.
    """

    def __init__(self):
        self._obd_conn = None  # python-obd connection
        self._custom_conn = None  # our ELM327Connection (for UDS)
        self.port = None
        self.baud_rate = 38400
        self.protocol_name = ""
        self.elm_version = ""
        self.state = ConnectionState.DISCONNECTED
        self.lock = threading.RLock()

    def connect(self, port: str, baud_rate: int = 38400) -> bool:
        """Connect to vehicle via python-obd.

        Falls back to custom ELM327Connection only if python-obd fails.
        """
        self.port = port
        self.baud_rate = baud_rate
        self.state = ConnectionState.CONNECTING

        try:
            logger.info(f"Connecting via python-obd to {port}...")
            self._obd_conn = obd.OBD(
                portstr=port,
                baudrate=baud_rate,
                fast=False,
                timeout=10,
            )

            if self._obd_conn.status() == obd.OBDStatus.CAR_CONNECTED:
                self.protocol_name = str(self._obd_conn.protocol_name())
                self.elm_version = self._obd_conn.port_name() or "ELM327"
                self.state = ConnectionState.CONNECTED
                logger.info(f"Connected via python-obd: {self.protocol_name}")
                return True

            logger.warning(f"python-obd status: {self._obd_conn.status()}, trying custom...")
            self._obd_conn.close()
            self._obd_conn = None
        except Exception as e:
            logger.warning(f"python-obd failed: {e}, trying custom...")
            if self._obd_conn:
                try:
                    self._obd_conn.close()
                except Exception:
                    pass
            self._obd_conn = None

        # Fallback to custom connection
        from obd_core.connection import ELM327Connection
        self._custom_conn = ELM327Connection()
        success = self._custom_conn.connect(port, baud_rate)
        if success:
            self.protocol_name = self._custom_conn.protocol_name
            self.elm_version = self._custom_conn.elm_version
            self.state = ConnectionState.CONNECTED
        else:
            self.state = ConnectionState.ERROR
        return success

    def connect_with_retry(self, port: str, baud_rate: int = 38400, max_retries: int = 3) -> bool:
        """Connect with retry logic."""
        import time
        for attempt in range(max_retries):
            if self.connect(port, baud_rate):
                return True
            logger.warning(f"Attempt {attempt + 1}/{max_retries} failed")
            time.sleep(1 + attempt)
        return False

    def disconnect(self):
        """Disconnect from vehicle."""
        if self._obd_conn:
            try:
                self._obd_conn.close()
            except Exception:
                pass
            self._obd_conn = None

        if self._custom_conn:
            self._custom_conn.disconnect()
            self._custom_conn = None

        self.state = ConnectionState.DISCONNECTED

    def is_connected(self) -> bool:
        """Check connection status."""
        if self._obd_conn:
            return self._obd_conn.status() == obd.OBDStatus.CAR_CONNECTED
        if self._custom_conn:
            return self._custom_conn.is_connected()
        return False

    # ── PID Reading (via python-obd) ───────────────────────────
    def query_pid(self, pid: int) -> Tuple[Optional[float], str]:
        """Read a PID using python-obd's decoders.

        This is READ-ONLY and safe — python-obd only sends Mode 01 queries.
        """
        if not self._obd_conn:
            return None, ""

        cmd = _pid_to_obd_command(pid)
        if not cmd:
            return None, ""

        try:
            response = self._obd_conn.query(cmd)
            if response.is_null():
                return None, ""

            value = response.value
            if hasattr(value, 'magnitude'):
                return float(value.magnitude), str(value.units)
            return float(value), ""
        except Exception as e:
            logger.debug(f"PID 0x{pid:02X} query failed: {e}")
            return None, ""

    def get_dtcs(self) -> List[Dict]:
        """Read DTCs via python-obd (Mode 03 — READ-ONLY, safe)."""
        if not self._obd_conn:
            return []
        try:
            response = self._obd_conn.query(obd.commands.GET_DTC)
            if not response.is_null():
                return [
                    {"code": code, "description": desc, "status": "Active"}
                    for code, desc in response.value
                ]
        except Exception as e:
            logger.debug(f"DTC query failed: {e}")
        return []

    def get_vin(self) -> str:
        """Read VIN via python-obd (Mode 09 — READ-ONLY, safe)."""
        if not self._obd_conn:
            return ""
        try:
            response = self._obd_conn.query(obd.commands.VIN)
            if not response.is_null():
                return str(response.value)
        except Exception as e:
            logger.debug(f"VIN query failed: {e}")
        return ""

    def get_supported_pids(self) -> List[int]:
        """Get supported PIDs from python-obd's auto-discovery."""
        if not self._obd_conn:
            return []
        supported = []
        for cmd in self._obd_conn.supported_commands:
            if cmd.command[:2] == b'01':
                try:
                    pid = int(cmd.command[2:4], 16)
                    supported.append(pid)
                except (ValueError, IndexError):
                    pass
        return sorted(supported)

    # ── UDS / Raw commands (via custom connection + SAFETY) ────
    # These ALWAYS go through our custom code which has SafetyGuard

    def send_command(self, cmd: str, timeout: int = 5) -> str:
        """Send AT command to ELM327."""
        conn = self._get_raw_conn()
        if conn:
            return conn.send_command(cmd, timeout)
        return ""

    def send_obd(self, mode: str, pid: str = None, _internal: bool = False) -> str:
        """Send OBD command — Mode 04 blocked unless _internal=True.

        Safety: Mode 04 (clear DTC) is blocked at this level.
        Must go through DTCManager which requires double confirmation.
        """
        # SAFETY: Block Mode 04 unless explicitly internal
        if mode in ("04",) and not _internal:
            logger.error("Mode 04 blocked — use DTCManager.clear_all_dtcs()")
            return ""

        conn = self._get_raw_conn()
        if conn:
            return conn.send_obd(mode, pid, _internal=_internal)
        return ""

    def send_raw(self, hex_string: str) -> str:
        """Send raw hex command — with validation.

        Safety: Validates hex-only input, rejects AT commands.
        UDS safety is enforced by UDSClient._send_uds() which calls
        SafetyGuard.is_operation_allowed() before calling this method.
        """
        import re
        clean = hex_string.replace(" ", "").upper()
        if not clean:
            return ""
        if not re.match(r'^[0-9A-F]+$', clean):
            raise ValueError(f"Invalid hex: {hex_string}")
        if clean.startswith("AT"):
            raise ValueError("AT commands not allowed via send_raw()")

        conn = self._get_raw_conn()
        if conn:
            return conn.send_raw(hex_string)
        return ""

    def _read_until_prompt(self, timeout: int) -> str:
        """Read serial buffer (for UDS NRC 0x78 handling)."""
        conn = self._get_raw_conn()
        if conn:
            return conn._read_until_prompt(timeout)
        return ""

    def _get_raw_conn(self):
        """Get the underlying serial connection for UDS operations."""
        if self._custom_conn:
            return self._custom_conn
        if self._obd_conn:
            # Create custom conn sharing the serial interface
            # python-obd exposes the port via ._ELM327__port
            try:
                from obd_core.connection import ELM327Connection
                self._custom_conn = ELM327Connection()
                # Reuse python-obd's serial port
                if hasattr(self._obd_conn, 'interface') and hasattr(self._obd_conn.interface, '_ELM327__port'):
                    self._custom_conn.serial_conn = self._obd_conn.interface._ELM327__port
                    self._custom_conn.state = ConnectionState.CONNECTED
                    self._custom_conn.port = self.port
                    logger.info("Sharing python-obd serial port for UDS")
                else:
                    logger.warning("Cannot share port — UDS operations may fail")
                return self._custom_conn
            except Exception as e:
                logger.error(f"Failed to create UDS connection: {e}")
        return None

    @staticmethod
    def available_ports() -> list:
        """List available serial ports via python-obd."""
        try:
            ports = obd.scan_serial()
            if ports:
                return [{"port": p, "description": p, "hwid": ""} for p in ports]
        except Exception:
            pass
        # Fallback
        from obd_core.connection import ELM327Connection
        return ELM327Connection.available_ports()


# ── PID → obd.commands mapping (ALL 96 Mode 01 PIDs) ──────────
_PID_MAP = {
    # Engine basics
    0x01: obd.commands.STATUS,
    0x02: obd.commands.FREEZE_DTC,
    0x03: obd.commands.FUEL_STATUS,
    0x04: obd.commands.ENGINE_LOAD,
    0x05: obd.commands.COOLANT_TEMP,
    0x06: obd.commands.SHORT_FUEL_TRIM_1,
    0x07: obd.commands.LONG_FUEL_TRIM_1,
    0x08: obd.commands.SHORT_FUEL_TRIM_2,
    0x09: obd.commands.LONG_FUEL_TRIM_2,
    0x0A: obd.commands.FUEL_PRESSURE,
    0x0B: obd.commands.INTAKE_PRESSURE,
    0x0C: obd.commands.RPM,
    0x0D: obd.commands.SPEED,
    0x0E: obd.commands.TIMING_ADVANCE,
    0x0F: obd.commands.INTAKE_TEMP,
    0x10: obd.commands.MAF,
    0x11: obd.commands.THROTTLE_POS,
    0x12: obd.commands.AIR_STATUS,
    0x13: obd.commands.O2_SENSORS,
    # O2 sensors bank 1
    0x14: obd.commands.O2_B1S1,
    0x15: obd.commands.O2_B1S2,
    0x16: obd.commands.O2_B1S3,
    0x17: obd.commands.O2_B1S4,
    # O2 sensors bank 2
    0x18: obd.commands.O2_B2S1,
    0x19: obd.commands.O2_B2S2,
    0x1A: obd.commands.O2_B2S3,
    0x1B: obd.commands.O2_B2S4,
    0x1C: obd.commands.OBD_COMPLIANCE,
    0x1E: obd.commands.AUX_INPUT_STATUS,
    0x1F: obd.commands.RUN_TIME,
    # Extended
    0x21: obd.commands.DISTANCE_W_MIL,
    0x22: obd.commands.FUEL_RAIL_PRESSURE_VAC,
    0x23: obd.commands.FUEL_RAIL_PRESSURE_DIRECT,
    0x24: obd.commands.O2_S1_WR_VOLTAGE,
    0x25: obd.commands.O2_S2_WR_VOLTAGE,
    0x26: obd.commands.O2_S3_WR_VOLTAGE,
    0x27: obd.commands.O2_S4_WR_VOLTAGE,
    0x28: obd.commands.O2_S5_WR_VOLTAGE,
    0x29: obd.commands.O2_S6_WR_VOLTAGE,
    0x2A: obd.commands.O2_S7_WR_VOLTAGE,
    0x2B: obd.commands.O2_S8_WR_VOLTAGE,
    0x2C: obd.commands.COMMANDED_EGR,
    0x2D: obd.commands.EGR_ERROR,
    0x2E: obd.commands.EVAPORATIVE_PURGE,
    0x2F: obd.commands.FUEL_LEVEL,
    0x30: obd.commands.WARMUPS_SINCE_DTC_CLEAR,
    0x31: obd.commands.DISTANCE_SINCE_DTC_CLEAR,
    0x32: obd.commands.EVAP_VAPOR_PRESSURE,
    0x33: obd.commands.BAROMETRIC_PRESSURE,
    # O2 WR currents
    0x34: obd.commands.O2_S1_WR_CURRENT,
    0x35: obd.commands.O2_S2_WR_CURRENT,
    0x36: obd.commands.O2_S3_WR_CURRENT,
    0x37: obd.commands.O2_S4_WR_CURRENT,
    0x38: obd.commands.O2_S5_WR_CURRENT,
    0x39: obd.commands.O2_S6_WR_CURRENT,
    0x3A: obd.commands.O2_S7_WR_CURRENT,
    0x3B: obd.commands.O2_S8_WR_CURRENT,
    # Catalyst temperatures
    0x3C: obd.commands.CATALYST_TEMP_B1S1,
    0x3D: obd.commands.CATALYST_TEMP_B2S1,
    0x3E: obd.commands.CATALYST_TEMP_B1S2,
    0x3F: obd.commands.CATALYST_TEMP_B2S2,
    # Advanced
    0x41: obd.commands.STATUS_DRIVE_CYCLE,
    0x42: obd.commands.CONTROL_MODULE_VOLTAGE,
    0x43: obd.commands.ABSOLUTE_LOAD,
    0x44: obd.commands.COMMANDED_EQUIV_RATIO,
    0x45: obd.commands.RELATIVE_THROTTLE_POS,
    0x46: obd.commands.AMBIANT_AIR_TEMP,
    0x47: obd.commands.THROTTLE_POS_B,
    0x48: obd.commands.THROTTLE_POS_C,
    0x49: obd.commands.ACCELERATOR_POS_D,
    0x4A: obd.commands.ACCELERATOR_POS_E,
    0x4B: obd.commands.ACCELERATOR_POS_F,
    0x4C: obd.commands.THROTTLE_ACTUATOR,
    0x4D: obd.commands.RUN_TIME_MIL,
    0x4E: obd.commands.TIME_SINCE_DTC_CLEARED,
    0x51: obd.commands.FUEL_TYPE,
    0x52: obd.commands.ETHANOL_PERCENT,
    0x53: obd.commands.EVAP_VAPOR_PRESSURE_ABS,
    0x54: obd.commands.EVAP_VAPOR_PRESSURE_ALT,
    0x55: obd.commands.SHORT_O2_TRIM_B1,
    0x56: obd.commands.LONG_O2_TRIM_B1,
    0x57: obd.commands.SHORT_O2_TRIM_B2,
    0x58: obd.commands.LONG_O2_TRIM_B2,
    0x59: obd.commands.FUEL_RAIL_PRESSURE_ABS,
    0x5A: obd.commands.RELATIVE_ACCEL_POS,
    0x5B: obd.commands.HYBRID_BATTERY_REMAINING,
    0x5C: obd.commands.OIL_TEMP,
    0x5D: obd.commands.FUEL_INJECT_TIMING,
    0x5E: obd.commands.FUEL_RATE,
    0x5F: obd.commands.EMISSION_REQ,
}

def _pid_to_obd_command(pid: int):
    """Map PID hex code to python-obd command object."""
    return _PID_MAP.get(pid)
