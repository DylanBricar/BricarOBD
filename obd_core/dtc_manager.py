"""DTC management with safety guards and session storage."""

import json
import logging
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from obd_core.obd_reader import OBDReader
from obd_core.uds_client import UDSClient
from obd_core.safety import SafetyGuard
from config import SESSIONS_DIR

logger = logging.getLogger(__name__)


@dataclass
class DTCRecord:
    """Diagnostic Trouble Code record."""
    code: str
    description: str
    status: str
    status_byte: int
    source: str = "OBD"
    timestamp: str = ""
    frozen: bool = False

    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now().isoformat()


class DTCManager:
    """Manages DTC reading, storage, and export."""

    def __init__(self, obd_reader: OBDReader, uds_client: UDSClient, safety: SafetyGuard):
        """Initialize DTC manager.

        Args:
            obd_reader: OBDReader instance
            uds_client: UDSClient instance
            safety: SafetyGuard instance
        """
        self.obd_reader = obd_reader
        self.uds_client = uds_client
        self.safety = safety

    def read_all_dtcs(self, make: str = "") -> List[DTCRecord]:
        """Read DTCs from both OBD and UDS sources.

        Combines and deduplicates results.

        Args:
            make: Vehicle manufacturer (for ECU address selection)

        Returns:
            Sorted list of DTCRecord objects
        """
        dtc_dict = {}

        # Read ALL DTCs via custom connection (disconnect python-obd temporarily).
        # This is more reliable than querying through async python-obd.
        conn = self.obd_reader.connection
        if hasattr(conn, 'use_custom_connection'):
            def _read_all(custom_conn):
                """Read OBD + UDS DTCs from all ECUs."""
                obd_results = []
                uds_results = []

                # Ensure echo is off (clone ELM327 may not honor it in fast reconnect)
                custom_conn.send_command("ATE0")
                # Warm up CAN bus after fast reconnect
                warmup = custom_conn.send_command("0100", timeout=3)
                logger.debug(f"CAN warm-up response: {warmup[:60] if warmup else '(empty)'}")

                # 1. Read OBD DTCs (Mode 03) via raw serial
                logger.info("Reading OBD DTCs (Mode 03)...")
                response = custom_conn.send_obd("03")
                obd_results = self.obd_reader._parse_dtc_response(response, "03")
                logger.info(f"OBD Mode 03: {len(obd_results)} DTCs")

                # 2. Read pending DTCs (Mode 07)
                response = custom_conn.send_obd("07")
                pending = self.obd_reader._parse_dtc_response(response, "07")
                logger.info(f"OBD Mode 07: {len(pending)} pending DTCs")
                for dtc in pending:
                    dtc["status"] = "Pending"
                obd_results.extend(pending)

                # 3. Read permanent DTCs (Mode 0A)
                response = custom_conn.send_obd("0A")
                permanent = self.obd_reader._parse_dtc_response(response, "0A")
                logger.info(f"OBD Mode 0A: {len(permanent)} permanent DTCs")
                for dtc in permanent:
                    dtc["status"] = "Permanent"
                obd_results.extend(permanent)

                # 4. Read DTCs from all ECUs directly (skip tester_present)
                # Enable headers so we can see which ECU responds
                custom_conn.send_command("ATH1")
                # Use make passed as parameter (from app.detected_make)
                from obd_core.ecu_database import get_ecus_for_make
                if make:
                    candidates = get_ecus_for_make(make)
                else:
                    from obd_core.uds_client import GENERIC_ECUS
                    candidates = list(GENERIC_ECUS)
                logger.info(f"Scanning {len(candidates)} ECUs for DTCs...")

                for ecu in candidates:
                    if self.uds_client.set_target_ecu(ecu.request_id, ecu.response_id):
                        # Open extended diagnostic session first
                        self.uds_client.diagnostic_session_control(0x03)

                        # Try reading DTCs with multiple sub-functions
                        ecu_dtcs = self.uds_client.read_dtc_info(0x02, 0xFF)
                        if not ecu_dtcs:
                            # Try sub-function 0x02 with confirmed+testFailed mask
                            ecu_dtcs = self.uds_client.read_dtc_info(0x02, 0x09)
                        if not ecu_dtcs:
                            # Try reportSupportedDTC (0x0A) - some ECUs only support this
                            ecu_dtcs = self.uds_client.read_dtc_info(0x0A, 0xFF)

                        if ecu_dtcs:
                            for dtc in ecu_dtcs:
                                dtc['ecu_name'] = ecu.name
                            uds_results.extend(ecu_dtcs)
                            logger.info(f"  {ecu.name}: {len(ecu_dtcs)} DTCs")
                        else:
                            logger.debug(f"  {ecu.name}: no DTCs or no response")
                # Restore default state
                custom_conn.send_command("AT CRA")  # Clear receive filter
                custom_conn.send_command("ATH0")

                return {"obd": obd_results, "uds": uds_results}

            results = conn.use_custom_connection(_read_all)
            if results:
                for dtc in results.get("obd", []):
                    key = dtc.get("code", "")
                    if key:
                        dtc_dict[key] = DTCRecord(
                            code=key,
                            description=dtc.get("description", ""),
                            status=dtc.get("status", "Active"),
                            status_byte=dtc.get("status_byte", 0),
                            source="OBD"
                        )
                uds_dtcs = results.get("uds", [])
            else:
                uds_dtcs = []
        else:
            # Fallback: read OBD DTCs directly
            obd_dtcs = self.obd_reader.read_dtcs()
            for dtc in obd_dtcs:
                key = dtc.get("code", "")
                if key:
                    dtc_dict[key] = DTCRecord(
                        code=key,
                        description=dtc.get("description", ""),
                        status=dtc.get("status", ""),
                        status_byte=dtc.get("status_byte", 0),
                        source="OBD"
                    )
            uds_dtcs = self.uds_client.read_dtc_info()

        for dtc in uds_dtcs:
            key = dtc.get("dtc_code", "")
            if key:
                ecu_name = dtc.get("ecu_name", "UDS")
                if key not in dtc_dict:
                    dtc_dict[key] = DTCRecord(
                        code=key,
                        description="",
                        status=", ".join(dtc.get("status_flags", [])),
                        status_byte=dtc.get("status_byte", 0),
                        source=ecu_name
                    )

        records = list(dtc_dict.values())
        return sorted(records, key=lambda x: x.code)

    def read_pending_dtcs(self) -> List[DTCRecord]:
        """Read pending DTCs from OBD Mode 07.

        Returns:
            List of DTCRecord objects
        """
        dtcs = self.obd_reader.read_pending_dtcs()
        return self._convert_obd_dtcs(dtcs, "OBD")

    def read_permanent_dtcs(self) -> List[DTCRecord]:
        """Read permanent DTCs from OBD Mode 0A.

        Returns:
            List of DTCRecord objects
        """
        dtcs = self.obd_reader.read_permanent_dtcs()
        return self._convert_obd_dtcs(dtcs, "OBD")

    def backup_before_clear(self) -> Path:
        """Create automatic backup of DTCs and freeze frame data before clearing.

        Returns:
            Path to backup file
        """
        # Read current DTCs
        dtcs = self.read_all_dtcs()

        # Read freeze frame data for common PIDs
        freeze_data = {}
        freeze_pids = [0x04, 0x05, 0x0C, 0x0D, 0x0F, 0x11, 0x42]
        for pid in freeze_pids:
            val, unit = self.obd_reader.read_freeze_frame(pid)
            if val is not None:
                freeze_data[f"0x{pid:02X}"] = {"value": val, "unit": unit}

        # Create backup
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_data = {
            "type": "pre_clear_backup",
            "timestamp": datetime.now().isoformat(),
            "dtc_count": len(dtcs),
            "dtcs": [self._dtc_to_dict(dtc) for dtc in dtcs],
            "freeze_frame": freeze_data,
            "note": "Automatic backup before DTC clear"
        }

        filename = f"backup_pre_clear_{timestamp}.json"
        filepath = SESSIONS_DIR / filename

        filepath.write_text(json.dumps(backup_data, indent=2))
        logger.info(f"Pre-clear backup saved: {filepath}")

        return filepath

    def clear_all_dtcs(self, confirmed: bool = False) -> Tuple[bool, str]:
        """Clear all DTCs with safety validation.

        Args:
            confirmed: Must be True to proceed

        Returns:
            Tuple of (success, message)
        """
        if not confirmed:
            return False, "Confirmation required"

        allowed, reason = self.safety.validate_dtc_clear()
        if not allowed:
            return False, reason

        # Automatic backup before clearing
        try:
            backup_path = self.backup_before_clear()
            logger.info(f"Backup created at {backup_path}")
        except Exception as e:
            logger.warning(f"Backup failed (proceeding with clear): {e}")

        # Clear via custom connection (python-obd blocks Mode 04)
        conn = self.obd_reader.connection
        if hasattr(conn, 'use_custom_connection'):
            def _do_clear(custom_conn):
                custom_conn.send_command("ATE0")
                # Try OBD Mode 04 first
                resp = custom_conn.send_obd("04", _internal=True)
                if resp and ("44" in resp or "OK" in resp.upper()):
                    return "obd"
                # Fallback: UDS Service 0x14
                resp = custom_conn.send_raw("14FFFFFF")
                if resp and "54" in resp:
                    return "uds"
                return None

            result = conn.use_custom_connection(_do_clear)
            if result == "obd":
                self.safety.log_operation("ClearDTC", 0x04, "", "OBD Mode 04 success")
                self.safety.record_dtc_clear()
                return True, "DTCs cleared successfully"
            elif result == "uds":
                self.safety.log_operation("ClearDTC", 0x14, "", "UDS Service 0x14 success")
                self.safety.record_dtc_clear()
                return True, "DTCs cleared successfully via UDS"
        else:
            # Direct connection (no python-obd)
            success = self.obd_reader._clear_dtcs()
            if success:
                self.safety.log_operation("ClearDTC", 0x04, "", "OBD Mode 04 success")
                self.safety.record_dtc_clear()
                return True, "DTCs cleared successfully"

        return False, "Failed to clear DTCs via OBD and UDS"

    def save_dtcs(self, dtc_records: List[DTCRecord], filename: str = None) -> Path:
        """Save DTC records to JSON file.

        Args:
            dtc_records: List of DTCRecord objects
            filename: Optional custom filename

        Returns:
            Path to saved file
        """
        if not filename:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"dtc_session_{timestamp}.json"

        # Sanitize filename to prevent path traversal
        filename = Path(filename).name
        filepath = SESSIONS_DIR / filename
        data = [self._dtc_to_dict(dtc) for dtc in dtc_records]

        try:
            filepath.write_text(json.dumps(data, indent=2))
            logger.info(f"Saved {len(dtc_records)} DTCs to {filepath}")
            return filepath
        except Exception as e:
            logger.error(f"Error saving DTCs: {e}")
            raise

    def save_single_dtc(self, dtc: DTCRecord, filename: str = None) -> Path:
        """Save a single DTC record to JSON.

        Args:
            dtc: DTCRecord to save
            filename: Optional custom filename

        Returns:
            Path to saved file
        """
        if not filename:
            filename = f"dtc_{dtc.code}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"

        return self.save_dtcs([dtc], filename)

    def load_dtcs(self, filepath: Path) -> List[DTCRecord]:
        """Load DTC records from JSON file.

        Args:
            filepath: Path to JSON file

        Returns:
            List of DTCRecord objects
        """
        try:
            data = json.loads(filepath.read_text())
            # Handle both formats: list of dicts, or dict with "dtcs" key
            if isinstance(data, dict):
                dtc_list = data.get("dtcs", data.get("data", []))
            elif isinstance(data, list):
                dtc_list = data
            else:
                return []
            records = [self._dict_to_dtc(d) for d in dtc_list]
            logger.info(f"Loaded {len(records)} DTCs from {filepath}")
            return records
        except Exception as e:
            logger.error(f"Error loading DTCs: {e}")
            raise

    def get_saved_sessions(self) -> List[Dict]:
        """List all saved DTC sessions.

        Returns:
            List of {filename, date, dtc_count} dicts
        """
        sessions = []
        try:
            for filepath in SESSIONS_DIR.glob("*.json"):
                try:
                    data = json.loads(filepath.read_text())
                    sessions.append({
                        "filename": filepath.name,
                        "date": filepath.stat().st_mtime,
                        "dtc_count": len(data) if isinstance(data, list) else 0
                    })
                except Exception as e:
                    logger.warning(f"Error reading session {filepath.name}: {e}")
        except Exception as e:
            logger.error(f"Error listing sessions: {e}")

        return sorted(sessions, key=lambda x: x["date"], reverse=True)

    def export_to_text(self, dtc_records: List[DTCRecord], filepath: Path) -> bool:
        """Export DTC records to human-readable text file.

        Args:
            dtc_records: List of DTCRecord objects
            filepath: Path to output file

        Returns:
            True on success
        """
        try:
            lines = []
            lines.append("=" * 80)
            lines.append("OBD DIAGNOSTIC TROUBLE CODES REPORT")
            lines.append("=" * 80)
            lines.append(f"Generated: {datetime.now().isoformat()}")
            lines.append(f"Total DTCs: {len(dtc_records)}")
            lines.append("")

            lines.append("-" * 80)
            lines.append(f"{'Code':<10} {'Status':<30} {'Source':<10} {'Description':<30}")
            lines.append("-" * 80)

            for dtc in dtc_records:
                lines.append(
                    f"{dtc.code:<10} {dtc.status:<30} {dtc.source:<10} {dtc.description:<30}"
                )

            lines.append("-" * 80)
            lines.append("")

            filepath.write_text("\n".join(lines))
            logger.info(f"Exported DTCs to text file: {filepath}")
            return True
        except Exception as e:
            logger.error(f"Error exporting to text: {e}")
            return False

    def export_to_json(self, dtc_records: List[DTCRecord], filepath: Path) -> bool:
        """Export DTC records to JSON with metadata.

        Args:
            dtc_records: List of DTCRecord objects
            filepath: Path to output file

        Returns:
            True on success
        """
        try:
            data = {
                "metadata": {
                    "timestamp": datetime.now().isoformat(),
                    "tool_version": "1.0.0",
                    "dtc_count": len(dtc_records)
                },
                "dtcs": [self._dtc_to_dict(dtc) for dtc in dtc_records]
            }
            filepath.write_text(json.dumps(data, indent=2))
            logger.info(f"Exported DTCs to JSON file: {filepath}")
            return True
        except Exception as e:
            logger.error(f"Error exporting to JSON: {e}")
            return False

    def _convert_obd_dtcs(self, dtcs: list, source: str = "OBD") -> list:
        """Convert raw DTC dicts to DTCRecord objects.

        Args:
            dtcs: List of raw DTC dictionaries
            source: Source identifier (OBD, UDS, etc.)

        Returns:
            List of DTCRecord objects
        """
        records = []
        for dtc in dtcs:
            code = dtc.get("code", "")
            if code:
                records.append(DTCRecord(
                    code=code,
                    description=dtc.get("description", ""),
                    status=dtc.get("status", ""),
                    status_byte=dtc.get("status_byte", 0),
                    source=source,
                ))
        return records

    def _dtc_to_dict(self, dtc: DTCRecord) -> Dict:
        """Convert DTCRecord to dictionary for JSON serialization.

        Args:
            dtc: DTCRecord object

        Returns:
            Dictionary representation
        """
        return asdict(dtc)

    def _dict_to_dtc(self, d: Dict) -> DTCRecord:
        """Convert dictionary to DTCRecord from JSON.

        Args:
            d: Dictionary from JSON

        Returns:
            DTCRecord object
        """
        return DTCRecord(
            code=d.get("code", ""),
            description=d.get("description", ""),
            status=d.get("status", ""),
            status_byte=d.get("status_byte", 0),
            source=d.get("source", "OBD"),
            timestamp=d.get("timestamp", datetime.now().isoformat()),
            frozen=d.get("frozen", False)
        )
