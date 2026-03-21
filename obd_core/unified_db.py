"""BricarOBD unified database loader.

Reads data/bricarobd_database.zip — a single archive containing
all diagnostic data: ECU definitions, operations, parameters,
formulas, and vehicle profiles.

Structure inside the zip:
  index.json — Pre-compiled operations index (3.3M ops)
  ecus/index.json — ECU definitions index
  ecus/*.json — Individual ECU definition files (full data)
  ext/ — Additional vehicle PID profiles
"""

from __future__ import annotations

import json
import logging
import sys
import zipfile
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


def _find_db_path() -> Path:
    """Find the database zip in dev mode or PyInstaller bundle.

    If the zip doesn't exist but split parts do (bricarobd_db_part_*),
    reassemble them automatically on first launch.
    """
    if hasattr(sys, '_MEIPASS'):
        base = Path(sys._MEIPASS) / "data"
    else:
        base = Path(__file__).parent.parent / "data"

    zip_path = base / "bricarobd_database.zip"

    # Auto-reassemble from parts if zip doesn't exist
    if not zip_path.exists():
        parts = sorted(base.glob("bricarobd_db_part_*"))
        if parts:
            logger.info(f"Assembling database from {len(parts)} parts...")
            try:
                with open(zip_path, "wb") as out:
                    for part in parts:
                        out.write(part.read_bytes())
                logger.info(f"Database assembled: {zip_path}")
            except Exception as e:
                logger.error(f"Failed to assemble database: {e}")

    return zip_path


_DB_PATH = _find_db_path()


class BricarDatabase:
    """BricarOBD diagnostic database — reads from a single zip archive."""

    def __init__(self):
        self._ops: list = []
        self._metadata: dict = {}
        self._loaded = False
        self._index_by_manufacturer: dict[str, list[int]] = {}
        self._zip: Optional[zipfile.ZipFile] = None

    def is_available(self) -> bool:
        return _DB_PATH.exists()

    def load(self) -> bool:
        """Load the operations index from the database archive."""
        if self._loaded:
            return True
        if not _DB_PATH.exists():
            logger.warning("BricarOBD database not found")
            return False
        try:
            logger.info("Loading BricarOBD database...")
            self._zip = zipfile.ZipFile(_DB_PATH, "r")
            with self._zip.open("index.json") as f:
                data = json.load(f)
            self._ops = data.get("operations", [])
            self._metadata = data.get("metadata", {})
            self._build_index()
            self._loaded = True
            logger.info(f"BricarOBD DB: {len(self._ops):,} operations loaded")
            return True
        except Exception as e:
            logger.error(f"Failed to load database: {e}")
            return False

    def _build_index(self):
        """Build manufacturer index for fast filtering."""
        self._index_by_manufacturer = {}
        for i, op in enumerate(self._ops):
            mfr = op.get("manufacturer", "other")
            if mfr not in self._index_by_manufacturer:
                self._index_by_manufacturer[mfr] = []
            self._index_by_manufacturer[mfr].append(i)

    def get_metadata(self) -> dict:
        if not self._loaded:
            self.load()
        return self._metadata

    def get_operations_for_manufacturer(self, manufacturer: str,
                                         query: str = "",
                                         op_type: str = None,
                                         limit: int = 500) -> list[dict]:
        """Get operations filtered by manufacturer and optional search."""
        if not self._loaded:
            self.load()

        indices = self._index_by_manufacturer.get(manufacturer, [])
        if not indices:
            return []

        query_lower = query.lower() if query else ""
        results = []

        for idx in indices:
            if len(results) >= limit:
                break
            op = self._ops[idx]
            if op_type and op.get("type") != op_type:
                continue
            if query_lower:
                searchable = f"{op.get('name', '')} {op.get('sentbytes', '')} {op.get('did', '')} {op.get('ecu_name', '')}".lower()
                if query_lower not in searchable:
                    continue
            results.append(op)

        return results

    def get_groups_for_manufacturer(self, manufacturer: str) -> list[dict]:
        """Get ECU groups with operation counts for a manufacturer."""
        if not self._loaded:
            self.load()

        indices = self._index_by_manufacturer.get(manufacturer, [])
        groups = {}
        for idx in indices:
            op = self._ops[idx]
            ecu = op.get("ecu_name", "Unknown")
            if ecu not in groups:
                groups[ecu] = {"name": ecu, "total": 0, "reads": 0, "writes": 0,
                               "ecu_tx": op.get("ecu_tx", ""), "ecu_rx": op.get("ecu_rx", "")}
            groups[ecu]["total"] += 1
            if op.get("safe"):
                groups[ecu]["reads"] += 1
            else:
                groups[ecu]["writes"] += 1

        return sorted(groups.values(), key=lambda g: -g["total"])

    def get_manufacturer_stats(self) -> dict[str, int]:
        if not self._loaded:
            self.load()
        return {k: len(v) for k, v in self._index_by_manufacturer.items()}

    def count_for_manufacturer(self, manufacturer: str) -> int:
        if not self._loaded:
            self.load()
        return len(self._index_by_manufacturer.get(manufacturer, []))

    def load_ecu_definition(self, filename: str) -> Optional[dict]:
        """Load a full ECU definition file from the archive."""
        if not self._zip:
            return None
        try:
            with self._zip.open(f"ecus/{filename}") as f:
                return json.load(f)
        except Exception:
            return None

    def get_ecu_index(self) -> Optional[dict]:
        """Load the ECU definitions index."""
        if not self._zip:
            return None
        try:
            with self._zip.open("ecus/index.json") as f:
                return json.load(f)
        except Exception:
            return None


# ── Make → manufacturer mapping ───────────────────────────────
_MAKE_TO_MANUFACTURER = {
    "Peugeot": "psa", "Citroën": "psa", "Citroen": "psa",
    "DS": "psa", "Opel": "psa", "Vauxhall": "psa",
    "Volkswagen": "vag", "Audi": "vag", "Seat": "vag",
    "Cupra": "vag", "Skoda": "vag", "Škoda": "vag", "Porsche": "vag",
    "BMW": "bmw", "Mini": "bmw", "MINI": "bmw",
    "Mercedes-Benz": "mercedes", "Mercedes": "mercedes",
    "Renault": "renault", "Dacia": "renault",
    "Nissan": "renault", "Infiniti": "renault",
    "Hyundai": "hyundai", "Kia": "hyundai", "Genesis": "hyundai",
    "Ford": "ford", "Lincoln": "ford",
    "Toyota": "toyota", "Lexus": "toyota",
    "Honda": "honda", "Acura": "honda",
    "Volvo": "volvo",
    "Fiat": "fiat", "Alfa Romeo": "fiat",
    "Lancia": "fiat", "Abarth": "fiat", "Maserati": "fiat",
    "Subaru": "subaru", "Mazda": "other",
}


def make_to_manufacturer(make: str) -> str:
    return _MAKE_TO_MANUFACTURER.get(make, "other")


# Singleton
_instance: Optional[BricarDatabase] = None


def get_unified_db() -> BricarDatabase:
    global _instance
    if _instance is None:
        _instance = BricarDatabase()
    return _instance
