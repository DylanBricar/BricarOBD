"""Demo/simulation mode for testing without a real OBD adapter."""

import random
import math
import time
import threading

from obd_core.connection import ConnectionState

class DemoConnection:
    """Simulates an ELM327 connection with realistic vehicle data."""

    def __init__(self):
        self.port = "DEMO"
        self.baud_rate = 0
        self.protocol_name = "Simulation (Demo)"
        self.elm_version = "BricarOBD Demo v1.0"
        self._connected = False
        self._start_time = 0
        self.lock = threading.RLock()
        self.state = ConnectionState.DISCONNECTED

        # Simulated vehicle state
        self._rpm = 850  # idle
        self._speed = 0
        self._coolant = 20  # cold start
        self._load = 15
        self._throttle = 0
        self._intake_temp = 25
        self._maf = 3.5
        self._fuel = 75
        self._voltage = 12.6
        self._timing = 10
        self._oil_temp = 20
        self._ambient = 22
        self._vehicle_make = "Peugeot"
        self._vehicle_model = "207"

    def connect(self, port="DEMO", baud_rate=0):
        """Simulate connection."""
        time.sleep(0.5)  # Simulate connection delay
        self._connected = True
        self._start_time = time.time()
        self.state = ConnectionState.CONNECTED
        return True

    def disconnect(self):
        """Simulate disconnection."""
        self._connected = False
        self.state = ConnectionState.DISCONNECTED

    def is_connected(self):
        return self._connected

    def connect_with_retry(self, port="DEMO", baud_rate=0, max_retries=3):
        """Connect with automatic retry on failure."""
        return self.connect(port, baud_rate)

    def send_command(self, cmd, timeout=5):
        """Simulate ELM327 command response."""
        cmd_upper = cmd.strip().upper()
        # AT SH xxx - set CAN header (accept any address)
        if cmd_upper.startswith("AT") or cmd_upper.startswith("ATSH"):
            return "OK"
        # Empty command (used for re-read in NRC 0x78 handling)
        if not cmd_upper:
            return ""
        return "OK"

    def send_obd(self, mode, pid=None, _internal=False):
        """Simulate OBD response with realistic data."""
        if not self._connected:
            return ""
        if mode in ("04",) and not _internal:
            return ""

        self._update_simulation()

        if mode == "01" and pid:
            return self._simulate_mode01(int(pid, 16))
        elif mode == "03":
            return self._simulate_dtcs()
        elif mode == "04":
            return "44"  # DTC clear OK
        elif mode == "07":
            return "47"  # No pending DTCs
        elif mode == "09" and pid:
            return self._simulate_mode09(int(pid, 16))
        elif mode == "0A":
            return "4A"  # No permanent DTCs
        return "NO DATA"

    def send_raw(self, hex_string):
        """Simulate UDS response with manufacturer-specific data."""
        if not self._connected:
            return ""
        import re
        clean = hex_string.replace(" ", "").upper()
        if not clean:
            return ""
        if not re.match(r'^[0-9A-F]+$', clean):
            return ""
        if clean.startswith("AT"):
            return ""

        # UDS Service routing
        service = clean[:2]

        # 0x3E TesterPresent - respond OK for all ECU addresses
        if service == "3E":
            return "7E 00"

        # 0x10 DiagnosticSessionControl
        if service == "10":
            return "50 01"

        # 0x22 ReadDataByIdentifier - simulate various DIDs
        if service == "22" and len(clean) >= 6:
            did = clean[2:6]
            return self._simulate_did_read(did)

        # 0x19 ReadDTCInformation
        if service == "19":
            return "59 02 FF"

        # 0x14 ClearDTC
        if service == "14":
            return "54"

        return "7F " + service + " 31"  # NRC: request out of range

    def _simulate_did_read(self, did_hex: str) -> str:
        """Simulate UDS DID read responses for a Peugeot 207.

        Args:
            did_hex: 4-char hex DID (e.g., "F190")

        Returns:
            Simulated positive response
        """
        # VIN
        vin_hex = "56 46 33 57 43 39 48 58 43 31 53 31 32 33 34 35 36"
        did_responses = {
            "F190": f"62 F1 90 {vin_hex}",  # VIN
            "F18C": "62 F1 8C 50 53 41 32 30 37 45 43 55 30 31",  # Serial: PSA207ECU01
            "F191": "62 F1 91 48 57 20 56 32 2E 31",  # HW V2.1
            "F189": "62 F1 89 53 57 20 56 34 2E 33 2E 32",  # SW V4.3.2
            "F18B": "62 F1 8B 32 30 32 34 30 35 31 35",  # Mfg date: 20240515
            "F186": "62 F1 86 01",  # Active session: default
            # PSA extended DIDs
            "1100": "62 11 00 25",  # CPU Usage: 37%
            "2100": "62 21 00 5A 3C 1E",  # Gauging group 1
            "2200": "62 22 00 01 02 03",  # Fuel management config
            "F080": "62 F0 80 50 53 41 20 41 45 45 32 30 30 34",  # ZA: PSA AEE2004
        }

        resp = did_responses.get(did_hex.upper())
        if resp:
            return resp
        return "7F 22 31"  # NRC: request out of range

    def _read_until_prompt(self, timeout):
        return ""

    def _update_simulation(self):
        """Update simulated values with realistic variations."""
        t = time.time() - self._start_time

        # Engine warms up over time
        if self._coolant < 90:
            self._coolant = min(90, 20 + t * 0.5)
        else:
            self._coolant = 90 + random.uniform(-2, 2)

        if self._oil_temp < 85:
            self._oil_temp = min(85, 20 + t * 0.3)
        else:
            self._oil_temp = 85 + random.uniform(-2, 2)

        # Simulate driving pattern (sine wave speed)
        drive_phase = math.sin(t * 0.05) * 0.5 + 0.5  # 0 to 1
        self._speed = max(0, drive_phase * 80 + random.uniform(-3, 3))
        self._rpm = max(750, 850 + self._speed * 25 + random.uniform(-50, 50))
        self._load = max(10, 15 + self._speed * 0.5 + random.uniform(-3, 3))
        self._throttle = max(0, min(100, self._speed * 0.4 + random.uniform(-2, 2)))
        self._maf = max(2, 3.5 + self._speed * 0.15 + random.uniform(-0.5, 0.5))
        self._timing = 10 + self._speed * 0.1 + random.uniform(-2, 2)

        # Battery voltage
        self._voltage = 13.8 + random.uniform(-0.3, 0.3) if self._rpm > 1000 else 12.4 + random.uniform(-0.2, 0.2)

        # Fuel consumption
        self._fuel = max(0, self._fuel - 0.0001 * self._load)

        # Ambient stays stable
        self._ambient = 22 + random.uniform(-1, 1)
        self._intake_temp = self._ambient + 5 + self._load * 0.1

    def _simulate_mode01(self, pid):
        """Generate realistic Mode 01 responses."""
        responses = {
            0x01: "41 01 00 07 E5 00",  # Monitor status: no MIL, 3 DTCs
            0x03: "41 03 02 00",        # Fuel status: closed loop
            0x04: self._format_response(0x04, [int(self._load * 255 / 100)]),
            0x05: self._format_response(0x05, [int(self._coolant + 40)]),
            0x06: self._format_response(0x06, [int(128 + random.uniform(-5, 5))]),
            0x07: self._format_response(0x07, [int(128 + random.uniform(-3, 3))]),
            0x0B: self._format_response(0x0B, [int(100 + self._speed * 0.2)]),
            0x0C: self._format_response(0x0C, [int(self._rpm * 4 / 256), int(self._rpm * 4) % 256]),
            0x0D: self._format_response(0x0D, [int(min(255, self._speed))]),
            0x0E: self._format_response(0x0E, [int((self._timing + 64) * 2)]),
            0x0F: self._format_response(0x0F, [int(self._intake_temp + 40)]),
            0x10: self._format_response(0x10, [int(self._maf * 100 / 256), int(self._maf * 100) % 256]),
            0x11: self._format_response(0x11, [int(self._throttle * 255 / 100)]),
            0x1C: "41 1C 06",           # OBD standard: EOBD
            0x1F: self._format_response(0x1F, [int((time.time() - self._start_time) / 256), int(time.time() - self._start_time) % 256]),
            0x2F: self._format_response(0x2F, [int(self._fuel * 255 / 100)]),
            0x33: self._format_response(0x33, [101]),
            0x41: "41 41 00 00 00 00",  # Drive cycle status
            0x42: self._format_response(0x42, [int(self._voltage * 1000 / 256), int(self._voltage * 1000) % 256]),
            0x46: self._format_response(0x46, [int(self._ambient + 40)]),
            0x51: "41 51 01",           # Fuel type: gasoline
            0x5C: self._format_response(0x5C, [int(self._oil_temp + 40)]),
            0x5E: self._format_response(0x5E, [0, int(self._load * 0.3)]),
        }

        # Support PIDs (bit fields showing what's available)
        responses[0x00] = "41 00 BE 3F B8 13"
        responses[0x20] = "41 20 80 15 B0 15"
        responses[0x40] = "41 40 7A 1C 80 00"

        return responses.get(pid, "NO DATA")

    def _simulate_dtcs(self):
        """Return simulated DTCs (one active code for demo)."""
        # P0420 - Catalyst efficiency below threshold
        return "43 01 04 20 00 00"

    def _simulate_mode09(self, info_type):
        """Simulate Mode 09 vehicle info."""
        if info_type == 0x02:
            # VIN: VF3WC9HXC1S123456
            vin_hex = "56 46 33 57 43 39 48 58 43 31 53 31 32 33 34 35 36"
            return f"49 02 01 {vin_hex}"
        return "NO DATA"

    def _format_response(self, pid, data_bytes):
        """Format a Mode 01 response."""
        data_hex = " ".join(f"{b:02X}" for b in data_bytes)
        return f"41 {pid:02X} {data_hex}"

    @staticmethod
    def available_ports():
        """Return demo port."""
        return [{"port": "DEMO", "description": "BricarOBD Demo Mode", "hwid": "DEMO"}]
