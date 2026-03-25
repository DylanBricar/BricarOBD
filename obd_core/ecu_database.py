from dataclasses import dataclass
from typing import List, Dict, Tuple, Optional


@dataclass
class ECUInfo:
    name: str
    request_id: int
    response_id: int
    protocol: str
    description: str


@dataclass
class VehicleProfile:
    make: str
    model: str
    year_range: Tuple[int, int]
    ecus: List[ECUInfo]
    notes: str


GENERIC_ECUS: List[ECUInfo] = [
    ECUInfo(
        name="Engine Control Module",
        request_id=0x7E0,
        response_id=0x7E8,
        protocol="OBD-II",
        description="Primary engine control unit"
    ),
    ECUInfo(
        name="Transmission Control Module",
        request_id=0x7E1,
        response_id=0x7E9,
        protocol="OBD-II",
        description="Transmission control unit"
    ),
    ECUInfo(
        name="Anti-Lock Brake System",
        request_id=0x7E2,
        response_id=0x7EA,
        protocol="OBD-II",
        description="ABS control module"
    ),
    ECUInfo(
        name="Airbag Control Module",
        request_id=0x7E3,
        response_id=0x7EB,
        protocol="OBD-II",
        description="Airbag/SRS control unit"
    ),
    ECUInfo(
        name="Body Control Module",
        request_id=0x7E4,
        response_id=0x7EC,
        protocol="OBD-II",
        description="Body electrical control module"
    ),
    ECUInfo(
        name="Instrument Cluster",
        request_id=0x7E5,
        response_id=0x7ED,
        protocol="OBD-II",
        description="Instrument cluster module"
    ),
    ECUInfo(
        name="Climate Control Module",
        request_id=0x7E6,
        response_id=0x7EE,
        protocol="OBD-II",
        description="HVAC control module"
    ),
]


PSA_EXTENDED_ECUS = {
    "BSI": {"request_id": 0x75D, "response_id": 0x65D, "name": "Body Systems Interface", "protocol": "UDS"},
    "INJ": {"request_id": 0x6A8, "response_id": 0x688, "name": "Injection/Engine ECU", "protocol": "UDS"},
    "ABRASR": {"request_id": 0x6AD, "response_id": 0x68D, "name": "ABS/ESP Module", "protocol": "UDS"},
    "CLIM": {"request_id": 0x76D, "response_id": 0x66D, "name": "Climate Control", "protocol": "UDS"},
    "BOITEVIT": {"request_id": 0x6A9, "response_id": 0x689, "name": "Gearbox ECU", "protocol": "UDS"},
    "DIRAV": {"request_id": 0x6B2, "response_id": 0x692, "name": "Power Steering", "protocol": "UDS"},
    "AIRBAG": {"request_id": 0x772, "response_id": 0x672, "name": "Airbag Module", "protocol": "UDS"},
    "AFFICHEUR": {"request_id": 0x765, "response_id": 0x665, "name": "Display Unit", "protocol": "UDS"},
    "COMMANDES_BSI": {"request_id": 0x773, "response_id": 0x673, "name": "BSI Commands", "protocol": "UDS"},
    "TELEMATIQUE": {"request_id": 0x764, "response_id": 0x664, "name": "Telematics Unit", "protocol": "UDS"},
    "COMBINE": {"request_id": 0x734, "response_id": 0x714, "name": "Instrument Cluster", "protocol": "UDS"},
    "RADIO": {"request_id": 0x7A0, "response_id": 0x6A0, "name": "Radio/Audio", "protocol": "UDS"},
    "PARKING": {"request_id": 0x762, "response_id": 0x662, "name": "Parking Sensors", "protocol": "UDS"},
}


PSA_EXTENDED_DIDS = {
    # Identification zones
    0xF190: "VIN (Vehicle Identification Number)",
    0xF18C: "ECU Serial Number",
    0xF18B: "Manufacturing Date",
    0xF080: "ZA Zone (Identification)",
    0xF0FE: "ZI Zone (Calibration Info)",
    0x0901: "ECU Software Reference",

    # Diagnostic data
    0x1100: "CPU Usage",
    0x2100: "Gauging Group 1",
    0x2101: "Gauging Group 2",
    0x2102: "Gauging Group 3",
    0x2103: "Gauging Group 4",
    0x2104: "Gauging Group 5",
    0x2105: "Gauging Group 6",
    0x2106: "Gauging Group 7",

    # Vehicle Configuration
    0x2200: "Fuel Management Config",
    0x2201: "Oil Management Config",
    0x2202: "Engine Parameters 1",
    0x2203: "Engine Parameters 2",
    0x2204: "Engine Parameters 3",
    0x2280: "Maintenance Schedule 1",
    0x2281: "Maintenance Schedule 2",
    0x2282: "Maintenance Schedule 3",

    # Group data
    0xD400: "Module Group Presence",
    0xD401: "Module Visibility Settings",
    0xD402: "Module Active Config",

    # BSI specific
    0x2901: "Secured Traceability Zone",
    0x2430: "BSI Configuration 1",
    0x2431: "BSI Configuration 2",
    0x2432: "BSI Configuration 3",
    0x2433: "BSI Configuration 4",
    0x2434: "BSI Coding Parameters",
    0x2435: "BSI Network Config",
    0x2436: "BSI Security Config",
    0x2437: "BSI Options Config",

    # PSA BSI zones (source: ludwig-v/arduino-psa-diag)
    0x0100: "Factory data list",
    0x0101: "Factory data values",
    0x0102: "Factory byte 1",
    0x0103: "Factory byte 2",
    0x0104: "Factory byte 3",
    0x0105: "Factory byte 4",
    0x0106: "Factory byte 5",
    0x0107: "Factory byte 6",
    0x0108: "Factory Information",
    0x0109: "Vehicle Electronic Integration Mode",
    0x010A: "BSI Sleeping mode",
    0x010B: "Inviolability Forcing",
    0x010C: "After Ignition State Signal",
    0x010D: "Window Wipers Control",
    0x2205: "Gauging table (Oil)",
    0x2206: "Oil level origin",
    0x2207: "Oil pressure origin",
    0x220A: "Critical fuel level",
    0x2283: "First Maintenance Threshold",
    0x2284: "Total time before first maintenance",
    0x2289: "Maintenance modified by Engine ECU",
    0x2300: "Parking assistance type (AAS / SAM / CPK)",
    0x2301: "BSG presence (Trailer ECU)",
    0x2302: "Parking aid (AAS) sound to COM200X",
    0x2303: "Visual parking assistance (AAS)",
    0x2304: "Airbag inhibition key",
    0x2305: "Alarm mounting type",
    0x2306: "Alarm volumetric detection inhibition",
    0x2307: "Presence and type of sunroof",
    0x2308: "Right-hand drive",
    0x2309: "Alarm type",
    0x230A: "Key type (Mechanic / Electronic / ADML)",
    0x230B: "Number of contactors for seat-belt warning",
    0x230C: "Detection slick presence for front seats",
    0x230D: "TNB presence over LIN (Seat-Belt warning visual bar)",
    0x230E: "Driver detection inhibition",
    0x230F: "Front passenger detection inhibition",
    0x2310: "Rear Central passenger detection inhibition",
    0x2311: "Rear Right passenger detection inhibition",
    0x2312: "Rear Left passenger detection inhibition",
    0x2314: "EMF presence (Multifunction display)",
    0x2315: "Matrix presence (MATT)",
    0x2317: "Pseudo Fuel Consumption",
    0x2318: "Climate type",
    0x231A: "Gearbox type",
    0x231B: "Vehicle starting type",
    0x231C: "Direct Under inflation detection (DSG) presence",
    0x2321: "Driving-school car",
    0x2323: "Front fog lights",
    0x2325: "Front lighting",
    0x2326: "Smart Beam lighting (CAFR)",
    0x2327: "Welcome / Ambiance lighting",
    0x232A: "Daytime running lights type",
    0x232D: "DPF type (FAP)",
    0x232F: "Passenger lock presence",
    0x2330: "Window closing by remote key",
    0x2333: "Battery type",
    0x2334: "Battery estimation type (SOC)",
    0x233A: "Amplifier presence",
    0x2341: "VTH presence",
    0x2343: "Parking lighting (lateral)",
    0x2344: "Dynamic control (CDS) / ESP presence",
    0x2346: "Water detection in diesel fuel",
    0x2348: "Speed limit (LVV) / Cruise speed (RVV) / Adapative cruise (ACC) presence",
    0x2349: "Overspeed alert",
    0x234A: "2 Multiplexed front power windows",
    0x234B: "4 Multiplexed power windows",
    0x234D: "Powered rearview mirrors",
    0x2350: "Vehicle type (Silhouette)",
    0x2351: "Child safety type",
    0x2352: "Doors locking type",
    0x2353: "Selective doors locking type",
    0x2354: "Key 3rd button functionality",
    0x2357: "Key 3rd button press type",
    0x2361: "Aeration function presence",
    0x2365: "Thatcham mode",
    0x2366: "Service module (MDS) / Autonomous Telematic Unit (BTA) presence",
    0x2367: "Rear fog lights",
    0x2368: "Rear right fog light",
    0x2369: "Rear left fog light",
    0x236A: "Left Reverse light",
    0x236B: "Right Reverse lights",
    0x236C: "Multifunction camera (CMF) presence",
    0x236D: "Rain sensor (CDPL) over LIN",
    0x236E: "Outdoor side lights",
    0x2370: "GSI Function presence",
    0x2371: "Simple driver electric seat presence",
    0x2372: "Simple passanger electric seat presence",
    0x2373: "Driving aid type (AFIL / SAM)",
    0x2375: "Hill Assist presence",
    0x2376: "Blown air probe type",
    0x2377: "Sunshine probe type",
    0x2379: "Pollutants sensor presence",
    0x237B: "Temperature sensors for air conditioning evaporator",
    0x237C: "Air conditioning pressure sensor",
    0x237D: "Air conditioning compressor type",
    0x237F: "Controlled air inlet",
    0x2380: "Front air mixing type (mono / bi-zone)",
    0x2381: "Air distribution type",
    0x2387: "Air conditioning type",
    0x2388: "Air conditioning with no traction",
    0x2389: "Air conditioning compressor entrainment ratio",
    0x238B: "Thermal combustion degradation strategy presence",
    0x238C: "Thermal preconditioning (Webasto) presence and type",
    0x238F: "Police mode",
    0x2392: "FMUX presence and type",
    0x2394: "Radio type",
    0x2395: "Telematic unit type (RNEG/SMEG/MRN/NAC/RCC)",
    0x239C: "Black panel mode",
    0x239D: "CTP presence and type",
    0x239E: "Air heater presence and type",
    0x23A0: "Velum presence and type",
    0x23A2: "Thermal comfort with Start&Stop",
    0x23A6: "Selective rear opening from key",
    0x23A9: "Electric child safety push type",
    0x23AB: "COM2008 sound type presence",
    0x23AE: "Intersection lighting",
    0x23B1: "Start&Stop function inside Air Conditioning",
    0x23B2: "Controlled air inlet module presence",
    0x23B5: "Sky roof without powered velum",
    0x23B6: "MUS over LIN",
    0x23B7: "SIRENE over LIN",
    0x23BA: "Steering wheel angle sensor (CAV)",
    0x23BB: "Start&Stop presence and type",
    0x23C0: "Programmed speeds on touchscreen",
    0x23C8: "Indirect Under inflation (DSG) push type (BSI / Touchscreen / Instrument panel)",
    0x23CC: "Front door locks and rear opening type (Superlock)",
    0x23CD: "Blower load shedding",
    0x23CE: "Gas Vehicle (GNV / GPL)",
    0x23CF: "Wired air conditioning control panel",
    0x23D0: "SCR (Selective Catalyst Reduction) presence",
    0x23D1: "Rising check Alarm presence",
    0x23D3: "Air flow type",
    0x23D4: "Rear windows wipers inhibition type",
    0x23D5: "Left command push type (COM200X) - Daytime running lights, buzzer, page, telematic",
    0x23D7: "Economic cleaning setting",
    0x23D8: "Electric vacuum pump presence",
    0x23DB: "Taxi mode",
    0x23DC: "ESP new regulation type",
    0x23E2: "Heated windshield",
    0x23E3: "ON/OFF Radio push type (FMUX)",
    0x23E4: "Standard alternator (not piloted)",
    0x23E5: "Piloted alternator management type",
    0x23E6: "Low pressure fuel pump",
    0x23E8: "Heated and ventilated seats presence",
    0x23E9: "Vacuum optimization type",
    0x23EA: "Automatic headlights setting",
    0x23EC: "AAS inhibition push type (Touchscreen, Instrument Panel)",
    0x23F6: "Overspeed sound alert (LVV)",
    0x23F9: "EURO Generation (4/5/6/6.1/6.2/6.3)",
    0x23FC: "ASR PLUS presence",
    0x23FD: "Automatic Emergency Brake type (FARC)",
    0x23FE: "Air conditioning HMI acquisition",
    0x23FF: "Front position lamps lighting type with daytime running lights (Always-on front/rear lights)",
    0x2407: "Ambiance rheostat push type (Instrument Panel / Menu / BSI)",
    0x240D: "PLV function presence",
    0x240E: "Alarm architecture type",
    0x240F: "Telematic settings (SOS/Assistance/Telediag)",
    0x2410: "Paddle shifters",
    0x2414: "Driver Attention Assist",
    0x2415: "Speed limit information presence and type",
    0x2425: "CRT presence",
    0x2427: "Electronic hill descent (HADC) presence",
    0x242A: "Forward collision warning (ARC)",
    0x242B: "Rear doors locks type",
    0x243A: "Hydraulic brake assist",
    0x2446: "Additional rear position light",
    0x2448: "Heated steering wheel",
    0x245C: "Voltage regulator (DMTR) read presence",
    0x247C: "Diagnostic algorithm version for brake sensor",
}


VAG_MEASURING_BLOCKS = {
    # Engine data groups (used with service 0x22 or VAG-specific readouts)
    "001": {"name": "Basic Engine Data", "fields": ["RPM", "Coolant Temp", "Lambda", "Residual"]},
    "002": {"name": "Idle/Load Data", "fields": ["Idle Speed", "Engine Load", "Injection Time", "Air Mass"]},
    "003": {"name": "Mixture Control", "fields": ["EGR", "Air Mass Actual", "Air Mass Specified", "Throttle"]},
    "010": {"name": "Ignition", "fields": ["RPM", "Ignition Angle", "Knock Sensor 1", "Knock Retard"]},
    "011": {"name": "Ignition Cyl 1-4", "fields": ["Knock Cyl 1", "Knock Cyl 2", "Knock Cyl 3", "Knock Cyl 4"]},
    "020": {"name": "Knock Control 1", "fields": ["RPM", "Knock Sensor 1", "Knock Retard Cyl 1", "Knock Retard Cyl 2"]},
    "030": {"name": "Lambda Control B1", "fields": ["Lambda Adapt B1", "Lambda Voltage B1", "Lambda Status", "Lambda Corr"]},
    "031": {"name": "Lambda Control B2", "fields": ["Lambda Adapt B2", "Lambda Voltage B2", "Lambda Status", "Lambda Corr"]},
    "050": {"name": "Speed Control", "fields": ["RPM Target", "RPM Actual", "Throttle Duty", "Idle Switch"]},
    "060": {"name": "Throttle Control", "fields": ["Throttle Angle Req", "Throttle Angle Actual", "Accelerator Pos", "Throttle Motor"]},
    "070": {"name": "EVAP System", "fields": ["EVAP Duty", "EVAP Valve Pos", "Tank Pressure", "Purge Valve"]},
    "080": {"name": "ECU Identification", "fields": ["ECU Part Number", "SW Version", "HW Version", "Coding"]},
    "101": {"name": "Fuel Injection 1", "fields": ["Injection Qty", "Rail Pressure Actual", "Rail Pressure Target", "Fuel Temp"]},
    "110": {"name": "Boost Pressure", "fields": ["Boost Actual", "Boost Target", "Boost Duty", "Wastegate Pos"]},
    "115": {"name": "Turbo Data", "fields": ["Turbo Speed", "Turbo Temp In", "Turbo Temp Out", "Intercooler Temp"]},
    "120": {"name": "CAN Communication", "fields": ["CAN Status", "Torque Request", "Torque Actual", "CAN Error"]},
    "130": {"name": "Cooling System", "fields": ["Coolant Temp", "Fan Speed", "Thermostat Duty", "Coolant Flow"]},
}


BMW_EXTENDED_SERVICES = {
    0x21: {"name": "Read Data by Local ID (1 byte)", "description": "BMW proprietary extended PID read"},
    0x22: {"name": "Read Data by Identifier (2 bytes)", "description": "Standard UDS read - used by BMW for extended data"},
    0x23: {"name": "Read Memory by Address (3 bytes)", "description": "BMW memory read service"},
}


BMW_COMMON_DIDS = {
    0xD100: "Boost Pressure (mbar)",
    0xD101: "Charge Air Temperature (°C)",
    0xD901: "Oil Temperature (°C)",
    0xD902: "Oil Pressure (bar)",
    0xDA01: "DPF Soot Loading (%)",
    0xDA02: "DPF Regeneration Status",
    0xDA10: "Current Gear",
    0xDA11: "Transmission Temperature (°C)",
    0xDC01: "Ambient Temperature (°C)",
}


RENAULT_EXTENDED_DIDS = {
    0x21A1: "Rail Pressure (bar, scaling x0.1)",
    0x21B0: "Turbo Boost Pressure (mbar)",
    0x21B1: "Exhaust Gas Temperature Pre-Turbo (°C)",
    0x21B2: "Exhaust Gas Temperature Post-Turbo (°C)",
    0x21C0: "DPF Differential Pressure (mbar)",
    0x21C1: "DPF Soot Loading (%)",
    0x21D0: "EGR Position (%)",
    0x21D1: "EGR Temperature (°C)",
    0x2200: "Battery Voltage (V)",
    0x2201: "Alternator Load (%)",
}


# ── MERCEDES-BENZ EXTENDED ─────────────────────────────────────
MERCEDES_EXTENDED_ECUS = {
    "ME_SFI": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Engine ECU (ME-SFI/CDI)", "protocol": "UDS"},
    "EGS": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission (EGS)", "protocol": "UDS"},
    "ESP": {"request_id": 0x7E2, "response_id": 0x7EA, "name": "ESP/ABS Module", "protocol": "UDS"},
    "SRS": {"request_id": 0x7E3, "response_id": 0x7EB, "name": "Airbag (SRS)", "protocol": "UDS"},
    "SAM_F": {"request_id": 0x740, "response_id": 0x640, "name": "SAM Front (Fuse Box)", "protocol": "UDS"},
    "SAM_R": {"request_id": 0x741, "response_id": 0x641, "name": "SAM Rear", "protocol": "UDS"},
    "IC": {"request_id": 0x743, "response_id": 0x643, "name": "Instrument Cluster", "protocol": "UDS"},
    "EZS": {"request_id": 0x744, "response_id": 0x644, "name": "Electronic Ignition (EZS)", "protocol": "UDS"},
    "HVAC": {"request_id": 0x742, "response_id": 0x642, "name": "Climate Control (HVAC)", "protocol": "UDS"},
    "PARKTRONIC": {"request_id": 0x74E, "response_id": 0x64E, "name": "Parktronic", "protocol": "UDS"},
    "COMAND": {"request_id": 0x74F, "response_id": 0x64F, "name": "COMAND/Audio", "protocol": "UDS"},
    "EPS": {"request_id": 0x746, "response_id": 0x646, "name": "Electric Power Steering", "protocol": "UDS"},
}

MERCEDES_EXTENDED_DIDS = {
    # Engine monitoring (READ-ONLY)
    0x1001: "Engine Speed (RPM)",
    0x1002: "Engine Coolant Temperature (°C)",
    0x1003: "Engine Oil Temperature (°C)",
    0x1004: "Engine Oil Pressure (bar)",
    0x1005: "Intake Air Temperature (°C)",
    0x1006: "Mass Air Flow (g/s)",
    0x1007: "Throttle Position (%)",
    0x1008: "Boost Pressure (mbar)",
    0x1009: "Exhaust Gas Temperature Pre-Cat (°C)",
    0x100A: "Exhaust Gas Temperature Post-Cat (°C)",
    0x100B: "Fuel Rail Pressure (bar)",
    0x100C: "Lambda Sensor Voltage (mV)",
    0x100D: "Injection Quantity (mg/stroke)",
    0x100E: "Turbo Boost Actual (mbar)",
    0x100F: "Turbo Boost Target (mbar)",
    # Transmission monitoring
    0x2001: "Transmission Fluid Temperature (°C)",
    0x2002: "Current Gear",
    0x2003: "Torque Converter Lockup Status",
    0x2004: "Transmission Input Speed (RPM)",
    0x2005: "Transmission Output Speed (RPM)",
    # DPF monitoring
    0x3001: "DPF Soot Loading (%)",
    0x3002: "DPF Differential Pressure (mbar)",
    0x3003: "DPF Temperature Pre (°C)",
    0x3004: "DPF Temperature Post (°C)",
    0x3005: "DPF Regeneration Status",
    0x3006: "DPF Distance Since Last Regen (km)",
    # Standard identification
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
    0xF191: "ECU Hardware Version",
    0xF189: "ECU Software Version",
}

# ── FORD EXTENDED ──────────────────────────────────────────────
FORD_EXTENDED_ECUS = {
    "PCM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Powertrain Control (PCM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission Control (TCM)", "protocol": "UDS"},
    "ABS": {"request_id": 0x760, "response_id": 0x660, "name": "ABS Module", "protocol": "UDS"},
    "RCM": {"request_id": 0x737, "response_id": 0x637, "name": "Restraint Control (Airbag)", "protocol": "UDS"},
    "BCM": {"request_id": 0x726, "response_id": 0x626, "name": "Body Control Module", "protocol": "UDS"},
    "IPC": {"request_id": 0x720, "response_id": 0x620, "name": "Instrument Panel Cluster", "protocol": "UDS"},
    "APIM": {"request_id": 0x7D0, "response_id": 0x7D8, "name": "SYNC/APIM Module", "protocol": "UDS"},
    "PSCM": {"request_id": 0x730, "response_id": 0x630, "name": "Power Steering", "protocol": "UDS"},
    "PAM": {"request_id": 0x736, "response_id": 0x636, "name": "Parking Aid Module", "protocol": "UDS"},
    "HVAC": {"request_id": 0x733, "response_id": 0x633, "name": "Climate Control", "protocol": "UDS"},
}

FORD_EXTENDED_DIDS = {
    # Engine monitoring (READ-ONLY)
    0x1001: "Engine Speed (RPM)",
    0x1002: "Coolant Temperature (°C)",
    0x1003: "Intake Air Temperature (°C)",
    0x1004: "Mass Air Flow (g/s)",
    0x1005: "Throttle Position (%)",
    0x1006: "Engine Oil Temperature (°C)",
    0x1007: "Engine Oil Life (%)",
    0x1008: "Fuel Rail Pressure (kPa)",
    0x1009: "Boost Pressure (kPa)",
    0x100A: "Exhaust Gas Temperature (°C)",
    # Transmission
    0x2001: "Transmission Fluid Temperature (°C)",
    0x2002: "Current Gear",
    0x2003: "Torque Converter Slip (RPM)",
    # Battery / Electrical
    0x4001: "Battery Voltage (V)",
    0x4002: "Battery State of Charge (%)",
    0x4003: "Alternator Output (V)",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
    0xF191: "ECU Hardware Version",
    0xF189: "ECU Software Version",
}

# ── TOYOTA/LEXUS EXTENDED ──────────────────────────────────────
TOYOTA_EXTENDED_ECUS = {
    "ECM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Engine Control (ECM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission Control (TCM)", "protocol": "UDS"},
    "VSC": {"request_id": 0x7B0, "response_id": 0x7B8, "name": "Vehicle Stability Control", "protocol": "UDS"},
    "SRS": {"request_id": 0x7B1, "response_id": 0x7B9, "name": "Airbag (SRS)", "protocol": "UDS"},
    "BODY": {"request_id": 0x750, "response_id": 0x758, "name": "Body ECU", "protocol": "UDS"},
    "METER": {"request_id": 0x7C0, "response_id": 0x7C8, "name": "Combination Meter", "protocol": "UDS"},
    "AC": {"request_id": 0x7C4, "response_id": 0x7CC, "name": "Air Conditioning", "protocol": "UDS"},
    "EPS": {"request_id": 0x7A0, "response_id": 0x7A8, "name": "Electric Power Steering", "protocol": "UDS"},
    "GATEWAY": {"request_id": 0x750, "response_id": 0x758, "name": "Gateway ECU", "protocol": "UDS"},
}

TOYOTA_EXTENDED_DIDS = {
    # Mode 21 (Toyota proprietary - READ-ONLY)
    0x2101: "Engine RPM (extended)",
    0x2102: "Coolant Temperature (extended)",
    0x2103: "Vehicle Speed (extended)",
    0x2104: "Engine Load (extended)",
    0x2105: "Intake Air Temperature (extended)",
    0x2106: "Throttle Position (extended)",
    0x2107: "Fuel System Status",
    0x2108: "Ignition Timing (°BTDC)",
    0x2109: "Mass Air Flow (g/s)",
    0x210A: "Fuel Pressure (kPa)",
    0x210B: "Manifold Pressure (kPa)",
    0x210C: "Battery Voltage (V)",
    0x210D: "A/C Compressor Status",
    0x210E: "A/C Pressure (kPa)",
    0x210F: "Oil Temperature (°C)",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}

# ── HONDA/ACURA EXTENDED ───────────────────────────────────────
HONDA_EXTENDED_ECUS = {
    "PCM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Powertrain Control (PCM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission Control", "protocol": "UDS"},
    "VSA": {"request_id": 0x7B0, "response_id": 0x7B8, "name": "Vehicle Stability Assist", "protocol": "UDS"},
    "SRS": {"request_id": 0x7B2, "response_id": 0x7BA, "name": "Airbag (SRS)", "protocol": "UDS"},
    "MICU": {"request_id": 0x740, "response_id": 0x748, "name": "Multiplex ICU (Body)", "protocol": "UDS"},
    "GAUGE": {"request_id": 0x7C0, "response_id": 0x7C8, "name": "Gauge Assembly", "protocol": "UDS"},
    "EPS": {"request_id": 0x7A0, "response_id": 0x7A8, "name": "Electric Power Steering", "protocol": "UDS"},
    "HVAC": {"request_id": 0x7C4, "response_id": 0x7CC, "name": "Climate Control", "protocol": "UDS"},
}

HONDA_EXTENDED_DIDS = {
    # Honda Mode 21 (proprietary - READ-ONLY)
    0x2101: "Engine Speed (RPM)",
    0x2102: "Coolant Temperature (°C)",
    0x2103: "Vehicle Speed (km/h)",
    0x2104: "Battery Voltage (V)",
    0x2105: "Intake Air Temperature (°C)",
    0x2106: "Throttle Position (%)",
    0x2107: "MAP Sensor (kPa)",
    0x2108: "Ignition Timing (°BTDC)",
    0x2109: "VTEC Solenoid Status",
    0x210A: "A/F Ratio",
    0x210B: "Short Term Fuel Trim (%)",
    0x210C: "Long Term Fuel Trim (%)",
    0x210D: "Oil Pressure Warning",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}

# ── HYUNDAI/KIA EXTENDED ───────────────────────────────────────
HYUNDAI_KIA_EXTENDED_ECUS = {
    "ECM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Engine Control (ECM)", "protocol": "UDS"},
    "TCU": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission Control (TCU)", "protocol": "UDS"},
    "ESC": {"request_id": 0x7D1, "response_id": 0x7D9, "name": "Electronic Stability Control", "protocol": "UDS"},
    "ACU": {"request_id": 0x7D2, "response_id": 0x7DA, "name": "Airbag Control Unit", "protocol": "UDS"},
    "BCM": {"request_id": 0x7A0, "response_id": 0x7A8, "name": "Body Control Module", "protocol": "UDS"},
    "CLU": {"request_id": 0x7C6, "response_id": 0x7CE, "name": "Cluster (Instrument)", "protocol": "UDS"},
    "FATC": {"request_id": 0x7B3, "response_id": 0x7BB, "name": "Climate Control (FATC)", "protocol": "UDS"},
    "MDPS": {"request_id": 0x7D4, "response_id": 0x7DC, "name": "Power Steering (MDPS)", "protocol": "UDS"},
    "TPMS": {"request_id": 0x7A5, "response_id": 0x7AD, "name": "Tire Pressure Monitor", "protocol": "UDS"},
    "AVN": {"request_id": 0x7C5, "response_id": 0x7CD, "name": "Audio/Navigation", "protocol": "UDS"},
}

HYUNDAI_KIA_EXTENDED_DIDS = {
    # Engine monitoring (READ-ONLY)
    0x0101: "Engine Speed (RPM)",
    0x0102: "Coolant Temperature (°C)",
    0x0103: "Intake Air Temperature (°C)",
    0x0104: "Manifold Absolute Pressure (kPa)",
    0x0105: "Throttle Position (%)",
    0x0106: "Mass Air Flow (g/s)",
    0x0107: "Vehicle Speed (km/h)",
    0x0108: "Ignition Timing (°BTDC)",
    0x0109: "Engine Oil Temperature (°C)",
    0x010A: "Engine Oil Pressure (bar)",
    0x010B: "Fuel Rail Pressure (bar)",
    0x010C: "Boost Pressure Actual (kPa)",
    0x010D: "Boost Pressure Target (kPa)",
    0x010E: "EGR Valve Position (%)",
    0x010F: "Battery Voltage (V)",
    # EV-specific (Hyundai Ioniq, Kia EV6, etc.)
    0x2101: "HV Battery SOC (%)",
    0x2102: "HV Battery Voltage (V)",
    0x2103: "HV Battery Current (A)",
    0x2104: "HV Battery Temperature (°C)",
    0x2105: "Motor Speed (RPM)",
    0x2106: "Motor Torque (Nm)",
    0x2107: "Charging Status",
    0x2108: "DC Fast Charge Power (kW)",
    0x2109: "Battery Cell Voltage Min (V)",
    0x210A: "Battery Cell Voltage Max (V)",
    0x210B: "Battery SOH (%)",
    0x210C: "Odometer (km)",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}

# ── FIAT/ALFA ROMEO/LANCIA (FCA/STELLANTIS) ───────────────────
FIAT_EXTENDED_ECUS = {
    "ECM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Engine ECU (ECM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission (TCM)", "protocol": "UDS"},
    "ABS_ESP": {"request_id": 0x7E2, "response_id": 0x7EA, "name": "ABS/ESP Module", "protocol": "UDS"},
    "ACM": {"request_id": 0x744, "response_id": 0x644, "name": "Airbag Control (ACM)", "protocol": "UDS"},
    "BCM": {"request_id": 0x740, "response_id": 0x648, "name": "Body Computer (BCM)", "protocol": "UDS"},
    "IPC": {"request_id": 0x742, "response_id": 0x64A, "name": "Instrument Panel", "protocol": "UDS"},
    "HVAC": {"request_id": 0x743, "response_id": 0x64B, "name": "Climate Control", "protocol": "UDS"},
    "EPS": {"request_id": 0x746, "response_id": 0x64E, "name": "Power Steering (EPS)", "protocol": "UDS"},
    "RADIO": {"request_id": 0x747, "response_id": 0x64F, "name": "Radio/Uconnect", "protocol": "UDS"},
    "PDC": {"request_id": 0x748, "response_id": 0x650, "name": "Parking Distance Control", "protocol": "UDS"},
}

FIAT_EXTENDED_DIDS = {
    # Engine monitoring (READ-ONLY)
    0x1001: "Engine Speed (RPM)",
    0x1002: "Coolant Temperature (°C)",
    0x1003: "Intake Air Temperature (°C)",
    0x1004: "Engine Oil Temperature (°C)",
    0x1005: "Throttle Position (%)",
    0x1006: "Boost Pressure (mbar)",
    0x1007: "Fuel Rail Pressure (bar)",
    0x1008: "EGR Valve Position (%)",
    0x1009: "Lambda Value",
    0x100A: "Injection Timing (°BTDC)",
    0x100B: "Battery Voltage (V)",
    # MultiAir specific
    0x2001: "MultiAir Solenoid Status",
    0x2002: "MultiAir Oil Temperature (°C)",
    0x2003: "MultiAir Oil Pressure (bar)",
    # DPF
    0x3001: "DPF Soot Loading (%)",
    0x3002: "DPF Differential Pressure (mbar)",
    0x3003: "DPF Regeneration Count",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}

# ── VOLVO EXTENDED ─────────────────────────────────────────────
VOLVO_EXTENDED_ECUS = {
    "ECM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Engine Control (ECM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission (TCM)", "protocol": "UDS"},
    "SRS": {"request_id": 0x720, "response_id": 0x728, "name": "Airbag (SRS)", "protocol": "UDS"},
    "BCM": {"request_id": 0x726, "response_id": 0x72E, "name": "Body Control (CEM)", "protocol": "UDS"},
    "DIM": {"request_id": 0x740, "response_id": 0x748, "name": "Driver Info Module (DIM)", "protocol": "UDS"},
    "ABS": {"request_id": 0x760, "response_id": 0x768, "name": "ABS Module", "protocol": "UDS"},
    "CCM": {"request_id": 0x745, "response_id": 0x74D, "name": "Climate Control (CCM)", "protocol": "UDS"},
    "SWM": {"request_id": 0x750, "response_id": 0x758, "name": "Steering Wheel Module", "protocol": "UDS"},
    "PSM": {"request_id": 0x744, "response_id": 0x74C, "name": "Power Seat Module", "protocol": "UDS"},
}

VOLVO_EXTENDED_DIDS = {
    0x1001: "Engine Speed (RPM)",
    0x1002: "Coolant Temperature (°C)",
    0x1003: "Engine Oil Temperature (°C)",
    0x1004: "Intake Air Temperature (°C)",
    0x1005: "Boost Pressure (kPa)",
    0x1006: "Throttle Position (%)",
    0x1007: "Battery Voltage (V)",
    0x1008: "Engine Oil Level (mm)",
    0x1009: "Fuel Level (%)",
    0x100A: "Ambient Temperature (°C)",
    0x100B: "Transmission Fluid Temperature (°C)",
    # DPF
    0x3001: "DPF Soot Loading (%)",
    0x3002: "DPF Regeneration Status",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}

# ── MAZDA EXTENDED ─────────────────────────────────────────────
MAZDA_EXTENDED_ECUS = {
    "PCM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Powertrain Control (PCM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission Control", "protocol": "UDS"},
    "DSC": {"request_id": 0x7D0, "response_id": 0x7D8, "name": "Dynamic Stability Control", "protocol": "UDS"},
    "SAS": {"request_id": 0x7D2, "response_id": 0x7DA, "name": "Steering Angle Sensor", "protocol": "UDS"},
    "IC": {"request_id": 0x720, "response_id": 0x728, "name": "Instrument Cluster", "protocol": "UDS"},
    "BCM": {"request_id": 0x726, "response_id": 0x72E, "name": "Body Control Module", "protocol": "UDS"},
    "RCM": {"request_id": 0x730, "response_id": 0x738, "name": "Restraint Control (Airbag)", "protocol": "UDS"},
    "EPS": {"request_id": 0x740, "response_id": 0x748, "name": "Electric Power Steering", "protocol": "UDS"},
}

MAZDA_EXTENDED_DIDS = {
    0x1001: "Engine Speed (RPM)",
    0x1002: "Coolant Temperature (°C)",
    0x1003: "Intake Air Temperature (°C)",
    0x1004: "Mass Air Flow (g/s)",
    0x1005: "Throttle Position (%)",
    0x1006: "Battery Voltage (V)",
    0x1007: "Oil Pressure (kPa)",
    0x1008: "Ignition Timing (°BTDC)",
    0x1009: "i-stop Status",
    0x100A: "Skyactiv Compression Ratio",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}

# ── SUBARU EXTENDED (SSM Protocol) ─────────────────────────────
SUBARU_EXTENDED_ECUS = {
    "ECM": {"request_id": 0x7E0, "response_id": 0x7E8, "name": "Engine Control (ECM)", "protocol": "UDS"},
    "TCM": {"request_id": 0x7E1, "response_id": 0x7E9, "name": "Transmission (TCM)", "protocol": "UDS"},
    "VDC": {"request_id": 0x7E2, "response_id": 0x7EA, "name": "Vehicle Dynamic Control", "protocol": "UDS"},
    "SRS": {"request_id": 0x7E3, "response_id": 0x7EB, "name": "Airbag (SRS)", "protocol": "UDS"},
    "BCM": {"request_id": 0x740, "response_id": 0x748, "name": "Body Control Module", "protocol": "UDS"},
    "METER": {"request_id": 0x744, "response_id": 0x74C, "name": "Combination Meter", "protocol": "UDS"},
    "EPS": {"request_id": 0x746, "response_id": 0x74E, "name": "Electric Power Steering", "protocol": "UDS"},
    "HVAC": {"request_id": 0x748, "response_id": 0x750, "name": "Climate Control", "protocol": "UDS"},
}

SUBARU_EXTENDED_DIDS = {
    # SSM-compatible DIDs (READ-ONLY)
    0x0007: "Coolant Temperature (°C)",
    0x0008: "Intake Air Temperature (°C)",
    0x000D: "Engine Speed (RPM)",
    0x000E: "Vehicle Speed (km/h)",
    0x0010: "Throttle Position (%)",
    0x0012: "Battery Voltage (V)",
    0x0015: "Mass Air Flow (g/s)",
    0x0019: "Boost Pressure (psi)",
    0x001A: "Atmospheric Pressure (mmHg)",
    0x001C: "Knock Correction (°)",
    0x001D: "A/F Learning (LTFT)",
    0x001E: "A/F Correction (STFT)",
    0x0020: "Ignition Timing (°BTDC)",
    0x0024: "Coolant Temperature 2 (°C)",
    # Oil monitoring
    0x0046: "Oil Temperature (°C)",
    0x0047: "Oil Pressure (psi)",
    # Turbo specific
    0x0024: "Turbo Boost (psi)",
    0x0025: "Wastegate Duty Cycle (%)",
    0x0026: "Target Boost (psi)",
    # Standard
    0xF190: "VIN",
    0xF18C: "ECU Serial Number",
}


# ── MANUFACTURER → EXTENDED ECU MAPPING ────────────────────────
# Instead of listing every model, we map manufacturer groups to their
# extended ECU dict. The VIN decoder identifies the make, and we
# dynamically build the profile with generic + extended ECUs.

MANUFACTURER_ECU_MAP = {
    # PSA / Stellantis
    "peugeot": PSA_EXTENDED_ECUS,
    "citroën": PSA_EXTENDED_ECUS,
    "citroen": PSA_EXTENDED_ECUS,
    "ds": PSA_EXTENDED_ECUS,
    "opel": PSA_EXTENDED_ECUS,
    "vauxhall": PSA_EXTENDED_ECUS,
    # VAG
    "volkswagen": None,  # VAG uses measuring blocks, not standard ECU addressing
    "audi": None,
    "seat": None,
    "cupra": None,
    "skoda": None,
    "škoda": None,
    "porsche": None,
    # German premium
    "bmw": None,
    "mini": None,
    "mercedes-benz": MERCEDES_EXTENDED_ECUS,
    "mercedes": MERCEDES_EXTENDED_ECUS,
    # Ford
    "ford": FORD_EXTENDED_ECUS,
    "lincoln": FORD_EXTENDED_ECUS,
    # Japanese
    "toyota": TOYOTA_EXTENDED_ECUS,
    "lexus": TOYOTA_EXTENDED_ECUS,
    "honda": HONDA_EXTENDED_ECUS,
    "acura": HONDA_EXTENDED_ECUS,
    "mazda": MAZDA_EXTENDED_ECUS,
    "subaru": SUBARU_EXTENDED_ECUS,
    "nissan": None,
    "infiniti": None,
    # Korean
    "hyundai": HYUNDAI_KIA_EXTENDED_ECUS,
    "kia": HYUNDAI_KIA_EXTENDED_ECUS,
    "genesis": HYUNDAI_KIA_EXTENDED_ECUS,
    # Italian
    "fiat": FIAT_EXTENDED_ECUS,
    "alfa romeo": FIAT_EXTENDED_ECUS,
    "lancia": FIAT_EXTENDED_ECUS,
    "abarth": FIAT_EXTENDED_ECUS,
    "maserati": FIAT_EXTENDED_ECUS,
    # Swedish
    "volvo": VOLVO_EXTENDED_ECUS,
    # French
    "renault": None,
    "dacia": None,
    # Other
    "tesla": None,
}

MANUFACTURER_DID_MAP = {
    "peugeot": PSA_EXTENDED_DIDS,
    "citroën": PSA_EXTENDED_DIDS,
    "citroen": PSA_EXTENDED_DIDS,
    "ds": PSA_EXTENDED_DIDS,
    "opel": PSA_EXTENDED_DIDS,
    "mercedes-benz": MERCEDES_EXTENDED_DIDS,
    "mercedes": MERCEDES_EXTENDED_DIDS,
    "ford": FORD_EXTENDED_DIDS,
    "lincoln": FORD_EXTENDED_DIDS,
    "toyota": TOYOTA_EXTENDED_DIDS,
    "lexus": TOYOTA_EXTENDED_DIDS,
    "honda": HONDA_EXTENDED_DIDS,
    "acura": HONDA_EXTENDED_DIDS,
    "mazda": MAZDA_EXTENDED_DIDS,
    "subaru": SUBARU_EXTENDED_DIDS,
    "hyundai": HYUNDAI_KIA_EXTENDED_DIDS,
    "kia": HYUNDAI_KIA_EXTENDED_DIDS,
    "genesis": HYUNDAI_KIA_EXTENDED_DIDS,
    "fiat": FIAT_EXTENDED_DIDS,
    "alfa romeo": FIAT_EXTENDED_DIDS,
    "volvo": VOLVO_EXTENDED_DIDS,
    "bmw": BMW_COMMON_DIDS,
    "renault": RENAULT_EXTENDED_DIDS,
    "dacia": RENAULT_EXTENDED_DIDS,
    "nissan": RENAULT_EXTENDED_DIDS,
}


VEHICLE_PROFILES: Dict[str, VehicleProfile] = {
    "peugeot_206": VehicleProfile(
        make="Peugeot",
        model="206",
        year_range=(1998, 2009),
        ecus=[
            ECUInfo(
                name="Peugeot 206 Engine Control",
                request_id=0x701,
                response_id=0x709,
                protocol="KWP2000",
                description="Engine control module"
            ),
            ECUInfo(
                name="Peugeot 206 ABS",
                request_id=0x751,
                response_id=0x759,
                protocol="KWP2000",
                description="Anti-lock brake system"
            ),
            ECUInfo(
                name="Peugeot 206 Airbag",
                request_id=0x761,
                response_id=0x769,
                protocol="KWP2000",
                description="Airbag control module"
            ),
            ECUInfo(
                name="Peugeot 206 BSI",
                request_id=0x771,
                response_id=0x779,
                protocol="KWP2000",
                description="Body system interface"
            ),
            ECUInfo(
                name="PSA Extended - Body Systems Interface",
                request_id=0x75D,
                response_id=0x65D,
                protocol="UDS",
                description="PSA Extended - Body Systems Interface"
            ),
            ECUInfo(
                name="PSA Extended - Injection/Engine ECU",
                request_id=0x6A8,
                response_id=0x688,
                protocol="UDS",
                description="PSA Extended - Injection/Engine ECU"
            ),
            ECUInfo(
                name="PSA Extended - ABS/ESP Module",
                request_id=0x6AD,
                response_id=0x68D,
                protocol="UDS",
                description="PSA Extended - ABS/ESP Module"
            ),
            ECUInfo(
                name="PSA Extended - Climate Control",
                request_id=0x76D,
                response_id=0x66D,
                protocol="UDS",
                description="PSA Extended - Climate Control"
            ),
            ECUInfo(
                name="PSA Extended - Gearbox ECU",
                request_id=0x6A9,
                response_id=0x689,
                protocol="UDS",
                description="PSA Extended - Gearbox ECU"
            ),
            ECUInfo(
                name="PSA Extended - Power Steering",
                request_id=0x6B2,
                response_id=0x692,
                protocol="UDS",
                description="PSA Extended - Power Steering"
            ),
            ECUInfo(
                name="PSA Extended - Airbag Module",
                request_id=0x772,
                response_id=0x672,
                protocol="UDS",
                description="PSA Extended - Airbag Module"
            ),
            ECUInfo(
                name="PSA Extended - Display Unit",
                request_id=0x765,
                response_id=0x665,
                protocol="UDS",
                description="PSA Extended - Display Unit"
            ),
            ECUInfo(
                name="PSA Extended - BSI Commands",
                request_id=0x773,
                response_id=0x673,
                protocol="UDS",
                description="PSA Extended - BSI Commands"
            ),
            ECUInfo(
                name="PSA Extended - Telematics Unit",
                request_id=0x764,
                response_id=0x664,
                protocol="UDS",
                description="PSA Extended - Telematics Unit"
            ),
            ECUInfo(
                name="PSA Extended - Instrument Cluster",
                request_id=0x734,
                response_id=0x714,
                protocol="UDS",
                description="PSA Extended - Instrument Cluster"
            ),
            ECUInfo(
                name="PSA Extended - Radio/Audio",
                request_id=0x7A0,
                response_id=0x6A0,
                protocol="UDS",
                description="PSA Extended - Radio/Audio"
            ),
            ECUInfo(
                name="PSA Extended - Parking Sensors",
                request_id=0x762,
                response_id=0x662,
                protocol="UDS",
                description="PSA Extended - Parking Sensors"
            ),
        ],
        notes="Common PSA platform, supports both KWP2000 and CAN protocols"
    ),
    "peugeot_207": VehicleProfile(
        make="Peugeot",
        model="207",
        year_range=(2006, 2015),
        ecus=[
            ECUInfo(
                name="Peugeot 207 Engine Control",
                request_id=0x701,
                response_id=0x709,
                protocol="KWP2000",
                description="Engine control module"
            ),
            ECUInfo(
                name="Peugeot 207 ABS",
                request_id=0x751,
                response_id=0x759,
                protocol="KWP2000",
                description="Anti-lock brake system"
            ),
            ECUInfo(
                name="Peugeot 207 Airbag",
                request_id=0x761,
                response_id=0x769,
                protocol="KWP2000",
                description="Airbag control module"
            ),
            ECUInfo(
                name="Peugeot 207 BSI",
                request_id=0x771,
                response_id=0x779,
                protocol="KWP2000",
                description="Body system interface"
            ),
            ECUInfo(
                name="PSA Extended - Body Systems Interface",
                request_id=0x75D,
                response_id=0x65D,
                protocol="UDS",
                description="PSA Extended - Body Systems Interface"
            ),
            ECUInfo(
                name="PSA Extended - Injection/Engine ECU",
                request_id=0x6A8,
                response_id=0x688,
                protocol="UDS",
                description="PSA Extended - Injection/Engine ECU"
            ),
            ECUInfo(
                name="PSA Extended - ABS/ESP Module",
                request_id=0x6AD,
                response_id=0x68D,
                protocol="UDS",
                description="PSA Extended - ABS/ESP Module"
            ),
            ECUInfo(
                name="PSA Extended - Climate Control",
                request_id=0x76D,
                response_id=0x66D,
                protocol="UDS",
                description="PSA Extended - Climate Control"
            ),
            ECUInfo(
                name="PSA Extended - Gearbox ECU",
                request_id=0x6A9,
                response_id=0x689,
                protocol="UDS",
                description="PSA Extended - Gearbox ECU"
            ),
            ECUInfo(
                name="PSA Extended - Power Steering",
                request_id=0x6B2,
                response_id=0x692,
                protocol="UDS",
                description="PSA Extended - Power Steering"
            ),
            ECUInfo(
                name="PSA Extended - Airbag Module",
                request_id=0x772,
                response_id=0x672,
                protocol="UDS",
                description="PSA Extended - Airbag Module"
            ),
            ECUInfo(
                name="PSA Extended - Display Unit",
                request_id=0x765,
                response_id=0x665,
                protocol="UDS",
                description="PSA Extended - Display Unit"
            ),
            ECUInfo(
                name="PSA Extended - BSI Commands",
                request_id=0x773,
                response_id=0x673,
                protocol="UDS",
                description="PSA Extended - BSI Commands"
            ),
            ECUInfo(
                name="PSA Extended - Telematics Unit",
                request_id=0x764,
                response_id=0x664,
                protocol="UDS",
                description="PSA Extended - Telematics Unit"
            ),
            ECUInfo(
                name="PSA Extended - Instrument Cluster",
                request_id=0x734,
                response_id=0x714,
                protocol="UDS",
                description="PSA Extended - Instrument Cluster"
            ),
            ECUInfo(
                name="PSA Extended - Radio/Audio",
                request_id=0x7A0,
                response_id=0x6A0,
                protocol="UDS",
                description="PSA Extended - Radio/Audio"
            ),
            ECUInfo(
                name="PSA Extended - Parking Sensors",
                request_id=0x762,
                response_id=0x662,
                protocol="UDS",
                description="PSA Extended - Parking Sensors"
            ),
        ],
        notes="PSA platform, supports KWP2000 and CAN"
    ),
    "audi_a3": VehicleProfile(
        make="Audi",
        model="A3",
        year_range=(1996, 2023),
        ecus=[
            ECUInfo(
                name="Audi A3 Engine Control",
                request_id=0x101,
                response_id=0x109,
                protocol="KWP2000",
                description="Engine control module (ME7.X)"
            ),
            ECUInfo(
                name="Audi A3 Transmission",
                request_id=0x201,
                response_id=0x209,
                protocol="KWP2000",
                description="Transmission control module"
            ),
            ECUInfo(
                name="Audi A3 ABS/ESP",
                request_id=0x301,
                response_id=0x309,
                protocol="KWP2000",
                description="ABS and electronic stability program"
            ),
            ECUInfo(
                name="Audi A3 Airbag",
                request_id=0x401,
                response_id=0x409,
                protocol="KWP2000",
                description="Airbag control module"
            ),
            ECUInfo(
                name="Audi A3 Gateway",
                request_id=0x501,
                response_id=0x509,
                protocol="CAN",
                description="Gateway module for CAN bus"
            ),
        ],
        notes="Volkswagen Group platform, uses ME7 engine controller"
    ),
    "audi_a4": VehicleProfile(
        make="Audi",
        model="A4",
        year_range=(1994, 2023),
        ecus=[
            ECUInfo(
                name="Audi A4 Engine Control",
                request_id=0x101,
                response_id=0x109,
                protocol="KWP2000",
                description="Engine control module"
            ),
            ECUInfo(
                name="Audi A4 Transmission",
                request_id=0x201,
                response_id=0x209,
                protocol="KWP2000",
                description="Transmission control module"
            ),
            ECUInfo(
                name="Audi A4 ABS/ESP",
                request_id=0x301,
                response_id=0x309,
                protocol="KWP2000",
                description="ABS and electronic stability program"
            ),
            ECUInfo(
                name="Audi A4 Airbag",
                request_id=0x401,
                response_id=0x409,
                protocol="KWP2000",
                description="Airbag control module"
            ),
            ECUInfo(
                name="Audi A4 Gateway",
                request_id=0x501,
                response_id=0x509,
                protocol="CAN",
                description="Gateway module for CAN bus"
            ),
        ],
        notes="Luxury sedan with advanced electronics"
    ),
    "vw_golf": VehicleProfile(
        make="Volkswagen",
        model="Golf",
        year_range=(1992, 2023),
        ecus=[
            ECUInfo(
                name="VW Golf Engine Control",
                request_id=0x101,
                response_id=0x109,
                protocol="KWP2000",
                description="Engine control module"
            ),
            ECUInfo(
                name="VW Golf Transmission",
                request_id=0x201,
                response_id=0x209,
                protocol="KWP2000",
                description="Transmission control module"
            ),
            ECUInfo(
                name="VW Golf ABS",
                request_id=0x301,
                response_id=0x309,
                protocol="KWP2000",
                description="ABS control module"
            ),
            ECUInfo(
                name="VW Golf Airbag",
                request_id=0x401,
                response_id=0x409,
                protocol="KWP2000",
                description="Airbag control module"
            ),
        ],
        notes="Popular compact car, widely available across generations"
    ),
    "generic": VehicleProfile(
        make="Generic",
        model="OBD-II Vehicle",
        year_range=(1996, 2023),
        ecus=GENERIC_ECUS,
        notes="Standard OBD-II ECU addresses, compatible with most vehicles"
    ),
    "mercedes_c_class": VehicleProfile(
        make="Mercedes-Benz",
        model="C-Class",
        year_range=(2007, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Mercedes - {v['name']}") for v in MERCEDES_EXTENDED_ECUS.values()],
        notes="W204/W205/W206 C-Class. Extended DIDs via UDS Service 0x22."
    ),
    "ford_focus": VehicleProfile(
        make="Ford",
        model="Focus",
        year_range=(2005, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Ford - {v['name']}") for v in FORD_EXTENDED_ECUS.values()],
        notes="Mk2/Mk3/Mk4 Focus. Ford IDS extended diagnostics."
    ),
    "toyota_corolla": VehicleProfile(
        make="Toyota",
        model="Corolla",
        year_range=(2002, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Toyota - {v['name']}") for v in TOYOTA_EXTENDED_ECUS.values()],
        notes="E120/E140/E170/E210 Corolla. Toyota Techstream extended data."
    ),
    "honda_civic": VehicleProfile(
        make="Honda",
        model="Civic",
        year_range=(2001, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Honda - {v['name']}") for v in HONDA_EXTENDED_ECUS.values()],
        notes="FD/FK/FL Civic. Honda HDS extended diagnostics."
    ),
    "hyundai_i30": VehicleProfile(
        make="Hyundai",
        model="i30",
        year_range=(2007, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Hyundai - {v['name']}") for v in HYUNDAI_KIA_EXTENDED_ECUS.values()],
        notes="GD/PD i30. Hyundai GDS2 extended diagnostics."
    ),
    "fiat_500": VehicleProfile(
        make="Fiat",
        model="500",
        year_range=(2007, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Fiat - {v['name']}") for v in FIAT_EXTENDED_ECUS.values()],
        notes="Fiat 500/500X/500L. FCA Uconnect diagnostics."
    ),
    "volvo_xc60": VehicleProfile(
        make="Volvo",
        model="XC60",
        year_range=(2008, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Volvo - {v['name']}") for v in VOLVO_EXTENDED_ECUS.values()],
        notes="Volvo XC60 SPA/P3 platform. VIDA extended diagnostics."
    ),
    "mazda_3": VehicleProfile(
        make="Mazda",
        model="3",
        year_range=(2004, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Mazda - {v['name']}") for v in MAZDA_EXTENDED_ECUS.values()],
        notes="BK/BL/BM/BP Mazda 3. Mazda IDS extended diagnostics."
    ),
    "subaru_impreza": VehicleProfile(
        make="Subaru",
        model="Impreza",
        year_range=(2001, 2025),
        ecus=GENERIC_ECUS + [ECUInfo(name=v["name"], request_id=v["request_id"],
              response_id=v["response_id"], protocol=v["protocol"],
              description=f"Subaru - {v['name']}") for v in SUBARU_EXTENDED_ECUS.values()],
        notes="GD/GR/GP/GK Impreza. Subaru SSM extended diagnostics."
    ),
}

# Per-model loops removed — profiles are now built dynamically
# via get_vehicle_profile(make, model) using MANUFACTURER_ECU_MAP.
# Legacy profiles above kept for backward compatibility only.


COMMON_UIDS: Dict[int, str] = {
    0xF190: "Vehicle Identification Number (VIN)",
    0xF18C: "ECU Serial Number",
    0xF191: "ECU Hardware Version",
    0xF187: "Spare Part Number",
    0xF189: "ECU Software Version",
    0xF186: "Active Diagnostic Session",
    0xF192: "System Name or Engine Type",
    0xF193: "System Name or Engine Type",
    0xF194: "System Identification Data",
    0xF195: "System Supplier Identifier",
    0xF196: "ECU Software Number",
    0xF197: "ECU Software Version Number",
    0xF198: "System Name or Engine Type",
    0xF199: "System Software Version",
    0xF19A: "System Supplier Identifier",
    0xF19B: "Component Software Number",
    0xF19C: "Component Software Version",
    0xF19D: "Spare Part Number",
    0xF19E: "Calibration Identification",
    0xF19F: "Calibration Revision Number",
    0xF1A0: "System Supplier Identifier",
    0xF1A1: "ECU Hardware Number",
    0xF1A2: "ECU Hardware Version Number",
    0xF1A3: "System Software Number",
    0xF1A4: "System Software Version Number",
    0xF1A5: "Spare Part Number",
    0xF1A6: "Calibration Identification",
    0xF1A7: "Calibration Revision Number",
    0xF1A8: "ODX File",
    0xF1A9: "Vehicle Manufacturer ECU Hardware Number",
    0xF1AA: "Vehicle Manufacturer ECU Hardware Version Number",
    0xF1AB: "System Supplier Specific ECU Hardware Number",
    0xF1AC: "System Supplier Specific ECU Hardware Version Number",
    0xF1AD: "ECU Serial Number",
    0xF1AE: "Fault Memory",
    0xF1AF: "Calibration Identification",
    0xF1B0: "Calibration Revision Number",
    0xF1B1: "System Software Version",
    0xF1B2: "System Supplier Identifier",
    0xF1B3: "ECU Hardware Number",
    0xF1B4: "ECU Hardware Version Number",
    0xF400: "Vehicle Speed",
    0xF401: "Engine Speed",
    0xF402: "Transmission Gear",
    0xF403: "Engine Load",
    0xF404: "Coolant Temperature",
    0xF405: "Intake Air Temperature",
    0xF406: "Fuel Level",
    0xF407: "Ignition Status",
    0xF408: "Brake Status",
    0xF409: "Steering Angle",
    0xF40A: "Gear Position",
}


def get_extended_dids(make: str) -> dict:
    """Get standard + manufacturer-specific DIDs for a vehicle make.

    Uses MANUFACTURER_DID_MAP for automatic lookup — no per-model config needed.
    """
    base = dict(COMMON_UIDS)
    ext = MANUFACTURER_DID_MAP.get(make.lower())
    if ext:
        base.update(ext)
    return base


def get_extended_ecus(make: str) -> List[ECUInfo]:
    """Get manufacturer-specific extended ECU addresses.

    Uses MANUFACTURER_ECU_MAP for automatic lookup — no per-model config needed.
    """
    ecu_dict = MANUFACTURER_ECU_MAP.get(make.lower())
    if not ecu_dict:
        return []
    return [
        ECUInfo(name=v["name"], request_id=v["request_id"],
                response_id=v["response_id"], protocol=v["protocol"],
                description=f"{make} - {v['name']}")
        for v in ecu_dict.values()
    ]


def get_ecus_for_make(make: str) -> List[ECUInfo]:
    """Get all ECUs (generic + extended) for a manufacturer.

    This is the main function used by the app — given a make (from VIN),
    it returns the complete ECU list without needing a model.
    """
    return GENERIC_ECUS + get_extended_ecus(make)


def get_vehicle_profile(make: str, model: str = "") -> VehicleProfile:
    """Build a vehicle profile dynamically from manufacturer data.

    No need to pre-register every model — the profile is built on-the-fly
    from the manufacturer's ECU map.
    """
    # Check legacy profiles first (for backward compat)
    if model:
        key = f"{make.lower()}_{model.lower()}".replace(" ", "_")
        if key in VEHICLE_PROFILES:
            return VEHICLE_PROFILES[key]

    # Build dynamic profile from manufacturer map
    ecus = get_ecus_for_make(make)
    return VehicleProfile(
        make=make, model=model or "All", year_range=(1996, 2026),
        ecus=ecus,
        notes=f"Auto-generated profile for {make} vehicles"
    )


def get_ecus_for_vehicle(make: str, model: str = "") -> List[ECUInfo]:
    """Get list of ECUs for a vehicle. Model is optional."""
    return get_vehicle_profile(make, model).ecus


def get_uid_description(uid: int) -> Optional[str]:
    """Get the description for a Data Identifier (UID)."""
    return COMMON_UIDS.get(uid)


def get_supported_makes() -> List[str]:
    """Get list of all supported manufacturer names."""
    return sorted(set(
        make.title() for make in MANUFACTURER_ECU_MAP.keys()
        if make not in ("citroen", "škoda", "mercedes", "mb")  # Skip aliases
    ))
