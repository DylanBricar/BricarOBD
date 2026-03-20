"""Hybrid OBD reader — uses python-obd library when available, falls back to custom implementation.

python-obd (pip install obd) provides:
- Battle-tested ELM327 connection (8+ years, thousands of users)
- 380 predefined commands with proper decoders
- Robust protocol negotiation
- Unit conversions via Pint

Our custom code adds:
- UDS client (Service 0x22, 0x19, 0x3E)
- Extended ECU scan by manufacturer
- Safety guards (default-deny, blocked services)
- VIN-based vehicle detection
- Anomaly detection
"""

from __future__ import annotations

import logging
from typing import Optional, Tuple, Dict, List

logger = logging.getLogger(__name__)

# Try to import python-obd
try:
    import obd
    OBD_AVAILABLE = True
    logger.info(f"python-obd {obd.__version__} available ({len(obd.commands)} commands)")
except ImportError:
    OBD_AVAILABLE = False
    logger.info("python-obd not installed — using custom OBD reader")


class HybridConnection:
    """Wraps python-obd connection with fallback to our custom ELM327Connection.

    Priority: python-obd for standard OBD-II, custom code for UDS/extended.
    """

    def __init__(self):
        self._obd_conn = None  # python-obd connection
        self._custom_conn = None  # our ELM327Connection
        self.port = None
        self.baud_rate = 38400
        self.protocol_name = ""
        self.elm_version = ""
        self._using_obd_lib = False

    def connect(self, port: str, baud_rate: int = 38400) -> bool:
        """Connect using python-obd if available, fallback to custom.

        Args:
            port: Serial port
            baud_rate: Baud rate

        Returns:
            True if connected
        """
        self.port = port
        self.baud_rate = baud_rate

        if OBD_AVAILABLE and port != "DEMO":
            try:
                logger.info(f"Connecting via python-obd to {port}...")
                self._obd_conn = obd.OBD(
                    portstr=port,
                    baudrate=baud_rate,
                    fast=False,
                    timeout=10,
                )

                if self._obd_conn.status() == obd.OBDStatus.CAR_CONNECTED:
                    self._using_obd_lib = True
                    self.protocol_name = str(self._obd_conn.protocol_name())
                    self.elm_version = self._obd_conn.port_name() or "ELM327"
                    logger.info(f"Connected via python-obd: {self.protocol_name}")
                    return True
                else:
                    logger.warning(f"python-obd status: {self._obd_conn.status()}, falling back to custom")
                    self._obd_conn.close()
                    self._obd_conn = None
            except Exception as e:
                logger.warning(f"python-obd failed: {e}, falling back to custom")
                self._obd_conn = None

        # Fallback to custom connection
        from obd_core.connection import ELM327Connection
        self._custom_conn = ELM327Connection()
        success = self._custom_conn.connect(port, baud_rate)
        if success:
            self._using_obd_lib = False
            self.protocol_name = self._custom_conn.protocol_name
            self.elm_version = self._custom_conn.elm_version
        return success

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

        self._using_obd_lib = False

    def is_connected(self) -> bool:
        """Check if connected."""
        if self._using_obd_lib and self._obd_conn:
            return self._obd_conn.status() == obd.OBDStatus.CAR_CONNECTED
        if self._custom_conn:
            return self._custom_conn.is_connected()
        return False

    def query_pid(self, pid: int) -> Tuple[Optional[float], str]:
        """Read a PID using python-obd if available.

        Args:
            pid: OBD-II PID code (e.g., 0x0C for RPM)

        Returns:
            (value, unit) or (None, "")
        """
        if self._using_obd_lib and self._obd_conn:
            return self._query_pid_obd_lib(pid)
        return None, ""  # Custom reader handles this separately

    def _query_pid_obd_lib(self, pid: int) -> Tuple[Optional[float], str]:
        """Query PID via python-obd library."""
        # Map PID code to obd command
        cmd = _pid_to_obd_command(pid)
        if not cmd:
            return None, ""

        try:
            response = self._obd_conn.query(cmd)
            if response.is_null():
                return None, ""

            value = response.value
            # Handle Pint quantities (python-obd returns unit-bearing values)
            if hasattr(value, 'magnitude'):
                return float(value.magnitude), str(value.units)
            return float(value), ""
        except Exception as e:
            logger.debug(f"PID 0x{pid:02X} query failed: {e}")
            return None, ""

    def get_dtcs(self) -> List[Dict]:
        """Read DTCs via python-obd if available."""
        if self._using_obd_lib and self._obd_conn:
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
        """Read VIN via python-obd if available."""
        if self._using_obd_lib and self._obd_conn:
            try:
                response = self._obd_conn.query(obd.commands.VIN)
                if not response.is_null():
                    return str(response.value)
            except Exception as e:
                logger.debug(f"VIN query failed: {e}")
        return ""

    def get_supported_pids(self) -> List[int]:
        """Get supported PIDs via python-obd."""
        if self._using_obd_lib and self._obd_conn:
            supported = []
            for cmd in self._obd_conn.supported_commands:
                if cmd.command[:2] == b'01':
                    try:
                        pid = int(cmd.command[2:4], 16)
                        supported.append(pid)
                    except (ValueError, IndexError):
                        pass
            return sorted(supported)
        return []

    # Pass-through methods for UDS (always uses custom connection)
    def send_command(self, cmd: str, timeout: int = 5) -> str:
        """Send raw AT/OBD command (for UDS operations)."""
        if self._custom_conn:
            return self._custom_conn.send_command(cmd, timeout)
        # If using obd lib, we need to create a custom conn for UDS
        if self._using_obd_lib:
            self._ensure_custom_conn()
            return self._custom_conn.send_command(cmd, timeout)
        return ""

    def send_obd(self, mode: str, pid: str = None, _internal: bool = False) -> str:
        """Send OBD command (pass-through to custom connection)."""
        self._ensure_custom_conn()
        return self._custom_conn.send_obd(mode, pid, _internal=_internal)

    def send_raw(self, hex_string: str) -> str:
        """Send raw hex for UDS (pass-through to custom connection)."""
        self._ensure_custom_conn()
        return self._custom_conn.send_raw(hex_string)

    def _read_until_prompt(self, timeout: int) -> str:
        """Read serial until prompt (for UDS NRC 0x78 handling)."""
        if self._custom_conn:
            return self._custom_conn._read_until_prompt(timeout)
        return ""

    def _ensure_custom_conn(self):
        """Create custom connection alongside obd lib for UDS access."""
        if not self._custom_conn and self._using_obd_lib and self.port:
            from obd_core.connection import ELM327Connection
            self._custom_conn = ELM327Connection()
            # Share the same serial port — this is tricky
            # Better approach: use the obd lib's underlying interface
            logger.warning("UDS operations require custom connection — sharing port may cause issues")

    @property
    def using_obd_library(self) -> bool:
        """Whether python-obd is being used for communication."""
        return self._using_obd_lib

    @property
    def lock(self):
        """Thread lock for serial access."""
        if self._custom_conn:
            return self._custom_conn.lock
        import threading
        return threading.RLock()

    @property
    def state(self):
        """Connection state."""
        if self._custom_conn:
            return self._custom_conn.state
        from obd_core.connection import ConnectionState
        if self.is_connected():
            return ConnectionState.CONNECTED
        return ConnectionState.DISCONNECTED

    @staticmethod
    def available_ports() -> list:
        """List available serial ports."""
        if OBD_AVAILABLE:
            try:
                ports = obd.scan_serial()
                return [{"port": p, "description": p, "hwid": ""} for p in ports]
            except Exception:
                pass
        from obd_core.connection import ELM327Connection
        return ELM327Connection.available_ports()


# ── PID to obd.commands mapping ────────────────────────────────
def _pid_to_obd_command(pid: int):
    """Map a PID hex code to an obd library command."""
    if not OBD_AVAILABLE:
        return None

    PID_MAP = {
        0x04: obd.commands.ENGINE_LOAD,
        0x05: obd.commands.COOLANT_TEMP,
        0x06: obd.commands.SHORT_FUEL_TRIM_1,
        0x07: obd.commands.LONG_FUEL_TRIM_1,
        0x0B: obd.commands.INTAKE_PRESSURE,
        0x0C: obd.commands.RPM,
        0x0D: obd.commands.SPEED,
        0x0E: obd.commands.TIMING_ADVANCE,
        0x0F: obd.commands.INTAKE_TEMP,
        0x10: obd.commands.MAF,
        0x11: obd.commands.THROTTLE_POS,
        0x1F: obd.commands.RUN_TIME,
        0x21: obd.commands.DISTANCE_W_MIL,
        0x2F: obd.commands.FUEL_LEVEL,
        0x31: obd.commands.DISTANCE_SINCE_DTC_CLEAR,
        0x33: obd.commands.BAROMETRIC_PRESSURE,
        0x42: obd.commands.CONTROL_MODULE_VOLTAGE,
        0x46: obd.commands.AMBIANT_AIR_TEMP,
        0x5C: obd.commands.OIL_TEMP,
        0x5E: obd.commands.FUEL_RATE,
    }

    return PID_MAP.get(pid)
