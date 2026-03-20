"""VIN decoder for automatic vehicle identification."""

import logging

logger = logging.getLogger(__name__)

# WMI (World Manufacturer Identifier) - first 3 chars of VIN
# Source: ISO 3779, publicly available WMI database
WMI_DATABASE = {
    # PSA / Stellantis
    "VF3": {"make": "Peugeot", "country": "France"},
    "VF7": {"make": "Citroën", "country": "France"},
    "VR1": {"make": "Peugeot", "country": "France"},
    "VR3": {"make": "Peugeot", "country": "France"},
    "VF1": {"make": "Renault", "country": "France"},
    "VF6": {"make": "Renault", "country": "France"},
    "VF2": {"make": "Renault", "country": "France"},
    "UU1": {"make": "Dacia", "country": "Romania"},
    "UU6": {"make": "Dacia", "country": "Romania"},

    # VAG
    "WVW": {"make": "Volkswagen", "country": "Germany"},
    "WV1": {"make": "Volkswagen", "country": "Germany"},
    "WV2": {"make": "Volkswagen", "country": "Germany"},
    "WAU": {"make": "Audi", "country": "Germany"},
    "WUA": {"make": "Audi", "country": "Germany"},
    "TMB": {"make": "Skoda", "country": "Czech Republic"},
    "VSS": {"make": "Seat", "country": "Spain"},
    "WP0": {"make": "Porsche", "country": "Germany"},
    "WP1": {"make": "Porsche", "country": "Germany"},

    # BMW
    "WBA": {"make": "BMW", "country": "Germany"},
    "WBS": {"make": "BMW M", "country": "Germany"},
    "WBY": {"make": "BMW", "country": "Germany"},
    "WMW": {"make": "Mini", "country": "UK"},

    # Mercedes
    "WDB": {"make": "Mercedes-Benz", "country": "Germany"},
    "WDC": {"make": "Mercedes-Benz", "country": "Germany"},
    "WDD": {"make": "Mercedes-Benz", "country": "Germany"},
    "WDF": {"make": "Mercedes-Benz", "country": "Germany"},
    "W1K": {"make": "Mercedes-Benz", "country": "Germany"},
    "W1N": {"make": "Mercedes-Benz", "country": "Germany"},

    # Ford
    "WF0": {"make": "Ford", "country": "Germany"},
    "WFO": {"make": "Ford", "country": "Germany"},
    "1FA": {"make": "Ford", "country": "USA"},
    "1FB": {"make": "Ford", "country": "USA"},
    "1FC": {"make": "Ford", "country": "USA"},
    "1FD": {"make": "Ford", "country": "USA"},
    "1FM": {"make": "Ford", "country": "USA"},
    "1FT": {"make": "Ford", "country": "USA"},
    "2FA": {"make": "Ford", "country": "Canada"},
    "3FA": {"make": "Ford", "country": "Mexico"},

    # Toyota / Lexus
    "JTD": {"make": "Toyota", "country": "Japan"},
    "JTE": {"make": "Toyota", "country": "Japan"},
    "JTH": {"make": "Lexus", "country": "Japan"},
    "JTK": {"make": "Toyota", "country": "Japan"},
    "JTN": {"make": "Toyota", "country": "Japan"},
    "1NX": {"make": "Toyota", "country": "USA"},
    "2T1": {"make": "Toyota", "country": "Canada"},
    "4T1": {"make": "Toyota", "country": "USA"},
    "5TD": {"make": "Toyota", "country": "USA"},
    "SB1": {"make": "Toyota", "country": "UK"},

    # Honda / Acura
    "JHM": {"make": "Honda", "country": "Japan"},
    "SHH": {"make": "Honda", "country": "UK"},
    "1HG": {"make": "Honda", "country": "USA"},
    "2HG": {"make": "Honda", "country": "Canada"},
    "93H": {"make": "Honda", "country": "Brazil"},
    "JH4": {"make": "Acura", "country": "Japan"},

    # Hyundai / Kia / Genesis
    "KMH": {"make": "Hyundai", "country": "South Korea"},
    "KNA": {"make": "Kia", "country": "South Korea"},
    "KNJ": {"make": "Kia", "country": "South Korea"},
    "KND": {"make": "Kia", "country": "South Korea"},
    "5NP": {"make": "Hyundai", "country": "USA"},
    "5XY": {"make": "Kia", "country": "USA"},
    "KMF": {"make": "Genesis", "country": "South Korea"},

    # Fiat / Alfa Romeo / Lancia
    "ZFA": {"make": "Fiat", "country": "Italy"},
    "ZAR": {"make": "Alfa Romeo", "country": "Italy"},
    "ZLA": {"make": "Lancia", "country": "Italy"},
    "ZAM": {"make": "Maserati", "country": "Italy"},

    # Volvo
    "YV1": {"make": "Volvo", "country": "Sweden"},
    "YV4": {"make": "Volvo", "country": "Sweden"},

    # Mazda
    "JM1": {"make": "Mazda", "country": "Japan"},
    "JM3": {"make": "Mazda", "country": "Japan"},
    "JM7": {"make": "Mazda", "country": "Japan"},
    "1YV": {"make": "Mazda", "country": "USA"},

    # Subaru
    "JF1": {"make": "Subaru", "country": "Japan"},
    "JF2": {"make": "Subaru", "country": "Japan"},
    "4S3": {"make": "Subaru", "country": "USA"},
    "4S4": {"make": "Subaru", "country": "USA"},

    # Nissan / Infiniti
    "JN1": {"make": "Nissan", "country": "Japan"},
    "JN8": {"make": "Nissan", "country": "Japan"},
    "5N1": {"make": "Nissan", "country": "USA"},
    "JNK": {"make": "Infiniti", "country": "Japan"},

    # Opel / Vauxhall
    "W0L": {"make": "Opel", "country": "Germany"},
    "W0V": {"make": "Opel", "country": "Germany"},

    # Tesla
    "5YJ": {"make": "Tesla", "country": "USA"},
    "7SA": {"make": "Tesla", "country": "USA"},
}


def decode_vin(vin: str) -> dict:
    """Decode a VIN to extract manufacturer and vehicle info.

    Args:
        vin: 17-character Vehicle Identification Number

    Returns:
        Dict with keys: make, country, model_year, valid, vin, wmi
    """
    result = {
        "vin": vin,
        "make": "Unknown",
        "country": "Unknown",
        "model_year": "",
        "valid": False,
        "wmi": "",
    }

    if not vin or len(vin) < 3:
        return result

    # Clean VIN
    vin = vin.strip().upper().replace(" ", "")
    result["vin"] = vin

    if len(vin) != 17:
        logger.warning(f"Invalid VIN length: {len(vin)} (expected 17)")
        # Still try to decode WMI

    # WMI (first 3 characters)
    wmi = vin[:3]
    result["wmi"] = wmi

    if wmi in WMI_DATABASE:
        result["make"] = WMI_DATABASE[wmi]["make"]
        result["country"] = WMI_DATABASE[wmi]["country"]
        result["valid"] = True
    else:
        # Try first 2 chars (some WMIs use only 2 significant chars)
        for key, val in WMI_DATABASE.items():
            if key[:2] == wmi[:2]:
                result["make"] = val["make"]
                result["country"] = val["country"]
                result["valid"] = True
                break

    # Model year (10th character for 2010+)
    if len(vin) >= 10:
        year_char = vin[9]
        year_map = {
            'A': '2010', 'B': '2011', 'C': '2012', 'D': '2013', 'E': '2014',
            'F': '2015', 'G': '2016', 'H': '2017', 'J': '2018', 'K': '2019',
            'L': '2020', 'M': '2021', 'N': '2022', 'P': '2023', 'R': '2024',
            'S': '2025', 'T': '2026',
            '1': '2001', '2': '2002', '3': '2003', '4': '2004', '5': '2005',
            '6': '2006', '7': '2007', '8': '2008', '9': '2009',
        }
        result["model_year"] = year_map.get(year_char, "")

    logger.info(f"VIN decoded: {result['make']} ({result['country']}) year={result['model_year']}")
    return result


def get_profile_key_for_make(make: str) -> str:
    """Get the best matching vehicle profile key for a manufacturer.

    Args:
        make: Manufacturer name from VIN decode

    Returns:
        Profile key for VEHICLE_PROFILES dict, or "generic"
    """
    make_lower = make.lower()

    profile_map = {
        "peugeot": "peugeot_207",
        "citroën": "citroen_c3",
        "citroen": "citroen_c3",
        "renault": "renault_clio",
        "dacia": "dacia_sandero",
        "volkswagen": "vw_golf",
        "audi": "audi_a3",
        "skoda": "skoda_octavia",
        "seat": "seat_leon",
        "porsche": "vw_golf",
        "bmw": "bmw_serie_3",
        "mini": "mini_cooper",
        "mercedes-benz": "mercedes_classe_c",
        "mercedes": "mercedes_classe_c",
        "ford": "ford_focus",
        "toyota": "toyota_corolla",
        "lexus": "toyota_corolla",
        "honda": "honda_civic",
        "acura": "honda_civic",
        "hyundai": "hyundai_i30",
        "kia": "kia_ceed",
        "genesis": "hyundai_i30",
        "fiat": "fiat_500",
        "alfa romeo": "alfa_giulia",
        "lancia": "fiat_500",
        "volvo": "volvo_xc60",
        "mazda": "mazda_3",
        "subaru": "subaru_impreza",
        "nissan": "nissan_qashqai",
        "infiniti": "generic",
        "opel": "opel_corsa",
        "tesla": "generic",
    }

    return profile_map.get(make_lower, "generic")
