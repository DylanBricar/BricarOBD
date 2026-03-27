#!/usr/bin/env python3
"""
Comprehensive DTC description translator EN→FR.
Generates French translations for all DTC codes using pattern-based linguistic rules.
Preserves existing manual translations.
"""
import json
import re
import sys
from pathlib import Path

DATA_DIR = Path(__file__).parent.parent / "src-tauri" / "data"

# ============================================================
# COMPONENT TRANSLATIONS (ordered longest-first to avoid partial matches)
# ============================================================
COMPONENTS = [
    # Sensors
    ("Mass Air Flow", "Débit massique d'air"),
    ("Mass or Volume Air Flow", "Débit massique ou volumétrique d'air"),
    ("Manifold Absolute Pressure/Barometric Pressure", "Pression absolue du collecteur/pression barométrique"),
    ("Manifold Absolute Pressure", "Pression absolue du collecteur"),
    ("Barometric Pressure", "Pression barométrique"),
    ("Intake Air Temperature", "Température de l'air d'admission"),
    ("Engine Coolant Temperature", "Température du liquide de refroidissement"),
    ("Throttle/Pedal Position Sensor/Switch", "Capteur/contacteur de position du papillon/pédale"),
    ("Throttle Position Sensor/Switch", "Capteur/contacteur de position du papillon"),
    ("Throttle Position Sensor", "Capteur de position du papillon"),
    ("Pedal Position Sensor", "Capteur de position de la pédale"),
    ("Accelerator Pedal Position", "Position de la pédale d'accélérateur"),
    ("Vehicle Speed Sensor", "Capteur de vitesse du véhicule"),
    ("Knock Sensor", "Capteur de cliquetis"),
    ("Crankshaft Position Sensor", "Capteur de position du vilebrequin"),
    ("Camshaft Position Sensor", "Capteur de position de l'arbre à cames"),
    ("Crankshaft Position", "Position du vilebrequin"),
    ("Camshaft Position", "Position de l'arbre à cames"),
    ("Engine Speed Input", "Régime moteur"),
    ("Oxygen Sensor", "Sonde à oxygène"),
    ("O2 Sensor", "Sonde à oxygène"),
    ("A/F Sensor", "Sonde air/carburant"),
    ("Air-Fuel Ratio Sensor", "Sonde de rapport air/carburant"),
    ("Fuel Rail Pressure Sensor", "Capteur de pression de rampe d'injection"),
    ("Fuel Pressure Sensor", "Capteur de pression de carburant"),
    ("Fuel Pressure", "Pression de carburant"),
    ("Fuel Temperature Sensor", "Capteur de température de carburant"),
    ("Fuel Level Sensor", "Capteur de niveau de carburant"),
    ("Engine Oil Pressure Sensor/Switch", "Capteur/contacteur de pression d'huile moteur"),
    ("Engine Oil Pressure Sensor", "Capteur de pression d'huile moteur"),
    ("Engine Oil Temperature Sensor", "Capteur de température d'huile moteur"),
    ("Engine Oil Pressure", "Pression d'huile moteur"),
    ("Engine Oil Temperature", "Température d'huile moteur"),
    ("Exhaust Gas Temperature Sensor", "Capteur de température des gaz d'échappement"),
    ("Exhaust Gas Temperature", "Température des gaz d'échappement"),
    ("Catalyst Temperature Sensor", "Capteur de température du catalyseur"),
    ("Coolant Temperature Sensor", "Capteur de température du liquide de refroidissement"),
    ("Transmission Fluid Temperature Sensor", "Capteur de température du fluide de transmission"),
    ("Transmission Fluid Temperature", "Température du fluide de transmission"),
    ("Boost Pressure Sensor", "Capteur de pression de suralimentation"),
    ("A/C Refrigerant Pressure Sensor", "Capteur de pression du réfrigérant de climatisation"),
    ("Power Steering Pressure Sensor/Switch", "Capteur/contacteur de pression de direction assistée"),
    ("Power Steering Pressure Sensor", "Capteur de pression de direction assistée"),
    ("Wheel Speed Sensor", "Capteur de vitesse de roue"),
    ("Intake Manifold Pressure Sensor", "Capteur de pression du collecteur d'admission"),

    # Actuators & Systems
    ("Fuel Injector", "Injecteur de carburant"),
    ("Injector Circuit", "Circuit d'injecteur"),
    ("Fuel Volume Regulator", "Régulateur de volume de carburant"),
    ("Fuel Pump", "Pompe à carburant"),
    ("Fuel Shutoff Valve", "Vanne d'arrêt de carburant"),
    ("Cold Start Injector", "Injecteur de démarrage à froid"),
    ("Engine Shutoff Solenoid", "Solénoïde d'arrêt moteur"),
    ("Idle Air Control", "Contrôle d'air de ralenti"),
    ("Idle Control System", "Système de contrôle du ralenti"),
    ("Cruise Control", "Régulateur de vitesse"),
    ("Ignition Coil", "Bobine d'allumage"),
    ("Ignition/Distributor", "Allumage/distributeur"),
    ("Spark Plug", "Bougie d'allumage"),
    ("Glow Plug/Heater", "Bougie de préchauffage"),
    ("Glow Plug", "Bougie de préchauffage"),
    ("Turbocharger Wastegate Solenoid", "Solénoïde de wastegate du turbocompresseur"),
    ("Turbocharger Wastegate", "Wastegate du turbocompresseur"),
    ("Turbocharger Boost Sensor", "Capteur de pression de suralimentation du turbo"),
    ("Turbocharger/Supercharger", "Turbocompresseur/compresseur"),
    ("Turbocharger", "Turbocompresseur"),
    ("Supercharger", "Compresseur mécanique"),
    ("Exhaust Gas Recirculation", "Recirculation des gaz d'échappement (EGR)"),
    ("EGR", "EGR"),
    ("Evaporative Emission Control System", "Système de contrôle des émissions par évaporation (EVAP)"),
    ("EVAP", "EVAP"),
    ("Secondary Air Injection System", "Système d'injection d'air secondaire"),
    ("Catalyst System", "Système catalytique"),
    ("Catalytic Converter", "Catalyseur"),
    ("Warm Up Catalyst", "Catalyseur de préchauffage"),
    ("Main Catalyst", "Catalyseur principal"),
    ("Heated Catalyst", "Catalyseur chauffé"),
    ("Catalyst", "Catalyseur"),
    ("Injection Pump Fuel Metering Control", "Contrôle du dosage de carburant de la pompe d'injection"),
    ("Injection Timing Control", "Contrôle du calage d'injection"),
    ("Timing Reference High Resolution Signal", "Signal de référence de calage haute résolution"),
    ("Variable Valve Timing", "Calage variable des soupapes"),
    ("Intake Valve Control Solenoid", "Solénoïde de commande des soupapes d'admission"),
    ("Exhaust Valve Control Solenoid", "Solénoïde de commande des soupapes d'échappement"),
    ("Purge Control Valve", "Vanne de purge"),
    ("Vent Control", "Contrôle d'aération"),
    ("Switching Valve", "Vanne de commutation"),
    ("Control Module", "Calculateur"),
    ("ECM/PCM", "Calculateur moteur"),
    ("PCM", "Calculateur moteur"),
    ("ECM", "Calculateur moteur"),
    ("TCM", "Calculateur de transmission"),
    ("Serial Communication Link", "Liaison de communication série"),
    ("System Voltage", "Tension système"),

    # Transmission
    ("Torque Converter Clutch", "Embrayage du convertisseur de couple"),
    ("Torque Converter", "Convertisseur de couple"),
    ("Transmission Control System", "Système de contrôle de transmission"),
    ("Transmission", "Transmission"),
    ("Shift Solenoid", "Solénoïde de passage de vitesse"),
    ("Pressure Control Solenoid", "Solénoïde de contrôle de pression"),
    ("Gear Ratio", "Rapport de transmission"),
    ("Input/Turbine Speed Sensor", "Capteur de vitesse d'entrée/turbine"),
    ("Output Speed Sensor", "Capteur de vitesse de sortie"),
    ("Input Speed Sensor", "Capteur de vitesse d'entrée"),

    # Misc
    ("Malfunction Indicator Lamp", "Voyant de dysfonctionnement (MIL)"),
    ("Check Engine Light", "Voyant moteur"),
    ("Battery", "Batterie"),
    ("Alternator", "Alternateur"),
    ("Generator", "Générateur"),
    ("Starter", "Démarreur"),
    ("Immobilizer", "Antidémarrage"),
]

# ============================================================
# FAULT TYPE TRANSLATIONS
# ============================================================
FAULT_SUFFIXES = [
    # Order matters — longest match first
    ("Primary/Secondary Circuit", "Circuit primaire/secondaire"),
    ("Circuit Range/Performance", "Circuit — plage/performance"),
    ("Circuit Low Input", "Circuit — entrée basse"),
    ("Circuit High Input", "Circuit — entrée haute"),
    ("Circuit Intermittent", "Circuit — signal intermittent"),
    ("Circuit Malfunction", "Circuit — dysfonctionnement"),
    ("Circuit Open", "Circuit ouvert"),
    ("Circuit Shorted", "Circuit en court-circuit"),
    ("Circuit Short to Battery", "Circuit en court-circuit vers la batterie"),
    ("Circuit Short to Ground", "Circuit en court-circuit vers la masse"),
    ("Circuit Low", "Circuit — entrée basse"),
    ("Circuit High", "Circuit — entrée haute"),
    ("Range/Performance", "Plage/performance"),
    ("Low Input", "Entrée basse"),
    ("High Input", "Entrée haute"),
    ("Intermittent/Erratic", "Intermittent/erratique"),
    ("Intermittent", "Signal intermittent"),
    ("Malfunction", "Dysfonctionnement"),
    ("No Signal", "Pas de signal"),
    ("Too Many Pulses", "Trop d'impulsions"),
    ("Too Few Pulses", "Trop peu d'impulsions"),
    ("No Pulses", "Pas d'impulsions"),
    ("Incorrect Flow Detected", "Débit incorrect détecté"),
    ("Incorrect Flow", "Débit incorrect"),
    ("Flow Insufficient", "Débit insuffisant"),
    ("Flow Excessive", "Débit excessif"),
    ("Efficiency Below Threshold", "Efficacité en dessous du seuil"),
    ("Below Threshold", "En dessous du seuil"),
    ("Above Threshold", "Au-dessus du seuil"),
    ("Over Temperature", "Surchauffe"),
    ("Over Speed", "Survitesse"),
    ("Overboost", "Suralimentation excessive"),
    ("Underboost", "Suralimentation insuffisante"),
    ("Stuck Off", "Bloqué fermé"),
    ("Stuck On", "Bloqué ouvert"),
    ("Stuck Open", "Bloqué ouvert"),
    ("Stuck Closed", "Bloqué fermé"),
    ("Not Learned", "Non appris"),
    ("Detected", "Détecté"),
]

LOCATION_PATTERNS = [
    (r"\(Bank (\d)\s*,?\s*Sensor (\d)\)", r"(Banc \1, Capteur \2)"),
    (r"\(Bank (\d)\s*or\s*Single\s*Sensor\)", r"(Banc \1 ou capteur unique)"),
    (r"\(Bank (\d)\)", r"(Banc \1)"),
    (r"Bank (\d)", r"Banc \1"),
    (r"Cylinder (\d+)", r"Cylindre \1"),
    (r"- Cylinder (\d+)", r"— Cylindre \1"),
]


def translate_description(en_desc: str) -> str:
    """Translate an English DTC description to French using pattern matching."""
    if not en_desc or en_desc == "Reserved":
        return en_desc

    desc = en_desc.strip()

    # Special patterns
    if re.match(r"^Cylinder (\d+) Misfire Detected$", desc):
        m = re.match(r"^Cylinder (\d+) Misfire Detected$", desc)
        return f"Raté d'allumage détecté — Cylindre {m.group(1)}"

    if desc == "Random/Multiple Cylinder Misfire Detected":
        return "Ratés d'allumage aléatoires/multiples détectés"

    if re.match(r"^Misfire Detected", desc):
        return desc.replace("Misfire Detected", "Raté d'allumage détecté")

    if re.match(r"^Cylinder (\d+) Injector Circuit Low$", desc):
        m = re.match(r"^Cylinder (\d+) Injector Circuit Low$", desc)
        return f"Circuit d'injecteur cylindre {m.group(1)} — entrée basse"

    if re.match(r"^Cylinder (\d+) Injector Circuit High$", desc):
        m = re.match(r"^Cylinder (\d+) Injector Circuit High$", desc)
        return f"Circuit d'injecteur cylindre {m.group(1)} — entrée haute"

    if re.match(r"^Cylinder (\d+) Contribution/Balance", desc):
        m = re.match(r"^Cylinder (\d+)", desc)
        return f"Contribution/équilibre du cylindre {m.group(1)} — défaut"

    if re.match(r"^Injector Circuit Malfunction - Cylinder (\d+)$", desc):
        m = re.match(r"^Injector Circuit Malfunction - Cylinder (\d+)$", desc)
        return f"Dysfonctionnement du circuit d'injecteur — Cylindre {m.group(1)}"

    if re.match(r"^Fuel Injector Circuit Malfunction Cylinder (\d+)$", desc):
        m = re.match(r"^Fuel Injector Circuit Malfunction Cylinder (\d+)$", desc)
        return f"Dysfonctionnement du circuit d'injecteur — Cylindre {m.group(1)}"

    # Ignition coil pattern
    m = re.match(r"^Ignition Coil ([A-L]) Primary/Secondary Circuit", desc)
    if m:
        return f"Bobine d'allumage {m.group(1)} — circuit primaire/secondaire"

    # Shift solenoid pattern
    m = re.match(r"^Shift Solenoid ([A-F])(.*)$", desc)
    if m:
        suffix = translate_suffix(m.group(2).strip())
        return f"Solénoïde de passage de vitesse {m.group(1)}{' — ' + suffix if suffix else ''}"

    # Pressure control solenoid pattern
    m = re.match(r"^Pressure Control Solenoid ([A-F])(.*)$", desc)
    if m:
        suffix = translate_suffix(m.group(2).strip())
        return f"Solénoïde de contrôle de pression {m.group(1)}{' — ' + suffix if suffix else ''}"

    # Gear ratio pattern
    m = re.match(r"^Gear (\d+)(.*)$", desc)
    if m:
        suffix = translate_suffix(m.group(2).strip())
        return f"Rapport {m.group(1)}{' — ' + suffix if suffix else ''}"

    # Torque converter clutch solenoid
    if "Torque Converter Clutch Solenoid" in desc:
        suffix = desc.replace("Torque Converter Clutch Solenoid", "").strip()
        s = translate_suffix(suffix)
        return f"Solénoïde d'embrayage du convertisseur de couple{' — ' + s if s else ''}"

    # Generic component + fault pattern
    result = translate_generic(desc)
    if result != desc:
        return result

    # If all else fails, apply location patterns and return
    result = apply_locations(desc)
    return result


def translate_suffix(suffix: str) -> str:
    """Translate a fault suffix."""
    suffix = suffix.strip(" -—")
    for en, fr in FAULT_SUFFIXES:
        if suffix.lower() == en.lower():
            return fr
        if suffix.lower().startswith(en.lower()):
            rest = suffix[len(en):].strip()
            return fr + (" " + rest if rest else "")
    # Common simple suffixes
    simple = {
        "Performance": "Performance",
        "Electrical": "Électrique",
        "Mechanical": "Mécanique",
        "Stuck Off": "Bloqué fermé",
        "Stuck On": "Bloqué ouvert",
        "Open": "Ouvert",
        "Short": "Court-circuit",
        "Low": "Bas",
        "High": "Haut",
    }
    if suffix in simple:
        return simple[suffix]
    return suffix


def translate_generic(desc: str) -> str:
    """Try to translate using component + fault pattern."""
    best_component = None
    best_en = ""

    for en_comp, fr_comp in COMPONENTS:
        if en_comp in desc:
            if len(en_comp) > len(best_en):
                best_en = en_comp
                best_component = fr_comp

    if best_component:
        remainder = desc.replace(best_en, "").strip(" -—/")
        remainder = apply_locations(remainder)
        fr_suffix = translate_suffix(remainder) if remainder else ""

        # Build French description
        result = best_component
        if fr_suffix and fr_suffix != remainder:
            result += " — " + fr_suffix
        elif remainder:
            result += " — " + remainder

        return apply_locations(result)

    return desc


def apply_locations(text: str) -> str:
    """Apply location pattern translations (Bank X, Cylinder Y, etc.)."""
    for pattern, replacement in LOCATION_PATTERNS:
        text = re.sub(pattern, replacement, text)
    return text


def main():
    en_path = DATA_DIR / "dtc_descriptions.json"
    fr_path = DATA_DIR / "dtc_descriptions_fr.json"

    en = json.load(open(en_path))
    fr = json.load(open(fr_path))

    print(f"EN codes: {len(en)}")
    print(f"FR codes (existing): {len(fr)}")

    new_fr = dict(fr)  # Start with existing translations
    translated = 0
    kept = 0

    for code in sorted(en.keys()):
        if code in new_fr:
            kept += 1
            continue

        en_desc = en[code]
        fr_desc = translate_description(en_desc)
        new_fr[code] = fr_desc
        translated += 1

    print(f"Kept existing: {kept}")
    print(f"Translated: {translated}")
    print(f"Total FR: {len(new_fr)}")

    # Sort by code
    sorted_fr = dict(sorted(new_fr.items()))

    # Write output
    with open(fr_path, "w", encoding="utf-8") as f:
        json.dump(sorted_fr, f, ensure_ascii=False, indent=2)
    print(f"Written to {fr_path}")

    # Show some samples
    print("\n=== Sample translations ===")
    samples = ["P0700", "P0715", "P0720", "P0730", "P0750", "P0755",
               "P2000", "P2100", "P2195", "B0001", "C0035", "U0001", "U0100"]
    for s in samples:
        if s in new_fr and s not in fr:
            print(f"  {s}: {en.get(s, '?')}")
            print(f"    → {new_fr[s]}")


if __name__ == "__main__":
    main()
