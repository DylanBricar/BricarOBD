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

    def read_all_dtcs(self) -> List[DTCRecord]:
        """Read DTCs from both OBD and UDS sources.

        Combines and deduplicates results.

        Returns:
            Sorted list of DTCRecord objects
        """
        dtc_dict = {}

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
                if key not in dtc_dict:
                    dtc_dict[key] = DTCRecord(
                        code=key,
                        description="",
                        status=", ".join(dtc.get("status_flags", [])),
                        status_byte=dtc.get("status_byte", 0),
                        source="UDS"
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

        success = self.obd_reader._clear_dtcs()
        if success:
            self.safety.log_operation("ClearDTC", 0x04, "", "OBD Mode 04 success")
            self.safety.record_dtc_clear()
            return True, "DTCs cleared successfully"

        success = self.uds_client.clear_dtc(0xFFFFFF)
        if success:
            self.safety.log_operation("ClearDTC", 0x14, "", "UDS Service 0x14 success")
            self.safety.record_dtc_clear()
            return True, "DTCs cleared successfully via UDS"

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
