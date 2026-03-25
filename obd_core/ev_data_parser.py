"""EV/Battery data parser — loads extended PID definitions from CSV and JSON formats."""

import csv
import io
import json
import logging
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional

logger = logging.getLogger(__name__)


@dataclass
class EVParam:
    """Extended PID parameter for EV/battery monitoring."""
    name: str
    short_name: str
    command: str        # hex command (e.g., "220101")
    equation: str       # formula as string
    min_val: float
    max_val: float
    unit: str
    ecu_header: str     # CAN header (e.g., "7E4")
    source_file: str    # Which file it came from
    vehicle: str        # Vehicle model name


class EVDataParser:
    """Parses EV extended PID definitions from multiple formats."""

    def __init__(self, zip_path: Path = None):
        self._zip_path = zip_path or Path("/tmp/bricarobd_db.zip")
        self._all_params: List[EVParam] = []
        self._loaded = False
        self._vehicle_index: Dict[str, List[EVParam]] = {}  # vehicle_keyword → params

    @property
    def is_loaded(self) -> bool:
        return self._loaded

    def load(self) -> bool:
        """Load all EV parameters from the ZIP."""
        if self._loaded:
            return True
        if not self._zip_path.exists():
            return False

        try:
            with zipfile.ZipFile(self._zip_path, 'r') as zf:
                # Single pass over namelist — classify then parse
                torque_csvs = []
                wican_jsons = []
                ev_obd_jsons = []
                for name in zf.namelist():
                    name_lower = name.lower()
                    if name.endswith('.csv') and ('extendedpids' in name_lower or 'torque' in name_lower):
                        torque_csvs.append(name)
                    elif 'wican-fw/vehicle_profiles/' in name and name.endswith('.json'):
                        if '/docs/' not in name and not Path(name).stem.startswith('.'):
                            wican_jsons.append(name)
                    elif 'ev-obd-pids/' in name and name.endswith('.json') and 'obdble_cars' not in name:
                        ev_obd_jsons.append(name)

                for name in torque_csvs:
                    self._parse_torque_csv(zf, name)
                for name in wican_jsons:
                    self._parse_wican_json(zf, name)
                for name in ev_obd_jsons:
                    self._parse_ev_obd_json(zf, name)

            self._build_vehicle_index()
            self._loaded = True
            logger.info(f"EV data loaded: {len(self._all_params)} parameters, {len(self._vehicle_index)} vehicles")
            return True
        except Exception as e:
            logger.error(f"Failed to load EV data: {e}")
            return False

    def get_params_for_vehicle(self, make: str, model: str = "") -> List[EVParam]:
        """Get EV parameters matching a vehicle make/model."""
        if not self._loaded:
            return []

        # Make aliases (e.g., "Volkswagen" → also search "vw")
        _MAKE_ALIASES = {
            "volkswagen": ["vw", "volkswagen"], "peugeot": ["peugeot", "psa"],
            "citroen": ["citroen", "citroën", "psa"], "opel": ["opel", "vauxhall"],
            "hyundai": ["hyundai", "hkmc"], "kia": ["kia", "hkmc"],
            "bmw": ["bmw", "mini"], "mercedes": ["mercedes", "benz"],
            "renault": ["renault", "dacia"], "nissan": ["nissan", "leaf"],
        }
        make_lower = make.lower()
        model_lower = model.lower()
        search_terms = _MAKE_ALIASES.get(make_lower, [make_lower])
        results = []
        seen_keys = set()

        for keyword, params in self._vehicle_index.items():
            if any(term in keyword for term in search_terms):
                for param in params:
                    key = (param.command, param.name)
                    if key not in seen_keys:
                        if model_lower and model_lower not in keyword:
                            continue
                        results.append(param)
                        seen_keys.add(key)

        return results

    def get_all_vehicles(self) -> List[str]:
        """Get list of all vehicle keywords."""
        return sorted(self._vehicle_index.keys())

    def _parse_torque_csv(self, zf: zipfile.ZipFile, filename: str):
        """Parse Torque Pro CSV format."""
        try:
            # Extract vehicle name from path
            path_parts = Path(filename).parts
            vehicle = ""
            for i, part in enumerate(path_parts):
                if part.lower() in ('obd-pids-for-hkmc-evs', 'hyundai-ioniq-5-torque-pro-pids'):
                    if i + 1 < len(path_parts):
                        vehicle = path_parts[i + 1]
                        break

            if not vehicle:
                vehicle = "Unknown"

            content = zf.read(filename).decode('utf-8', errors='ignore')
            reader = csv.DictReader(io.StringIO(content))

            for row in reader:
                if not row or row.get('Name', '').startswith('~'):
                    continue

                name = row.get('Name', '').strip()
                short_name = row.get('ShortName', '').strip()
                mode_pid = row.get('ModeAndPID', '').strip()
                equation = row.get('Equation', '').strip()
                min_val_str = row.get('Min Value', '0')
                max_val_str = row.get('Max Value', '0')
                unit = row.get('Units', '').strip()
                header = row.get('Header', '7E4').strip()

                if not name or not mode_pid:
                    continue

                # Convert ModeAndPID hex to command
                command = mode_pid.replace('0x', '').upper()

                try:
                    min_val = float(min_val_str)
                    max_val = float(max_val_str)
                except (ValueError, TypeError):
                    min_val = 0.0
                    max_val = 100.0

                param = EVParam(
                    name=name,
                    short_name=short_name or name,
                    command=command,
                    equation=equation,
                    min_val=min_val,
                    max_val=max_val,
                    unit=unit,
                    ecu_header=header,
                    source_file=filename,
                    vehicle=vehicle,
                )
                self._all_params.append(param)
        except Exception as e:
            logger.debug(f"Error parsing Torque CSV {filename}: {e}")

    def _parse_wican_json(self, zf: zipfile.ZipFile, filename: str):
        """Parse WiCAN vehicle profile JSON."""
        try:
            content = zf.read(filename).decode('utf-8', errors='ignore')
            data = json.loads(content)

            car_model = data.get('car_model', 'Unknown')
            init_cmd = data.get('init', '')

            # Extract ECU header from init command (ATSH)
            ecu_header = '7E4'
            for cmd in init_cmd.split(';'):
                cmd = cmd.strip()
                if cmd.startswith('ATSH'):
                    ecu_header = cmd[4:].strip()
                    break

            pids = data.get('pids', [])
            for pid_entry in pids:
                pid_hex = pid_entry.get('pid', '').strip()
                parameters = pid_entry.get('parameters', {})

                if not pid_hex:
                    continue

                # Remove leading zeros and pad to 6 chars
                command = pid_hex.lstrip('0') or '0'
                command = command.upper().zfill(6)

                for param_name, formula in parameters.items():
                    param = EVParam(
                        name=param_name,
                        short_name=param_name,
                        command=command,
                        equation=formula,
                        min_val=0.0,
                        max_val=100.0,
                        unit='',
                        ecu_header=ecu_header,
                        source_file=filename,
                        vehicle=car_model,
                    )
                    self._all_params.append(param)
        except Exception as e:
            logger.debug(f"Error parsing WiCAN JSON {filename}: {e}")

    def _parse_ev_obd_json(self, zf: zipfile.ZipFile, filename: str):
        """Parse ev-obd-pids JSON format."""
        try:
            content = zf.read(filename).decode('utf-8', errors='ignore')
            data = json.loads(content)

            # Extract vehicle name from path
            path_parts = Path(filename).parts
            vehicle = "Unknown"
            for part in path_parts:
                if part.startswith('Hyundai') or part.startswith('Kia') or part.startswith('Volkswagen'):
                    vehicle = part
                    break

            for param_name, param_def in data.items():
                if not isinstance(param_def, dict):
                    continue

                equation = param_def.get('equation', '')
                command = param_def.get('command', '').strip()
                ecu_header = param_def.get('ecu', '7E4').strip()
                min_val_str = param_def.get('minValue', '0')
                max_val_str = param_def.get('maxValue', '100')
                unit = param_def.get('unit', '')

                if not command:
                    continue

                # Normalize command
                command = command.replace('0x', '').upper()

                try:
                    min_val = float(min_val_str)
                    max_val = float(max_val_str)
                except (ValueError, TypeError):
                    min_val = 0.0
                    max_val = 100.0

                param = EVParam(
                    name=param_name,
                    short_name=param_name,
                    command=command,
                    equation=equation,
                    min_val=min_val,
                    max_val=max_val,
                    unit=unit,
                    ecu_header=ecu_header,
                    source_file=filename,
                    vehicle=vehicle,
                )
                self._all_params.append(param)
        except Exception as e:
            logger.debug(f"Error parsing ev-obd-pids JSON {filename}: {e}")

    def _build_vehicle_index(self):
        """Build keyword index for vehicle matching."""
        keyword_map: Dict[str, List] = {}

        for param in self._all_params:
            vehicle_lower = param.vehicle.lower()
            keywords = set()

            # Extract keywords from vehicle name
            for word in vehicle_lower.split():
                word = word.strip('&(),')
                if word and len(word) > 2:
                    keywords.add(word)

            keywords.add(vehicle_lower[:50])

            for keyword in keywords:
                if keyword not in keyword_map:
                    keyword_map[keyword] = []
                keyword_map[keyword].append(param)

        self._vehicle_index = keyword_map
