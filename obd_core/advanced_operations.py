"""Advanced operations database — ONLY verified codes with public sources.

VERIFIED sources with URLs:
- arduino-psa-diag (GPL): https://github.com/ludwig-v/arduino-psa-diag
  → BSI zones, ECU addresses, actuator commands, security keys
- psa-seedkey-algorithm (GPL): https://github.com/ludwig-v/psa-seedkey-algorithm
  → Security key values per ECU variant
- DDT4All (GPL): https://github.com/cedricp/ddt4all
  → PSA/Renault DID definitions
- VW_Flash (GitHub): https://github.com/bri3d/VW_Flash
  → VAG UDS DIDs and routine IDs
- vag-uds-ids (GitHub): https://github.com/ConnorHowell/vag-uds-ids
  → VAG CAN address mapping (76 modules)
- OVMS (GPL): https://github.com/openvehicles/Open-Vehicle-Monitoring-System-3
  → Hyundai/Kia BMS and VMCU DIDs

UNVERIFIED codes are NOT included.
"""

from __future__ import annotations

from i18n import get_lang


# ── Operation Categories ──────────────────────────────────────
CATEGORIES = {
    "service_reset": {
        "name": {"fr": "Entretien (maintenance)", "en": "Service / Maintenance"},
        "desc": {
            "fr": "Lecture / écriture des données d'entretien du véhicule.",
            "en": "Read / write vehicle service maintenance data.",
        },
        "risk": "low",
    },
    "vehicle_config": {
        "name": {"fr": "Configuration véhicule", "en": "Vehicle Configuration"},
        "desc": {
            "fr": "Lecture de la configuration usine du véhicule (options, équipements).",
            "en": "Read factory vehicle configuration (options, equipment).",
        },
        "risk": "low",
    },
    "dpf_regeneration": {
        "name": {"fr": "Filtre à particules (FAP/DPF)", "en": "Diesel Particulate Filter (DPF)"},
        "desc": {
            "fr": "Lecture de l'état du filtre à particules.",
            "en": "Read DPF status data.",
        },
        "risk": "low",
    },
    "actuator_test": {
        "name": {"fr": "Tests actuateurs", "en": "Actuator Tests"},
        "desc": {
            "fr": "Active individuellement des actuateurs pour diagnostic (haut-parleurs, écran, caméra).",
            "en": "Individually activates actuators for diagnostics (speakers, screen, camera).",
        },
        "risk": "medium",
    },
    "ecu_identification": {
        "name": {"fr": "Identification calculateurs", "en": "ECU Identification"},
        "desc": {
            "fr": "Lecture d'informations détaillées des calculateurs (références, versions, codage).",
            "en": "Read detailed ECU information (part numbers, versions, coding).",
        },
        "risk": "low",
    },
    "ev_battery": {
        "name": {"fr": "Batterie haute tension (VE)", "en": "HV Battery (EV)"},
        "desc": {
            "fr": "Lecture des données de la batterie haute tension (véhicules électriques).",
            "en": "Read high-voltage battery data (electric vehicles).",
        },
        "risk": "low",
    },
}


# ── Manufacturer → Make mapping ───────────────────────────────
MANUFACTURER_GROUPS = {
    "psa": ["Peugeot", "Citroën", "Citroen", "DS", "Opel", "Vauxhall"],
    "vag": ["Volkswagen", "Audi", "Seat", "Cupra", "Skoda", "Škoda", "Porsche"],
    "bmw": ["BMW", "Mini", "MINI"],
    "mercedes": ["Mercedes-Benz", "Mercedes"],
    "renault": ["Renault", "Dacia"],
    "hyundai": ["Hyundai", "Kia", "Genesis"],
}


def _make_to_group(make: str) -> str:
    for group, makes in MANUFACTURER_GROUPS.items():
        if make in makes:
            return group
    return ""


# ══════════════════════════════════════════════════════════════
# PSA (Peugeot, Citroën, DS, Opel)
#
# Sources:
#   https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md
#   https://github.com/ludwig-v/arduino-psa-diag/blob/master/ECU_LIST.md
#   https://github.com/ludwig-v/arduino-psa-diag/blob/master/actuators/TELEMAT.md
#   https://github.com/ludwig-v/psa-seedkey-algorithm/blob/main/ECU_KEYS.md
#   https://github.com/cedricp/ddt4all
#
# BSI CAN: TX 0x752 / RX 0x652
# INJ CAN: TX 0x6A8 / RX 0x688
# NAC/RCC CAN: TX 0x764 / RX 0x664
# Applies to: PSA/Stellantis with BSI2010 (2010+ 208/308/2008/3008/5008,
#   C3/C4/C5/DS3/DS4/DS5, Opel Corsa F/Mokka/Crossland)
# ══════════════════════════════════════════════════════════════
_PSA_OPS = [
    # ── MAINTENANCE: Read BSI zone 2282 ───────────────────────
    {
        "id": "psa_maint_read",
        "category": "service_reset",
        "name": {"fr": "Lire seuil entretien (BSI zone 2282)", "en": "Read Service Threshold (BSI zone 2282)"},
        "desc": {
            "fr": "Lit la zone 2282 du BSI : seuil de maintenance en unités de 200 km.",
            "en": "Reads BSI zone 2282: maintenance threshold in units of 200 km.",
        },
        "ecu_name": "BSI",
        "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2282,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    # ── MAINTENANCE: Write BSI zone 2282 ──────────────────────
    {
        "id": "psa_maint_write",
        "category": "service_reset",
        "name": {"fr": "Écrire seuil entretien (BSI zone 2282)", "en": "Write Service Threshold (BSI zone 2282)"},
        "desc": {
            "fr": "Écrit la zone 2282 du BSI. Nécessite SecurityAccess (clé BSI: 0xB2B2 standard, "
                  "0xE4D8 BSI2010 Valeo, 0xC318 BSI2010 Continental). "
                  "Écrire zone 2901 après (traçabilité obligatoire).",
            "en": "Writes BSI zone 2282. Requires SecurityAccess (BSI key: 0xB2B2 standard, "
                  "0xE4D8 BSI2010 Valeo, 0xC318 BSI2010 Continental). "
                  "Write zone 2901 after (mandatory traceability).",
        },
        "ecu_name": "BSI",
        "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": True,
        "command_type": "write", "service": 0x2E, "did_or_rid": 0x2282,
        "data_template": "raw_hex",
        "parameters": [
            {"key": "raw_data", "name": {"fr": "Données hex (zone 2282)", "en": "Hex data (zone 2282)"}, "type": "hex",
             "hint": {"fr": "Lisez d'abord la valeur actuelle", "en": "Read current value first"}},
        ],
        "pre_conditions": {
            "fr": ["Contact mis", "Lire la zone d'abord", "Écrire 2E 2901 FD 00 00 00 01 01 01 après"],
            "en": ["Ignition on", "Read the zone first", "Write 2E 2901 FD 00 00 00 01 01 01 after"],
        },
        "risk": "medium",
        "notes": {
            "fr": "Clés sécurité BSI vérifiées en source.",
            "en": "BSI security keys verified in source.",
        },
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    # ── VEHICLE CONFIG: Read key BSI configuration zones ──────
    {
        "id": "psa_config_engine",
        "category": "vehicle_config",
        "name": {"fr": "Type moteur (BSI zone 231C)", "en": "Engine Type (BSI zone 231C)"},
        "desc": {"fr": "Lit le type de moteur configuré dans le BSI.", "en": "Reads engine type configured in BSI."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x231C,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    {
        "id": "psa_config_gearbox",
        "category": "vehicle_config",
        "name": {"fr": "Type boîte de vitesses (BSI zone 231A)", "en": "Gearbox Type (BSI zone 231A)"},
        "desc": {"fr": "Lit le type de boîte configuré dans le BSI.", "en": "Reads gearbox type configured in BSI."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x231A,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    {
        "id": "psa_config_dpf_type",
        "category": "vehicle_config",
        "name": {"fr": "Type FAP (BSI zone 232D)", "en": "DPF Type (BSI zone 232D)"},
        "desc": {"fr": "Lit le type de FAP configuré.", "en": "Reads configured DPF type."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x232D,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    {
        "id": "psa_config_battery",
        "category": "vehicle_config",
        "name": {"fr": "Type batterie (BSI zone 2333)", "en": "Battery Type (BSI zone 2333)"},
        "desc": {"fr": "Lit le type de batterie configuré.", "en": "Reads configured battery type."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2333,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    {
        "id": "psa_config_climate",
        "category": "vehicle_config",
        "name": {"fr": "Type climatisation (BSI zone 2318)", "en": "Climate Type (BSI zone 2318)"},
        "desc": {"fr": "Lit le type de climatisation configuré.", "en": "Reads configured climate type."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2318,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    {
        "id": "psa_config_startstop",
        "category": "vehicle_config",
        "name": {"fr": "Start&Stop (BSI zone 23BB)", "en": "Start&Stop (BSI zone 23BB)"},
        "desc": {"fr": "Lit la présence et type de Start&Stop.", "en": "Reads Start&Stop presence and type."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x23BB,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    {
        "id": "psa_config_drl",
        "category": "vehicle_config",
        "name": {"fr": "Feux diurnes (BSI zone 232A)", "en": "Daytime Running Lights (BSI zone 232A)"},
        "desc": {"fr": "Lit le type de feux de jour configuré.", "en": "Reads configured DRL type."},
        "ecu_name": "BSI", "ecu_tx": 0x752, "ecu_rx": 0x652,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x232A,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/zones/BMF.md",
    },
    # ── DPF: Read engine DID 0x2106 ───────────────────────────
    {
        "id": "psa_dpf_status",
        "category": "dpf_regeneration",
        "name": {"fr": "État FAP (moteur DID 0x2106)", "en": "DPF Status (engine DID 0x2106)"},
        "desc": {"fr": "Lit le groupe de données FAP du calculateur moteur.", "en": "Reads DPF data group from engine ECU."},
        "ecu_name": "INJ (Moteur)",
        "ecu_tx": 0x6A8, "ecu_rx": 0x688,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2106,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "Diesel HDi/BlueHDi"], "en": ["Ignition on", "HDi/BlueHDi diesel"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/cedricp/ddt4all",
    },
    # ── ACTUATOR TESTS: NAC/RCC (verified hex commands) ───────
    # Source: https://github.com/ludwig-v/arduino-psa-diag/blob/master/actuators/TELEMAT.md
    {
        "id": "psa_speaker_test_fr",
        "category": "actuator_test",
        "name": {"fr": "Test haut-parleur avant droit (1 kHz)", "en": "Front Right Speaker Test (1 kHz)"},
        "desc": {
            "fr": "Envoie un signal 1 kHz au haut-parleur avant droit via le NAC/RCC.",
            "en": "Sends a 1 kHz signal to the front right speaker via NAC/RCC.",
        },
        "ecu_name": "NAC/RCC",
        "ecu_tx": 0x764, "ecu_rx": 0x664,
        "session": 0x03, "security": False,
        "command_type": "raw",
        "service": 0x2F,
        "did_or_rid": 0xD620,
        "data_template": "raw_fixed",
        "raw_command": "2FD620030108",
        "raw_stop_command": "2FD62000",
        "parameters": [],
        "pre_conditions": {
            "fr": ["Contact mis", "NAC/RCC présent (écran tactile PSA 2016+)"],
            "en": ["Ignition on", "NAC/RCC present (PSA touchscreen 2016+)"],
        },
        "risk": "low",
        "notes": {
            "fr": "Envoie un bip de test. Envoi automatique de la commande STOP après.",
            "en": "Sends a test beep. STOP command sent automatically after.",
        },
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/actuators/TELEMAT.md",
    },
    {
        "id": "psa_speaker_test_fl",
        "category": "actuator_test",
        "name": {"fr": "Test haut-parleur avant gauche (1 kHz)", "en": "Front Left Speaker Test (1 kHz)"},
        "desc": {"fr": "Signal 1 kHz au haut-parleur avant gauche.", "en": "1 kHz signal to front left speaker."},
        "ecu_name": "NAC/RCC",
        "ecu_tx": 0x764, "ecu_rx": 0x664,
        "session": 0x03, "security": False,
        "command_type": "raw",
        "service": 0x2F, "did_or_rid": 0xD620,
        "data_template": "raw_fixed",
        "raw_command": "2FD620030208",
        "raw_stop_command": "2FD62000",
        "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "NAC/RCC présent"], "en": ["Ignition on", "NAC/RCC present"]},
        "risk": "low",
        "notes": {"fr": "Test sonore temporaire.", "en": "Temporary sound test."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/actuators/TELEMAT.md",
    },
    {
        "id": "psa_speaker_test_rr",
        "category": "actuator_test",
        "name": {"fr": "Test haut-parleur arrière droit (1 kHz)", "en": "Rear Right Speaker Test (1 kHz)"},
        "desc": {"fr": "Signal 1 kHz au haut-parleur arrière droit.", "en": "1 kHz signal to rear right speaker."},
        "ecu_name": "NAC/RCC",
        "ecu_tx": 0x764, "ecu_rx": 0x664,
        "session": 0x03, "security": False,
        "command_type": "raw",
        "service": 0x2F, "did_or_rid": 0xD620,
        "data_template": "raw_fixed",
        "raw_command": "2FD620030308",
        "raw_stop_command": "2FD62000",
        "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "NAC/RCC présent"], "en": ["Ignition on", "NAC/RCC present"]},
        "risk": "low",
        "notes": {"fr": "Test sonore temporaire.", "en": "Temporary sound test."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/actuators/TELEMAT.md",
    },
    {
        "id": "psa_speaker_test_rl",
        "category": "actuator_test",
        "name": {"fr": "Test haut-parleur arrière gauche (1 kHz)", "en": "Rear Left Speaker Test (1 kHz)"},
        "desc": {"fr": "Signal 1 kHz au haut-parleur arrière gauche.", "en": "1 kHz signal to rear left speaker."},
        "ecu_name": "NAC/RCC",
        "ecu_tx": 0x764, "ecu_rx": 0x664,
        "session": 0x03, "security": False,
        "command_type": "raw",
        "service": 0x2F, "did_or_rid": 0xD620,
        "data_template": "raw_fixed",
        "raw_command": "2FD620030408",
        "raw_stop_command": "2FD62000",
        "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "NAC/RCC présent"], "en": ["Ignition on", "NAC/RCC present"]},
        "risk": "low",
        "notes": {"fr": "Test sonore temporaire.", "en": "Temporary sound test."},
        "source": "https://github.com/ludwig-v/arduino-psa-diag/blob/master/actuators/TELEMAT.md",
    },
]

# ══════════════════════════════════════════════════════════════
# VAG (Volkswagen, Audi, Seat, Skoda, Porsche)
#
# Sources:
#   https://github.com/ConnorHowell/vag-uds-ids (CAN addresses from ODIS)
#   https://github.com/bri3d/VW_Flash (DIDs for Simos/DQ/Haldex)
#
# Engine CAN: TX 0x7E0 / RX 0x7E8
# Instrument Cluster CAN: TX 0x714 / RX 0x77E
# Battery Regulation CAN: TX 0x728 / RX 0x792
# Applies to: MQB platform (2012+ Golf 7, A3 8V, Leon 5F, Octavia 5E, etc.)
#
# NOTE: No verified WRITE operations found in public sources.
# VCDS/OBDeleven abstract all writes behind proprietary GUI.
# ══════════════════════════════════════════════════════════════
_VAG_OPS = [
    {
        "id": "vag_engine_code",
        "category": "ecu_identification",
        "name": {"fr": "Code moteur (DID 0xF1AD)", "en": "Engine Code Letters (DID 0xF1AD)"},
        "desc": {"fr": "Lit le code moteur (ex: CJSA, CZEA, DFGA).", "en": "Reads engine code letters (e.g.: CJSA, CZEA, DFGA)."},
        "ecu_name": "Engine (ECM)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xF1AD,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule — MQB (2012+).", "en": "Read-only — MQB (2012+)."},
        "source": "https://github.com/bri3d/VW_Flash/blob/master/lib/constants.py",
    },
    {
        "id": "vag_part_number",
        "category": "ecu_identification",
        "name": {"fr": "Référence pièce VAG (DID 0xF187)", "en": "VAG Spare Part Number (DID 0xF187)"},
        "desc": {"fr": "Lit la référence pièce VAG du calculateur.", "en": "Reads VAG spare part number from ECU."},
        "ecu_name": "Engine (ECM)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xF187,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/bri3d/VW_Flash/blob/master/lib/constants.py",
    },
    {
        "id": "vag_coding",
        "category": "ecu_identification",
        "name": {"fr": "Codage VAG (DID 0x0600)", "en": "VAG Coding Value (DID 0x0600)"},
        "desc": {"fr": "Lit la valeur de codage actuelle du calculateur.", "en": "Reads current ECU coding value."},
        "ecu_name": "Engine (ECM)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x0600,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/bri3d/VW_Flash/blob/master/lib/constants.py",
    },
    {
        "id": "vag_mileage",
        "category": "ecu_identification",
        "name": {"fr": "Kilométrage véhicule (DID 0x295A)", "en": "Vehicle Mileage (DID 0x295A)"},
        "desc": {"fr": "Lit le kilométrage stocké dans le combiné d'instruments.", "en": "Reads mileage stored in instrument cluster."},
        "ecu_name": "Instrument Cluster",
        "ecu_tx": 0x714, "ecu_rx": 0x77E,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x295A,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule — MQB.", "en": "Read-only — MQB."},
        "source": "https://github.com/bri3d/VW_Flash/blob/master/lib/constants.py",
    },
    {
        "id": "vag_voltage",
        "category": "ecu_identification",
        "name": {"fr": "Tension calculateur (DID 0xF442)", "en": "Control Module Voltage (DID 0xF442)"},
        "desc": {"fr": "Lit la tension d'alimentation du calculateur.", "en": "Reads ECU supply voltage."},
        "ecu_name": "Engine (ECM)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xF442,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis"], "en": ["Ignition on"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/bri3d/VW_Flash/blob/master/lib/constants.py",
    },
]

# ══════════════════════════════════════════════════════════════
# BMW (BMW, Mini)
#
# Sources:
#   ecu_database.py (DPF DIDs 0xDA01/0xDA02 verified)
# ══════════════════════════════════════════════════════════════
_BMW_OPS = [
    {
        "id": "bmw_dpf_soot",
        "category": "dpf_regeneration",
        "name": {"fr": "Taux de suie FAP (DID 0xDA01)", "en": "DPF Soot Loading (DID 0xDA01)"},
        "desc": {"fr": "Lit le taux de suie du filtre à particules.", "en": "Reads DPF soot loading."},
        "ecu_name": "DME/DDE (Engine)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xDA01,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "Diesel uniquement"], "en": ["Ignition on", "Diesel only"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "ecu_database.py — BMW DID 0xDA01",
    },
    {
        "id": "bmw_dpf_regen_status",
        "category": "dpf_regeneration",
        "name": {"fr": "État régénération DPF (DID 0xDA02)", "en": "DPF Regen Status (DID 0xDA02)"},
        "desc": {"fr": "Lit l'état de la dernière régénération.", "en": "Reads last regeneration status."},
        "ecu_name": "DME/DDE (Engine)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xDA02,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "Diesel uniquement"], "en": ["Ignition on", "Diesel only"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "ecu_database.py — BMW DID 0xDA02",
    },
]

# ══════════════════════════════════════════════════════════════
# Mercedes-Benz
# ══════════════════════════════════════════════════════════════
_MERCEDES_OPS = [
    {
        "id": "merc_dpf_status",
        "category": "dpf_regeneration",
        "name": {"fr": "État FAP CDI (DID 0x3001)", "en": "CDI DPF Status (DID 0x3001)"},
        "desc": {"fr": "Lit les données FAP du moteur CDI.", "en": "Reads CDI engine DPF data."},
        "ecu_name": "Engine (CDI)",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x3001,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "Diesel CDI"], "en": ["Ignition on", "CDI diesel"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "ecu_database.py — Mercedes DID 0x3001",
    },
]

# ══════════════════════════════════════════════════════════════
# Renault / Dacia
#
# Sources:
#   https://github.com/MeganeClubUkraine/k9k_pids (255 PIDs K9K 1.5 dCi)
#   https://github.com/gdincu/DaciaLodgy-K9K-OBD2-PIDs
#   https://github.com/cedricp/ddt4all
#
# Engine CAN: TX 0x7E0 / RX 0x7E8
# Applies to: Renault/Dacia with K9K 1.5 dCi engine
#   (Megane, Scenic, Clio, Captur, Kadjar, Duster, Sandero, Logan, Lodgy)
# ══════════════════════════════════════════════════════════════
_RENAULT_OPS = [
    # ── DPF: Soot mass ────────────────────────────────────────
    {
        "id": "ren_dpf_soot_mass",
        "category": "dpf_regeneration",
        "name": {"fr": "Masse de suie FAP (DID 0x242C)", "en": "DPF Soot Mass (DID 0x242C)"},
        "desc": {
            "fr": "Lit la masse de suie dans le FAP. Formule: (A*256+B)/100 grammes.",
            "en": "Reads soot mass in DPF. Formula: (A*256+B)/100 grams.",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x242C,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule. Formule: (A*256+B)/100 g", "en": "Read-only. Formula: (A*256+B)/100 g"},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Regen status ─────────────────────────────────────
    {
        "id": "ren_dpf_regen_status",
        "category": "dpf_regeneration",
        "name": {"fr": "Statut régénération FAP (DID 0x2434)", "en": "DPF Regen Status (DID 0x2434)"},
        "desc": {
            "fr": "Lit le statut de la régénération du FAP. Formule: (A&7).",
            "en": "Reads DPF regeneration status. Formula: (A&7).",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2434,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Inlet temperature ────────────────────────────────
    {
        "id": "ren_dpf_inlet_temp",
        "category": "dpf_regeneration",
        "name": {"fr": "Température entrée FAP (DID 0x2442)", "en": "DPF Inlet Temperature (DID 0x2442)"},
        "desc": {
            "fr": "Lit la température à l'entrée du FAP. Formule: ((A*256+B)-2730)/10 °C.",
            "en": "Reads DPF inlet temperature. Formula: ((A*256+B)-2730)/10 °C.",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2442,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Differential pressure ────────────────────────────
    {
        "id": "ren_dpf_diff_pressure",
        "category": "dpf_regeneration",
        "name": {"fr": "Pression différentielle FAP (DID 0x2542)", "en": "DPF Differential Pressure (DID 0x2542)"},
        "desc": {
            "fr": "Lit la pression différentielle du FAP. Formule: (A*256+B)-32768 mbar.",
            "en": "Reads DPF differential pressure. Formula: (A*256+B)-32768 mbar.",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2542,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Successful regen count ───────────────────────────
    {
        "id": "ren_dpf_regen_count",
        "category": "dpf_regeneration",
        "name": {"fr": "Nombre régénérations réussies (DID 0x2481)", "en": "Successful Regen Count (DID 0x2481)"},
        "desc": {"fr": "Nombre de régénérations réussies du FAP.", "en": "Number of successful DPF regenerations."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2481,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Distance since last regen ────────────────────────
    {
        "id": "ren_dpf_dist_since_regen",
        "category": "dpf_regeneration",
        "name": {"fr": "Distance depuis dernière régénération (DID 0x24A9)", "en": "Distance Since Last Regen (DID 0x24A9)"},
        "desc": {
            "fr": "Distance parcourue depuis la dernière régénération réussie. Formule: A*65536+B*256+C km.",
            "en": "Distance driven since last successful regeneration. Formula: A*65536+B*256+C km.",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x24A9,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Last regen duration ──────────────────────────────
    {
        "id": "ren_dpf_last_regen_duration",
        "category": "dpf_regeneration",
        "name": {"fr": "Durée dernière régénération (DID 0x2487)", "en": "Last Regen Duration (DID 0x2487)"},
        "desc": {
            "fr": "Durée de la dernière régénération. Formule: (A*256+B)/600 minutes.",
            "en": "Duration of last regeneration. Formula: (A*256+B)/600 minutes.",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2487,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── DPF: Oil dilution ─────────────────────────────────────
    {
        "id": "ren_dpf_oil_dilution",
        "category": "dpf_regeneration",
        "name": {"fr": "Dilution huile moteur par gasoil (DID 0x24EC)", "en": "Engine Oil Dilution (DID 0x24EC)"},
        "desc": {
            "fr": "Taux de dilution de l'huile moteur par le gasoil (injection post). Signé, /10^6 %.",
            "en": "Engine oil fuel dilution rate (post-injection). Signed, /10^6 %.",
        },
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x24EC,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule. Important pour la santé du moteur.", "en": "Read-only. Important for engine health."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── Engine: Fuel correction per cylinder ──────────────────
    {
        "id": "ren_fuel_corr_cyl1",
        "category": "ecu_identification",
        "name": {"fr": "Correction carburant cyl.1 (DID 0xFD07)", "en": "Fuel Correction Cyl.1 (DID 0xFD07)"},
        "desc": {"fr": "Correction injection cylindre 1. Formule: 0.0000305*(A*256+B).", "en": "Cylinder 1 injection correction. Formula: 0.0000305*(A*256+B)."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xFD07,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    {
        "id": "ren_fuel_corr_cyl2",
        "category": "ecu_identification",
        "name": {"fr": "Correction carburant cyl.2 (DID 0xFD08)", "en": "Fuel Correction Cyl.2 (DID 0xFD08)"},
        "desc": {"fr": "Correction injection cylindre 2.", "en": "Cylinder 2 injection correction."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xFD08,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    {
        "id": "ren_fuel_corr_cyl3",
        "category": "ecu_identification",
        "name": {"fr": "Correction carburant cyl.3 (DID 0xFD09)", "en": "Fuel Correction Cyl.3 (DID 0xFD09)"},
        "desc": {"fr": "Correction injection cylindre 3.", "en": "Cylinder 3 injection correction."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xFD09,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    {
        "id": "ren_fuel_corr_cyl4",
        "category": "ecu_identification",
        "name": {"fr": "Correction carburant cyl.4 (DID 0xFD0A)", "en": "Fuel Correction Cyl.4 (DID 0xFD0A)"},
        "desc": {"fr": "Correction injection cylindre 4.", "en": "Cylinder 4 injection correction."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0xFD0A,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── Engine: Boost/Turbo ───────────────────────────────────
    {
        "id": "ren_boost_pressure",
        "category": "ecu_identification",
        "name": {"fr": "Pression turbo (DID 0x2401)", "en": "Boost Pressure (DID 0x2401)"},
        "desc": {"fr": "Pression de suralimentation. Formule: A*256+B mbar.", "en": "Boost pressure. Formula: A*256+B mbar."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2401,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── Engine: EGR position ──────────────────────────────────
    {
        "id": "ren_egr_position",
        "category": "ecu_identification",
        "name": {"fr": "Position vanne EGR (DID 0x2407)", "en": "EGR Valve Position (DID 0x2407)"},
        "desc": {"fr": "Position de la vanne EGR. Formule: (A*256+B)/20.491 %.", "en": "EGR valve position. Formula: (A*256+B)/20.491 %."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2407,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
    # ── Engine: Fuel rail pressure ────────────────────────────
    {
        "id": "ren_fuel_rail",
        "category": "ecu_identification",
        "name": {"fr": "Pression rampe injection (DID 0x2801)", "en": "Fuel Rail Pressure (DID 0x2801)"},
        "desc": {"fr": "Pression dans la rampe d'injection. Formule: A*256+B bar.", "en": "Fuel rail pressure. Formula: A*256+B bar."},
        "ecu_name": "Engine K9K",
        "ecu_tx": 0x7E0, "ecu_rx": 0x7E8,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x2801,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "K9K 1.5 dCi"], "en": ["Ignition on", "K9K 1.5 dCi"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/MeganeClubUkraine/k9k_pids",
    },
]

# ══════════════════════════════════════════════════════════════
# Hyundai / Kia / Genesis (EV battery data)
#
# Source: https://github.com/openvehicles/Open-Vehicle-Monitoring-System-3
#   → vehicle/OVMS.V3/components/vehicle_hyundai_ioniq5/
# BMS CAN: TX 0x7E4 / RX 0x7EC
# VMCU CAN: TX 0x7E2 / RX 0x7EA
# Applies to: Ioniq 5, EV6, GV60 and other E-GMP platform EVs
# ══════════════════════════════════════════════════════════════
_HYUNDAI_OPS = [
    {
        "id": "hk_bms_soc",
        "category": "ev_battery",
        "name": {"fr": "SOC batterie HT (BMS DID 0x0105)", "en": "HV Battery SOC (BMS DID 0x0105)"},
        "desc": {"fr": "Lit l'état de charge de la batterie haute tension.", "en": "Reads high-voltage battery state of charge."},
        "ecu_name": "BMS (Battery)",
        "ecu_tx": 0x7E4, "ecu_rx": 0x7EC,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x0105,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "Véhicule électrique E-GMP"], "en": ["Ignition on", "E-GMP electric vehicle"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule — Ioniq 5, EV6, GV60.", "en": "Read-only — Ioniq 5, EV6, GV60."},
        "source": "https://github.com/openvehicles/Open-Vehicle-Monitoring-System-3",
    },
    {
        "id": "hk_bms_voltage",
        "category": "ev_battery",
        "name": {"fr": "Tension batterie HT (BMS DID 0x0101)", "en": "HV Battery Voltage (BMS DID 0x0101)"},
        "desc": {"fr": "Lit la tension de la batterie haute tension.", "en": "Reads high-voltage battery voltage."},
        "ecu_name": "BMS (Battery)",
        "ecu_tx": 0x7E4, "ecu_rx": 0x7EC,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x0101,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "VE E-GMP"], "en": ["Ignition on", "E-GMP EV"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/openvehicles/Open-Vehicle-Monitoring-System-3",
    },
    {
        "id": "hk_bms_temp",
        "category": "ev_battery",
        "name": {"fr": "Température batterie HT (BMS DID 0x0104)", "en": "HV Battery Temp (BMS DID 0x0104)"},
        "desc": {"fr": "Lit la température de la batterie haute tension.", "en": "Reads high-voltage battery temperature."},
        "ecu_name": "BMS (Battery)",
        "ecu_tx": 0x7E4, "ecu_rx": 0x7EC,
        "session": 0x03, "security": False,
        "command_type": "read", "service": 0x22, "did_or_rid": 0x0104,
        "data_template": None, "parameters": [],
        "pre_conditions": {"fr": ["Contact mis", "VE E-GMP"], "en": ["Ignition on", "E-GMP EV"]},
        "risk": "low",
        "notes": {"fr": "Lecture seule.", "en": "Read-only."},
        "source": "https://github.com/openvehicles/Open-Vehicle-Monitoring-System-3",
    },
]


# ── All operations indexed by manufacturer group ──────────────
_ALL_OPS = {
    "psa": _PSA_OPS,
    "vag": _VAG_OPS,
    "bmw": _BMW_OPS,
    "mercedes": _MERCEDES_OPS,
    "renault": _RENAULT_OPS,
    "hyundai": _HYUNDAI_OPS,
}


def get_operations_for_make(make: str) -> list[dict]:
    group = _make_to_group(make)
    return _ALL_OPS.get(group, [])


def get_group_for_make(make: str) -> str:
    return _make_to_group(make)


def get_all_categories() -> dict:
    return CATEGORIES
