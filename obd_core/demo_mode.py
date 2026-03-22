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
        if cmd_upper == "ATRV":
            return "12.6V"
        if cmd_upper.startswith("AT"):
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

        from utils.dev_console import log_obd_command
        cmd = f"{mode}{pid}" if pid else mode
        log_obd_command("TX", f"[DEMO] {cmd}")
        self._update_simulation()

        if mode == "01" and pid:
            return self._simulate_mode01(int(pid, 16))
        elif mode == "02" and pid:
            # Freeze frame — return same as Mode 01 but with "42" prefix
            mode01_resp = self._simulate_mode01(int(pid, 16))
            if mode01_resp != "NO DATA":
                return mode01_resp.replace("41", "42", 1)
            return "NO DATA"
        elif mode == "03":
            return self._simulate_dtcs()
        elif mode == "04":
            return "44"  # DTC clear OK
        elif mode == "07":
            return "47"  # No pending DTCs
        elif mode == "06" and pid:
            return self._simulate_mode06(int(pid, 16))
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

        # 0x27 SecurityAccess (for advanced operations demo)
        if service == "27":
            sub = clean[2:4] if len(clean) >= 4 else "01"
            if sub == "01":
                return "67 01 00 00 00 00"  # Zero seed = already unlocked
            elif sub == "02":
                return "67 02"  # Key accepted

        # 0x31 RoutineControl (for advanced operations demo)
        if service == "31" and len(clean) >= 6:
            sub = clean[2:4]
            rid = clean[4:8]
            if sub == "01":
                return f"71 01 {rid}"  # Routine started OK
            elif sub == "03":
                return f"71 03 {rid} 00"  # Routine completed

        # 0x2E WriteDataByIdentifier (for advanced operations demo)
        if service == "2E" and len(clean) >= 6:
            did = clean[2:6]
            return f"6E {did}"  # Write OK

        # 0x2F IOControlByIdentifier (for advanced operations demo)
        if service == "2F" and len(clean) >= 6:
            did = clean[2:6]
            return f"6F {did} 00"  # Control OK

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
            # Advanced operations DIDs (PSA)
            "2282": "62 22 82 00 4E 20 0F 03 19 00 7D 00",  # Maintenance: 20000km, 15/03/2025, 32000km interval
            "2106": "62 21 06 00 0A 00 32 00 1E 03",  # DPF: 10g soot, 50 regens, 30% load, status OK
            # PSA BSI vehicle config zones
            "231C": "62 23 1C 02",           # Engine type: 02=Diesel
            "231A": "62 23 1A 01",           # Gearbox: 01=Manual
            "232D": "62 23 2D 01",           # DPF type: 01=Additive
            "2333": "62 23 33 02",           # Battery type: 02=AGM
            "2318": "62 23 18 03",           # Climate: 03=Auto bi-zone
            "23BB": "62 23 BB 01",           # Start&Stop: 01=Present
            "232A": "62 23 2A 02",           # DRL type: 02=LED
            # BMW DPF DIDs
            "DA01": "62 DA 01 00 08 00 2A",  # Soot: 8g, 42 regens
            "DA02": "62 DA 02 01 00 50",     # Regen status: active=no, last=80%
            # Mercedes DPF DIDs
            "3001": "62 30 01 00 0C 00 38",  # Soot: 12g, 56 regens
            # Renault K9K 1.5 dCi (from k9k_pids)
            "242C": "62 24 2C 00 6E",        # Soot mass: 1.10g
            "2434": "62 24 34 00",           # Regen status: 0 (idle)
            "2442": "62 24 42 0D 52",        # DPF inlet temp: 68°C
            "2542": "62 25 42 80 0A",        # DPF diff pressure: 10 mbar
            "2481": "62 24 81 2A",           # Successful regen count: 42
            "24A9": "62 24 A9 00 01 2C",     # Dist since regen: 300 km
            "2487": "62 24 87 07 08",        # Last regen duration: 3 min
            "24EC": "62 24 EC 00 00 00 32",  # Oil dilution
            "FD07": "62 FD 07 80 00",        # Fuel corr cyl1
            "FD08": "62 FD 08 80 10",        # Fuel corr cyl2
            "FD09": "62 FD 09 7F F0",        # Fuel corr cyl3
            "FD0A": "62 FD 0A 80 08",        # Fuel corr cyl4
            "2401": "62 24 01 04 4C",        # Boost: 1100 mbar
            "2407": "62 24 07 02 00",        # EGR position: ~25%
            "2801": "62 28 01 05 DC",        # Fuel rail: 1500 bar
            # VAG DIDs
            "F1AD": "62 F1 AD 43 5A 45 41",  # Engine code: CZEA
            "F187": "62 F1 87 30 34 45 39 30 36 30 32 37 44",  # Part: 04E906027D
            "0600": "62 06 00 00 04 00 01",  # Coding value
            "295A": "62 29 5A 00 01 A4 38",  # Mileage: 107576 km
            "F442": "62 F4 42 35 B0",        # Voltage: 13.744V
            # Hyundai/Kia BMS
            "0105": "62 01 05 52",           # SOC: 82%
            "0101": "62 01 01 01 90",        # Voltage: 400V
            "0104": "62 01 04 1E",           # Temp: 30°C
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

    def _simulate_mode06(self, mid):
        """Simulate Mode 06 test results (monitoring)."""
        # Format: 46 MID 00 VAL_HI VAL_LO MIN MAX
        test_results = {
            # Oxygen sensors
            0x01: "46 01 00 00 B2 00 FF",  # O2 B1S1 — val=178, pass
            0x02: "46 02 00 00 9A 00 FF",  # O2 B1S2 — val=154, pass
            0x03: "46 03 00 00 C1 00 FF",  # O2 B2S1 — val=193, pass
            0x04: "46 04 00 00 A5 00 FF",  # O2 B2S2 — val=165, pass
            0x05: "46 05 00 00 88 00 FF",  # O2 B1S3 — val=136, pass
            0x06: "46 06 00 00 7F 00 FF",  # O2 B1S4 — val=127, pass
            0x07: "46 07 00 00 D3 00 FF",  # O2 B2S3 — val=211, pass
            0x08: "46 08 00 00 90 00 FF",  # O2 B2S4 — val=144, pass
            # Catalyst
            0x09: "46 09 00 00 6E 00 FF",  # Catalyst B1 — val=110, pass
            0x0A: "46 0A 00 00 72 00 FF",  # Catalyst B2 — val=114, pass
            # Heated catalyst
            0x0B: "46 0B 00 00 A0 00 FF",  # Heated Cat B1 — val=160, pass
            0x0C: "46 0C 00 00 98 00 FF",  # Heated Cat B2 — val=152, pass
            # EVAP
            0x0D: "46 0D 00 00 05 00 19",  # EVAP leak — val=5, pass (max=25)
            0x0E: "46 0E 00 00 42 00 FF",  # EVAP purge — val=66, pass
            # Secondary air
            0x0F: "46 0F 00 00 8C 00 FF",  # Secondary air B1 — pass
            0x10: "46 10 00 00 87 00 FF",  # Secondary air B2 — pass
            # O2 heaters
            0x12: "46 12 00 00 C0 00 FF",  # O2 heater B1S1 — pass
            0x13: "46 13 00 00 B5 00 FF",  # O2 heater B1S2 — pass
            0x14: "46 14 00 00 BB 00 FF",  # O2 heater B2S1 — pass
            0x15: "46 15 00 00 BA 00 FF",  # O2 heater B2S2 — pass
            # EGR
            0x1F: "46 1F 00 00 38 00 FF",  # EGR flow — val=56, pass
            0x20: "46 20 00 00 44 00 FF",  # EGR VVT — val=68, pass
            # Misfire
            0x21: "46 21 00 00 00 00 0A",  # Misfire Cyl1 — val=0, pass (max=10)
            0x22: "46 22 00 00 01 00 0A",  # Misfire Cyl2 — val=1, pass
            0x23: "46 23 00 00 00 00 0A",  # Misfire Cyl3 — val=0, pass
            0x24: "46 24 00 00 02 00 0A",  # Misfire Cyl4 — val=2, pass
            0x27: "46 27 00 00 03 00 14",  # Misfire General — val=3, pass (max=20)
            # Fuel system
            0x31: "46 31 00 00 7D 00 FF",  # Fuel System B1 — val=125, pass
            0x32: "46 32 00 00 80 00 FF",  # Fuel System B2 — val=128, pass
            # DPF
            0x39: "46 39 00 00 1A 00 64",  # PM Filter B1 — val=26, pass (max=100)
            # Boost
            0x3B: "46 3B 00 00 55 00 FF",  # Boost pressure — val=85, pass
        }
        return test_results.get(mid, "NO DATA")

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
