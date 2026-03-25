"""UDS (Unified Diagnostic Services) client for extended diagnostics."""

import logging
import time
from typing import Optional, List, Dict
from obd_core.connection import ELM327Connection
from obd_core.safety import SafetyGuard
from obd_core.ecu_database import GENERIC_ECUS, get_ecus_for_vehicle

logger = logging.getLogger(__name__)


class UDSClient:
    """UDS client for extended diagnostic operations."""

    NRC_DESCRIPTIONS = {
        0x10: "General Reject",
        0x11: "Service Not Supported",
        0x12: "Sub-function Not Supported",
        0x13: "Incorrect Message Length",
        0x14: "Response Too Long",
        0x21: "Busy Repeat Request",
        0x22: "Conditions Not Correct",
        0x24: "Request Sequence Error",
        0x25: "No Response From Subnet",
        0x26: "Failure Prevents Execution",
        0x31: "Request Out Of Range",
        0x33: "Security Access Denied",
        0x35: "Invalid Key",
        0x36: "Exceeded Number Of Attempts",
        0x37: "Required Time Delay Not Expired",
        0x72: "General Programming Failure",
        0x73: "Wrong Block Sequence Counter",
        0x78: "Request Correctly Received - Response Pending",
    }

    def __init__(self, connection: ELM327Connection, safety: SafetyGuard):
        """Initialize UDS client.

        Args:
            connection: ELM327Connection instance
            safety: SafetyGuard instance for operation validation
        """
        self.connection = connection
        self.safety = safety
        self.target_request_id = 0x7E0
        self.target_response_id = 0x7E8

    def set_target_ecu(self, request_id: int, response_id: int) -> bool:
        """Set CAN header and receive filter for specific ECU.

        Args:
            request_id: Request CAN ID (e.g., 0x7E0)
            response_id: Response CAN ID (e.g., 0x7E8)

        Returns:
            True on success
        """
        try:
            # Set send header
            cmd = f"AT SH {request_id:03X}"
            response = self.connection.send_command(cmd)
            if "OK" not in response:
                return False

            # Set receive filter so ELM327 listens for this ECU's responses
            # Without this, non-standard CAN IDs (PSA, VAG extended) get no response
            cra_cmd = f"AT CRA {response_id:03X}"
            self.connection.send_command(cra_cmd)

            self.target_request_id = request_id
            self.target_response_id = response_id
            logger.info(f"Set target ECU to {request_id:03X}/{response_id:03X}")
            return True
        except Exception as e:
            logger.error(f"Error setting target ECU: {e}")
        return False

    def set_ecu_by_name(self, ecu_name: str) -> bool:
        """Look up ECU from database and set as target.

        Args:
            ecu_name: ECU name to search for

        Returns:
            True if found and set
        """
        for ecu in GENERIC_ECUS:
            if ecu.name.lower() == ecu_name.lower():
                return self.set_target_ecu(ecu.request_id, ecu.response_id)
        return False

    def diagnostic_session_control(self, session_type: int = 0x01) -> bool:
        """Change diagnostic session (Service 0x10).

        Args:
            session_type: Session type (0x01=default, 0x03=extended)

        Returns:
            True on success
        """
        data = f"10{session_type:02X}"
        response = self._send_uds(data)
        result = self._parse_response(response, 0x50)
        success = result is not None
        self.safety.log_operation("DiagnosticSessionControl", 0x10, data, "OK" if success else "FAILED")
        return success

    def tester_present(self) -> bool:
        """Send tester present message (Service 0x3E).

        Keeps diagnostic session alive.

        Returns:
            True on success
        """
        data = "3E00"
        response = self._send_uds(data)
        # Check for positive response: 7E in response (with or without headers)
        if response:
            clean = response.replace(" ", "").replace("\n", "").replace("\r", "")
            # Positive response is 7E00 somewhere in the response
            if "7E00" in clean and "NO DATA" not in response and "ERROR" not in response:
                self.safety.log_operation("TesterPresent", 0x3E, data, "OK")
                logger.debug(f"TesterPresent OK: {response.strip()[:60]}")
                return True
        self.safety.log_operation("TesterPresent", 0x3E, data, f"FAILED: {response.strip()[:40] if response else 'empty'}")
        logger.debug(f"TesterPresent FAILED: {response.strip()[:60] if response else 'empty'}")
        return False

    def read_dtc_info(self, sub_function: int = 0x02, status_mask: int = 0xFF) -> List[Dict]:
        """Read DTC information (Service 0x19).

        Args:
            sub_function: 0x01=DTC by status, 0x02=DTC and status, 0x03=Mirror memory
            status_mask: DTC status mask

        Returns:
            List of {dtc_code, status_byte, status_flags} dicts
        """
        data = f"19{sub_function:02X}{status_mask:02X}"
        response = self._send_uds(data)
        result = self._parse_response(response, 0x59)

        dtcs = []
        if result:
            try:
                # Skip sub-function echo (1 byte) + status availability mask (1 byte)
                dtc_data = result[2:]
                # Each DTC is 3 bytes (hi, mid, lo) + 1 status byte = 4 bytes
                for i in range(0, len(dtc_data) - 3, 4):
                    dtc_hi = dtc_data[i]
                    dtc_mid = dtc_data[i + 1]
                    dtc_lo = dtc_data[i + 2]
                    status = dtc_data[i + 3]

                    dtc_code = f"{dtc_hi:02X}{dtc_mid:02X}{dtc_lo:02X}"
                    dtcs.append({
                        "dtc_code": dtc_code,
                        "status_byte": status,
                        "status_flags": self._parse_dtc_status(status),
                    })
            except (IndexError, TypeError):
                pass

        self.safety.log_operation("ReadDTCInfo", 0x19, data, f"Read {len(dtcs)} DTCs")
        return dtcs

    def read_data_by_id(self, did: int) -> Optional[bytes]:
        """Read data by identifier (Service 0x22).

        Args:
            did: Data Identifier to read

        Returns:
            Data bytes or None on failure
        """
        data = f"22{did:04X}"
        response = self._send_uds(data)
        result = self._parse_response(response, 0x62)
        self.safety.log_operation("ReadDataByID", 0x22, data, "OK" if result else "FAILED")
        return result

    def clear_dtc(self, dtc_group: int = 0xFFFFFF) -> bool:
        """Clear DTCs (Service 0x14).

        Checks safety validation before clearing.

        Args:
            dtc_group: DTC group (0xFFFFFF = all)

        Returns:
            True on success
        """
        allowed, reason = self.safety.is_operation_allowed(0x14)
        if not allowed:
            logger.error(f"Clear DTC not allowed: {reason}")
            return False

        data = f"14{dtc_group:06X}"
        response = self._send_uds(data)
        result = self._parse_response(response, 0x54)
        success = result is not None

        self.safety.log_operation("ClearDTC", 0x14, data, "OK" if success else "FAILED")
        return success

    def read_vin(self) -> str:
        """Read VIN from DID 0xF190.

        Returns:
            VIN string or empty string on failure
        """
        data = self.read_data_by_id(0xF190)
        if data:
            try:
                return data.decode('ascii', errors='ignore').strip()
            except Exception:
                pass
        return ""

    def read_ecu_info(self) -> Dict:
        """Read ECU information from multiple DIDs.

        Returns:
            Dictionary with {vin, serial, hw_version, sw_version}
        """
        info = {}

        dids = {
            0xF190: "vin",
            0xF18C: "serial",
            0xF191: "hw_version",
            0xF189: "sw_version"
        }

        for did, key in dids.items():
            data = self.read_data_by_id(did)
            if data:
                try:
                    info[key] = data.decode('ascii', errors='ignore').strip()
                except Exception:
                    info[key] = ""
            else:
                info[key] = ""

        return info

    def scan_ecus(self, make: str = "") -> List[Dict]:
        """Scan for responding ECUs using generic + manufacturer-specific addresses.

        Args:
            make: Vehicle manufacturer (from VIN decode). If provided,
                  also scans manufacturer-specific extended ECU addresses.

        Returns:
            List of {name, request_id, response_id} dicts for responding ECUs
        """
        from obd_core.ecu_database import get_ecus_for_make

        # Build candidate list: generic + manufacturer-specific
        if make:
            candidates = get_ecus_for_make(make)
            logger.info(f"Scanning {len(candidates)} ECU addresses for {make}")
        else:
            candidates = list(GENERIC_ECUS)
            logger.info(f"Scanning {len(candidates)} generic ECU addresses")

        responding_ecus = []
        seen_ids = set()

        for ecu in candidates:
            # Skip duplicates (generic + extended might overlap)
            if ecu.request_id in seen_ids:
                continue
            seen_ids.add(ecu.request_id)

            if self.set_target_ecu(ecu.request_id, ecu.response_id):
                if self.tester_present():
                    responding_ecus.append({
                        "name": ecu.name,
                        "request_id": ecu.request_id,
                        "response_id": ecu.response_id
                    })
                    logger.info(f"Found responding ECU: {ecu.name} (0x{ecu.request_id:03X})")

        # Restore default ELM327 state so python-obd queries keep working.
        try:
            self.connection.send_command("AT D")    # Reset to defaults
            self.connection.send_command("AT CRA")  # Clear receive filter (accept all)
            self.connection.send_command("AT H0")   # Headers off
            logger.info("Restored default ELM327 headers after ECU scan")
        except Exception:
            pass

        return responding_ecus

    def read_extended_data(self, make: str = "") -> Dict:
        """Read manufacturer-specific extended DIDs from current target ECU.

        Only reads DIDs that are in the manufacturer's known DID list.
        All operations are READ-ONLY (Service 0x22).

        Args:
            make: Vehicle manufacturer for DID selection

        Returns:
            Dict of {did_hex: {value, description}} for each successful read
        """
        from obd_core.ecu_database import get_extended_dids

        dids = get_extended_dids(make)
        results = {}

        for did, description in dids.items():
            # Skip standard identification DIDs (already read by read_ecu_info)
            if did >= 0xF180:
                continue

            data = self.read_data_by_id(did)
            if data:
                try:
                    # Try ASCII decode for text data
                    text = data.decode('ascii', errors='ignore').strip('\x00').strip()
                    if text and text.isprintable():
                        value = text
                    else:
                        value = data.hex().upper()
                except Exception:
                    value = data.hex().upper()

                results[f"0x{did:04X}"] = {
                    "value": value,
                    "description": description,
                    "raw": data.hex().upper()
                }

        return results

    def _send_uds(self, data_hex: str) -> str:
        """Send raw UDS command with safety validation and response-pending handling."""
        try:
            service_id = int(data_hex[:2], 16)
        except (ValueError, IndexError):
            logger.error(f"Invalid UDS data: {data_hex}")
            return ""

        allowed, reason = self.safety.is_operation_allowed(service_id)
        if not allowed:
            logger.error(f"UDS service 0x{service_id:02X} blocked: {reason}")
            return ""

        response = self.connection.send_raw(data_hex)
        logger.debug(f"UDS TX: {data_hex} → RX: {response.strip()[:80] if response else '(empty)'}")

        # Handle Response Pending (NRC 0x78) - re-read only, do NOT re-send
        max_pending_retries = 10
        for attempt in range(max_pending_retries):
            response_clean = response.replace(" ", "").replace("\n", "")
            if response_clean.startswith("7F") and len(response_clean) >= 6:
                try:
                    nrc = int(response_clean[4:6], 16)
                    if nrc == 0x78:
                        logger.debug(f"Response pending (attempt {attempt + 1}), waiting...")
                        time.sleep(2)  # Wait for ECU to process
                        # Read the next response without sending anything
                        try:
                            response = self.connection._read_until_prompt(5)
                        except Exception:
                            break
                        continue
                except (ValueError, IndexError):
                    pass
            break  # Not a 0x78, we have the final response

        self.safety.log_operation("UDS", service_id, data_hex, response)
        return response

    def _parse_response(self, response: str, expected_sid: int) -> Optional[bytes]:
        """Parse UDS response and validate SID.

        Args:
            response: Response string
            expected_sid: Expected service ID response

        Returns:
            Data bytes or None on failure
        """
        if not response or "NO DATA" in response or "UNABLE" in response:
            return None

        # Clean response: remove spaces, newlines, and echo
        lines = [l.strip() for l in response.replace('\r', '\n').split('\n') if l.strip()]
        # Filter out lines that look like echo (start with our request) or are empty
        clean_lines = []
        for line in lines:
            stripped = line.replace(" ", "")
            if stripped and "NODATA" not in stripped and "UNABLE" not in stripped:
                clean_lines.append(stripped)

        if not clean_lines:
            return None

        # Try each response line
        expected_hex = f"{expected_sid:02X}"
        for line in clean_lines:
            # Check for negative response
            if line.startswith("7F"):
                try:
                    rejected_sid = int(line[2:4], 16)
                    nrc = int(line[4:6], 16)
                    description = self._parse_negative_response(nrc)
                    logger.error(f"Negative response for SID 0x{rejected_sid:02X}: {description} (NRC 0x{nrc:02X})")
                except (ValueError, IndexError):
                    logger.error(f"Malformed negative response: {line}")
                return None

            # Direct match (no headers)
            if line.startswith(expected_hex):
                try:
                    data_hex = line[2:]
                    if data_hex:
                        return bytes.fromhex(data_hex)
                except ValueError:
                    pass

            # With CAN headers: look for SID after header bytes
            # CAN header formats: "7E8 06 59..." -> cleaned "7E80659..."
            # Try to find the expected SID in the line
            idx = line.find(expected_hex)
            if idx > 0:
                # Validate: the SID should be preceded by a length byte or header
                try:
                    data_hex = line[idx + 2:]
                    if data_hex:
                        return bytes.fromhex(data_hex)
                except ValueError:
                    pass

        return None

    def _parse_negative_response(self, nrc_byte: int) -> str:
        """Get human-readable NRC description.

        Args:
            nrc_byte: Negative Response Code byte

        Returns:
            NRC description string
        """
        return self.NRC_DESCRIPTIONS.get(nrc_byte, f"Unknown NRC (0x{nrc_byte:02X})")

    def _parse_dtc_status(self, status_byte: int) -> List[str]:
        """Parse DTC status byte into flags.

        Args:
            status_byte: Status byte

        Returns:
            List of status flags
        """
        flags = []
        if status_byte & 0x01:
            flags.append("Test Failed")
        if status_byte & 0x02:
            flags.append("Test Failed This Cycle")
        if status_byte & 0x04:
            flags.append("Pending")
        if status_byte & 0x08:
            flags.append("Confirmed")
        if status_byte & 0x10:
            flags.append("Test Not Completed")
        if status_byte & 0x20:
            flags.append("Test Not Completed This Cycle")
        if status_byte & 0x40:
            flags.append("Warning Indicator")
        if status_byte & 0x80:
            flags.append("ODX Available")
        return flags
