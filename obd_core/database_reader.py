"""ECU database reader — loads manufacturer ECU definitions from the DDT2000 database.

Reads the 4867-file ECU database (split ZIP) on demand. Each ECU file contains:
- requests[]: Commands to send (sentbytes) and their response parameter mappings
- data{}: Parameter decoders with step/offset/divideby formulas
"""

import json
import logging
import os
import zipfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from config import DATA_DIR

logger = logging.getLogger(__name__)

# Database ZIP path (reassembled from parts)
_DB_ZIP_PATH = DATA_DIR / "bricarobd_db.zip"
_DB_PARTS_PATTERN = "bricarobd_db_part_a"


@dataclass
class ECUParameter:
    """Decoded parameter definition from ECU database."""
    name: str
    unit: str = ""
    step: float = 1.0
    offset: float = 0.0
    divideby: float = 1.0
    signed: bool = False
    bitscount: int = 8
    bytescount: int = 1
    comment: str = ""
    scaled: bool = False


@dataclass
class ParamMapping:
    """Maps a parameter name to its position in a response."""
    param_name: str
    firstbyte: int
    bitoffset: Optional[int] = None  # If set, extract single bit


@dataclass
class ECURequest:
    """A command that can be sent to the ECU."""
    sentbytes: str  # Hex command (e.g., "21A4")
    name: str
    min_bytes: int = 0
    params: List[ParamMapping] = field(default_factory=list)


@dataclass
class ECUDefinition:
    """Complete ECU definition with requests and parameter decoders."""
    ecuname: str
    filename: str
    endian: str = "Big"
    address: str = ""
    requests: List[ECURequest] = field(default_factory=list)
    parameters: Dict[str, ECUParameter] = field(default_factory=dict)

    def get_read_requests(self) -> List[ECURequest]:
        """Get only requests that read data (Mode 21, 22, not actuators)."""
        reads = []
        for req in self.requests:
            cmd = req.sentbytes.upper()
            # Mode 21 (ReadLocalIdentifier) or 22 (ReadDataByIdentifier)
            # Exclude: 30/31/32 (actuators), 14 (clear), 10 (session), 3B (config)
            if cmd.startswith(("21", "22", "19")) and req.params:
                reads.append(req)
            # Also include Mode 17 (read DTCs) if it has params
            elif cmd.startswith("17") and req.params:
                reads.append(req)
        return reads

    def decode_response(self, request: ECURequest, response_bytes: bytes) -> Dict[str, object]:
        """Decode a response using the parameter definitions.

        Args:
            request: The request that was sent
            response_bytes: Raw response bytes (after stripping echo/headers)

        Returns:
            Dict mapping parameter name to decoded value (float or bool)
        """
        results = {}
        endian = "big" if self.endian == "Big" else "little"

        for mapping in request.params:
            param = self.parameters.get(mapping.param_name)
            if not param:
                continue

            try:
                byte_pos = mapping.firstbyte
                if byte_pos >= len(response_bytes):
                    continue

                # Bit extraction (boolean)
                if mapping.bitoffset is not None:
                    byte_val = response_bytes[byte_pos]
                    bit_val = bool(byte_val & (1 << mapping.bitoffset))
                    results[mapping.param_name] = bit_val
                    continue

                # Byte extraction
                n_bytes = param.bytescount
                if byte_pos + n_bytes > len(response_bytes):
                    continue

                raw_bytes = response_bytes[byte_pos:byte_pos + n_bytes]
                raw_value = int.from_bytes(raw_bytes, endian)

                # Signed interpretation
                if param.signed:
                    max_val = 1 << (n_bytes * 8)
                    if raw_value >= max_val // 2:
                        raw_value -= max_val

                # Scale: value = raw * step + offset
                if param.scaled:
                    value = raw_value * param.step + param.offset
                    if param.divideby != 1.0 and param.divideby != 0:
                        value /= param.divideby
                else:
                    value = float(raw_value)

                results[mapping.param_name] = round(value, 4)

            except (IndexError, ValueError, OverflowError):
                continue

        return results


class ECUDatabase:
    """Loads ECU definitions from the DDT2000 database ZIP."""

    def __init__(self):
        self._zip_path: Optional[Path] = None
        self._index: Dict[str, dict] = {}  # filename → {address, ecuname, projects, autoidents}
        self._loaded = False
        self._zip_file: Optional[zipfile.ZipFile] = None

    @property
    def is_loaded(self) -> bool:
        return self._loaded

    @property
    def ecu_count(self) -> int:
        return len(self._index)

    def load(self) -> bool:
        """Load the database index. Reassembles ZIP from parts if needed.

        Returns:
            True if database loaded successfully
        """
        if self._loaded:
            return True

        # Find or reassemble the ZIP
        zip_path = self._find_or_assemble_zip()
        if not zip_path:
            logger.warning("ECU database not found")
            return False

        # Load the ECU index (close any previous handle first)
        try:
            if self._zip_file:
                try:
                    self._zip_file.close()
                except Exception:
                    pass
            self._zip_file = zipfile.ZipFile(zip_path, 'r')
            with self._zip_file.open("ecus/index.json") as f:
                self._index = json.load(f)
            self._zip_path = zip_path
            self._loaded = True
            logger.info(f"ECU database loaded: {len(self._index)} ECU definitions")
            return True
        except (zipfile.BadZipFile, KeyError, json.JSONDecodeError) as e:
            logger.error(f"Failed to load ECU database: {e}")
            return False

    def close(self):
        """Close the ZIP file handle."""
        if self._zip_file:
            try:
                self._zip_file.close()
            except Exception:
                pass
            self._zip_file = None

    def search_ecus(self, query: str) -> List[dict]:
        """Search ECU index by name or project.

        Args:
            query: Search term (matches ecuname or project codes)

        Returns:
            List of {filename, ecuname, address, projects} dicts
        """
        if not self._loaded:
            return []

        query_lower = query.lower()
        results = []
        for filename, info in self._index.items():
            name = info.get("ecuname", "").lower()
            projects = [p.lower() for p in info.get("projects", [])]
            if (query_lower in name
                    or any(query_lower in p for p in projects)
                    or query_lower in filename.lower()):
                results.append({
                    "filename": filename,
                    "ecuname": info.get("ecuname", ""),
                    "address": info.get("address", ""),
                    "projects": info.get("projects", []),
                })
        return results[:100]  # Limit results

    def find_ecus_by_address(self, address: str) -> List[dict]:
        """Find all ECU definitions for a given diagnostic address.

        Args:
            address: Hex address string (e.g., "7A" for engine)

        Returns:
            List of matching ECU index entries
        """
        if not self._loaded:
            return []
        addr_upper = address.upper()
        return [
            {"filename": fn, **info}
            for fn, info in self._index.items()
            if info.get("address", "").upper() == addr_upper
        ]

    def get_all_addresses(self) -> List[Tuple[str, str]]:
        """Get all unique ECU addresses with a sample name.

        Returns:
            List of (address, sample_ecuname) tuples, sorted
        """
        if not self._loaded:
            return []
        addr_map = {}
        for info in self._index.values():
            addr = info.get("address", "")
            if addr and addr not in addr_map:
                addr_map[addr] = info.get("ecuname", "Unknown")
        return sorted(addr_map.items())

    def load_ecu_definition(self, filename: str) -> Optional[ECUDefinition]:
        """Load a full ECU definition from the ZIP.

        Args:
            filename: ECU JSON filename (e.g., "S2000_Atmo___SoftA3.json")

        Returns:
            ECUDefinition or None on error
        """
        if not self._zip_file:
            return None

        zip_path = f"ecus/{filename}"
        try:
            with self._zip_file.open(zip_path) as f:
                raw = json.load(f)
        except (KeyError, json.JSONDecodeError) as e:
            logger.error(f"Failed to load ECU {filename}: {e}")
            return None

        return self._parse_ecu_definition(raw, filename)

    def _parse_ecu_definition(self, raw: dict, filename: str) -> ECUDefinition:
        """Parse raw JSON into ECUDefinition."""
        ecu = ECUDefinition(
            ecuname=raw.get("ecuname", "Unknown"),
            filename=filename,
            endian=raw.get("endian", "Big"),
            address=self._index.get(filename, {}).get("address", ""),
        )

        # Parse parameters (data dict)
        raw_data = raw.get("data", {})
        if isinstance(raw_data, dict):
            for name, info in raw_data.items():
                if not isinstance(info, dict):
                    continue
                ecu.parameters[name] = ECUParameter(
                    name=name,
                    unit=info.get("unit", ""),
                    step=float(info.get("step", 1.0)),
                    offset=float(info.get("offset", 0.0)),
                    divideby=float(info.get("divideby", 1.0)),
                    signed=bool(info.get("signed", False)),
                    bitscount=int(info.get("bitscount", 8)),
                    bytescount=int(info.get("bytescount", 1)),
                    comment=info.get("comment", ""),
                    scaled=bool(info.get("scaled", False)),
                )

        # Parse requests
        raw_requests = raw.get("requests", [])
        if isinstance(raw_requests, list):
            for req_raw in raw_requests:
                if not isinstance(req_raw, dict):
                    continue
                req = ECURequest(
                    sentbytes=req_raw.get("sentbytes", ""),
                    name=req_raw.get("name", ""),
                    min_bytes=int(req_raw.get("minbytes", 0)),
                )
                # Parse parameter mappings
                items = req_raw.get("receivebyte_dataitems", {})
                for param_name, mapping_info in items.items():
                    req.params.append(ParamMapping(
                        param_name=param_name,
                        firstbyte=int(mapping_info.get("firstbyte", 0)),
                        bitoffset=mapping_info.get("bitoffset"),
                    ))
                if req.sentbytes:
                    ecu.requests.append(req)

        return ecu

    def _find_or_assemble_zip(self) -> Optional[Path]:
        """Find existing ZIP or reassemble from parts."""
        # Check assembled ZIP
        if _DB_ZIP_PATH.exists() and _DB_ZIP_PATH.stat().st_size > 1_000_000:
            return _DB_ZIP_PATH

        # Check /tmp
        tmp_zip = Path("/tmp/bricarobd_db.zip")
        if tmp_zip.exists() and tmp_zip.stat().st_size > 1_000_000:
            return tmp_zip

        # Reassemble from parts
        parts = sorted(DATA_DIR.glob(f"{_DB_PARTS_PATTERN}*"))
        if not parts:
            return None

        logger.info(f"Reassembling database from {len(parts)} parts...")
        try:
            with open(tmp_zip, 'wb') as out:
                for part in parts:
                    with open(part, 'rb') as inp:
                        while True:
                            chunk = inp.read(8192 * 1024)  # 8MB chunks
                            if not chunk:
                                break
                            out.write(chunk)
            logger.info(f"Database reassembled: {tmp_zip.stat().st_size / 1e6:.0f} MB")
            return tmp_zip
        except (IOError, OSError) as e:
            logger.error(f"Failed to reassemble database: {e}")
            return None
