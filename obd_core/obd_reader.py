"""OBD-II standard mode reader (Modes 01-0A)."""

import logging
from typing import Optional, Tuple, Dict, List
from obd_core.connection import ELM327Connection
from obd_core.pid_definitions import decode_pid, STANDARD_PIDS
from data.dtc_descriptions import decode_dtc_bytes, get_dtc_description

logger = logging.getLogger(__name__)


class OBDReader:
    """Reads OBD-II data from vehicle through ELM327 adapter."""

    def __init__(self, connection: ELM327Connection):
        """Initialize OBD reader with connection reference.

        Args:
            connection: ELM327Connection instance
        """
        self.connection = connection
        self._supported_pids = []

    def _parse_obd_response(self, response: str, expected_mode_response: str) -> List[str]:
        """Parse OBD response, handling multi-line/multi-ECU replies.

        Args:
            response: Raw response from ELM327
            expected_mode_response: Expected response prefix (e.g., "41" for Mode 01)

        Returns:
            List of data hex strings (one per responding ECU), without mode/PID prefix
        """
        if not response or "NO DATA" in response or "UNABLE" in response or "ERROR" in response:
            return []

        results = []
        lines = [line.strip() for line in response.replace('\r', '\n').split('\n') if line.strip()]

        for line in lines:
            clean = line.replace(" ", "")
            if clean.startswith(expected_mode_response):
                results.append(clean)

        return results

    def get_supported_pids(self) -> List[int]:
        """Query supported PIDs (0x00, 0x20, 0x40, etc.).

        Parses bit field responses to discover which PIDs are supported.

        Returns:
            List of supported PID codes
        """
        supported = []
        pid_query_sequence = [0x00, 0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0]

        for pid in pid_query_sequence:
            response = self.connection.send_obd("01", f"{pid:02X}")
            parsed = self._parse_obd_response(response, "41")

            if not parsed:
                continue

            data_line = parsed[0]
            data_hex = data_line[4:]

            if len(data_hex) < 8:
                continue

            try:
                value = int(data_hex[:8], 16)
                for bit in range(32):
                    if value & (1 << (31 - bit)):
                        pid_code = pid + bit + 1
                        if pid_code not in supported:
                            supported.append(pid_code)
            except ValueError:
                continue

        return sorted(supported)

    def discover_supported_pids(self) -> List[int]:
        """Discover which PIDs are supported by the connected vehicle.
        Caches results for future queries.

        Returns:
            List of supported PID codes
        """
        if not self._supported_pids:
            self._supported_pids = self.get_supported_pids()
            logger.info(f"Discovered {len(self._supported_pids)} supported PIDs")
        return self._supported_pids

    def is_pid_supported(self, pid: int) -> bool:
        """Check if a PID is supported by the vehicle.

        Args:
            pid: PID code to check

        Returns:
            True if supported
        """
        supported = self.discover_supported_pids()
        return pid in supported

    def reset_pid_cache(self):
        """Reset the supported PID cache (e.g., after reconnection)."""
        self._supported_pids = []

    def read_pid(self, pid: int) -> Tuple[Optional[float], str]:
        """Send Mode 01 command and read a single PID.

        Args:
            pid: PID code to read

        Returns:
            Tuple of (value, unit) or (None, "") on error
        """
        if pid not in STANDARD_PIDS:
            return None, ""

        # Skip unsupported PIDs silently (best practice from python-OBD)
        if self._supported_pids and pid not in self._supported_pids:
            return None, ""

        response = self.connection.send_obd("01", f"{pid:02X}")
        parsed = self._parse_obd_response(response, "41")

        if not parsed:
            return None, ""

        try:
            data_line = parsed[0]
            data_hex = data_line[4:]
            data_bytes = bytes.fromhex(data_hex)
            result = decode_pid(pid, data_bytes)
            if result:
                return result
        except (ValueError, IndexError) as e:
            logger.error(f"Error parsing PID {pid:02X}: {e}")

        return None, ""

    def read_multiple_pids(self, pids: List[int]) -> Dict[int, Tuple[Optional[float], str]]:
        """Read multiple PIDs in sequence.

        Args:
            pids: List of PID codes to read

        Returns:
            Dictionary mapping PID to (value, unit) tuple
        """
        results = {}
        for pid in pids:
            results[pid] = self.read_pid(pid)
        return results

    def read_dtcs(self) -> List[Dict]:
        """Read Diagnostic Trouble Codes (Mode 03).

        Returns:
            List of {code, status, description} dicts
        """
        response = self.connection.send_obd("03")
        return self._parse_dtc_response(response, "03")

    def read_pending_dtcs(self) -> List[Dict]:
        """Read Pending DTCs (Mode 07).

        Returns:
            List of {code, status, description} dicts
        """
        response = self.connection.send_obd("07")
        return self._parse_dtc_response(response, "07")

    def read_permanent_dtcs(self) -> List[Dict]:
        """Read Permanent DTCs (Mode 0A).

        Returns:
            List of {code, status, description} dicts
        """
        response = self.connection.send_obd("0A")
        return self._parse_dtc_response(response, "0A")

    def read_freeze_frame(self, pid: int) -> Tuple[Optional[float], str]:
        """Read freeze frame data for a specific PID (Mode 02).

        Args:
            pid: PID code to read from freeze frame

        Returns:
            Tuple of (value, unit) or (None, "") on error
        """
        response = self.connection.send_obd("02", f"{pid:02X}")
        parsed = self._parse_obd_response(response, "42")

        if not parsed:
            return None, ""

        try:
            data_line = parsed[0]
            data_hex = data_line[4:]
            data_bytes = bytes.fromhex(data_hex)
            result = decode_pid(pid, data_bytes)
            if result:
                return result
        except (ValueError, IndexError) as e:
            logger.error(f"Error parsing freeze frame PID {pid:02X}: {e}")

        return None, ""

    def get_vehicle_info(self) -> Dict:
        """Read vehicle information from Mode 09.

        Requests VIN (0x02), Calibration ID (0x04), ECU name (0x0A).

        Returns:
            Dictionary with {vin, calibration_id, ecu_name}
        """
        info = {}
        for info_type, info_key in [(0x02, "vin"), (0x04, "calibration_id"), (0x0A, "ecu_name")]:
            response = self.connection.send_obd("09", f"{info_type:02X}")
            parsed = self._parse_obd_response(response, "49")

            if parsed:
                try:
                    all_data = ""
                    for idx, line in enumerate(parsed):
                        # Skip mode (49) + infotype (XX) = 4 chars minimum
                        # If line has sequence counter (multi-frame CAN), skip 6 chars
                        # Detect by checking if 5th-6th chars look like a counter (01, 02, etc.)
                        skip = 4  # Default: mode + infotype
                        if len(line) > 6:
                            try:
                                counter = int(line[4:6], 16)
                                if 0 < counter <= 20:  # Valid sequence counter
                                    skip = 6
                            except ValueError:
                                pass
                        all_data += line[skip:]

                    data_bytes = bytes.fromhex(all_data)
                    info[info_key] = data_bytes.decode('ascii', errors='ignore').strip('\x00').strip()
                except (ValueError, UnicodeDecodeError):
                    info[info_key] = ""
            else:
                info[info_key] = ""

        return info

    def get_monitor_status(self) -> Dict:
        """Read monitor status from PID 0x01.

        Parses monitor status bits into human-readable format.

        Returns:
            Dictionary mapping monitor names to (available, complete) tuples
        """
        value, _ = self.read_pid(0x01)
        monitors = {}

        if value is None:
            return monitors

        status_byte = int(value) if isinstance(value, (int, float)) else 0

        monitor_bits = [
            ("Misfire Monitoring", 0),
            ("Fuel System Monitoring", 1),
            ("Component Monitoring", 2),
            ("Catalyst Monitoring", 3),
            ("Heated Catalyst Monitoring", 4),
            ("Evaporative System Monitoring", 5),
            ("Secondary Air System Monitoring", 6),
            ("A/C System Refrigerant Monitoring", 7),
        ]

        for name, bit in monitor_bits:
            available = bool(status_byte & (1 << bit))
            monitors[name] = (available, False)

        return monitors

    def _clear_dtcs(self) -> bool:
        """Clear DTCs using Mode 04.

        IMPORTANT: This is an internal method and should only be called through DTCManager which adds safety checks.

        Returns:
            True on success, False on failure
        """
        response = self.connection.send_obd("04", _internal=True)
        if response and "OK" in response:
            logger.info("DTCs cleared via Mode 04")
            return True
        logger.error(f"Failed to clear DTCs: {response}")
        return False

    def _parse_dtc_response(self, response: str, mode: str = "03") -> List[Dict]:
        """Parse DTC response from Mode 03/07/0A.

        In standard OBD-II:
        - Mode 03: stored (confirmed) DTCs
        - Mode 07: pending DTCs
        - Mode 0A: permanent DTCs

        Each DTC is 2 bytes. The bytes encode the DTC code, NOT status flags.

        Args:
            response: Response string from OBD command
            mode: OBD mode ("03", "07", or "0A")

        Returns:
            List of DTC records
        """
        mode_response_map = {"03": "43", "07": "47", "0A": "4A"}
        expected = mode_response_map.get(mode, "43")

        parsed = self._parse_obd_response(response, expected)
        dtcs = []

        if not parsed:
            return dtcs

        status_map = {"03": "Confirmed", "07": "Pending", "0A": "Permanent"}
        status_label = status_map.get(mode, "Active")

        for line in parsed:
            try:
                # Skip mode response byte (43/47/4A = 2 chars) + count byte (2 chars) = 4 chars
                data_hex = line[4:]

                for i in range(0, len(data_hex), 4):
                    if i + 4 > len(data_hex):
                        break
                    byte1 = int(data_hex[i:i+2], 16)
                    byte2 = int(data_hex[i+2:i+4], 16)

                    if byte1 == 0 and byte2 == 0:
                        continue

                    dtc_code = decode_dtc_bytes(byte1, byte2)
                    description = get_dtc_description(dtc_code)

                    dtcs.append({
                        "code": dtc_code,
                        "status": status_label,
                        "status_byte": 0,
                        "description": description,
                    })
            except (ValueError, IndexError) as e:
                logger.error(f"Error parsing DTC response line: {e}")

        return dtcs
