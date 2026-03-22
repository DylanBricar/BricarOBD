"""Common DTC repair tips for frequent codes.

Community-sourced troubleshooting guides for the most common OBD-II codes.
Each entry provides common causes, quick checks, and estimated difficulty.
"""

# difficulty: 1=DIY easy, 2=DIY intermediate, 3=mechanic recommended
REPAIR_TIPS = {
    "P0100": {
        "causes": {
            "fr": ["Capteur MAF sale ou défectueux", "Fuite d'air au niveau du filtre à air", "Connecteur/câblage endommagé"],
            "en": ["Dirty or faulty MAF sensor", "Air leak at air filter housing", "Damaged connector/wiring"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur MAF avec un spray nettoyant spécifique. Vérifier le filtre à air et les durites.",
            "en": "Clean the MAF sensor with specific MAF cleaner spray. Check air filter and hoses.",
        },
        "difficulty": 1,
    },
    "P0171": {
        "causes": {
            "fr": ["Fuite d'air (durite, joint admission)", "Capteur MAF sale", "Injecteur défectueux", "Pompe à carburant faible"],
            "en": ["Air leak (hose, intake gasket)", "Dirty MAF sensor", "Faulty injector", "Weak fuel pump"],
        },
        "quick_check": {
            "fr": "Chercher les fuites d'air au collecteur d'admission. Vérifier l'état du filtre à air et nettoyer le MAF.",
            "en": "Look for air leaks at intake manifold. Check air filter condition and clean MAF.",
        },
        "difficulty": 2,
    },
    "P0300": {
        "causes": {
            "fr": ["Bougies d'allumage usées", "Bobines d'allumage défectueuses", "Injecteurs encrassés", "Compression faible"],
            "en": ["Worn spark plugs", "Faulty ignition coils", "Clogged injectors", "Low compression"],
        },
        "quick_check": {
            "fr": "Remplacer les bougies si >30 000 km. Vérifier les bobines une par une en les intervertissant.",
            "en": "Replace spark plugs if >30,000 km. Check coils one by one by swapping them.",
        },
        "difficulty": 2,
    },
    "P0301": {
        "causes": {
            "fr": ["Bougie cylindre 1 usée", "Bobine cylindre 1 défectueuse", "Injecteur cylindre 1"],
            "en": ["Cylinder 1 worn spark plug", "Cylinder 1 faulty coil", "Cylinder 1 injector"],
        },
        "quick_check": {
            "fr": "Intervertir la bobine du cylindre 1 avec une autre. Si le raté suit la bobine, la remplacer.",
            "en": "Swap cylinder 1 coil with another. If misfire follows the coil, replace it.",
        },
        "difficulty": 1,
    },
    "P0302": {
        "causes": {
            "fr": ["Bougie cylindre 2 usée", "Bobine cylindre 2 défectueuse", "Injecteur cylindre 2"],
            "en": ["Cylinder 2 worn spark plug", "Cylinder 2 faulty coil", "Cylinder 2 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 2.", "en": "Same procedure as P0301, on cylinder 2."},
        "difficulty": 1,
    },
    "P0303": {
        "causes": {
            "fr": ["Bougie cylindre 3 usée", "Bobine cylindre 3 défectueuse", "Injecteur cylindre 3"],
            "en": ["Cylinder 3 worn spark plug", "Cylinder 3 faulty coil", "Cylinder 3 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 3.", "en": "Same procedure as P0301, on cylinder 3."},
        "difficulty": 1,
    },
    "P0304": {
        "causes": {
            "fr": ["Bougie cylindre 4 usée", "Bobine cylindre 4 défectueuse", "Injecteur cylindre 4"],
            "en": ["Cylinder 4 worn spark plug", "Cylinder 4 faulty coil", "Cylinder 4 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 4.", "en": "Same procedure as P0301, on cylinder 4."},
        "difficulty": 1,
    },
    "P0335": {
        "causes": {
            "fr": ["Capteur vilebrequin défectueux", "Câblage capteur endommagé", "Courroie/chaîne de distribution décalée"],
            "en": ["Faulty crankshaft sensor", "Damaged sensor wiring", "Timing belt/chain misaligned"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur du capteur vilebrequin. Mesurer la résistance (~500-1500 ohms).",
            "en": "Check crankshaft sensor connector. Measure resistance (~500-1500 ohms).",
        },
        "difficulty": 2,
    },
    "P0340": {
        "causes": {
            "fr": ["Capteur arbre à cames défectueux", "Câblage endommagé", "Distribution décalée"],
            "en": ["Faulty camshaft sensor", "Damaged wiring", "Timing misaligned"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur du capteur AAC. Le remplacer si la résistance est hors plage.",
            "en": "Check camshaft sensor connector. Replace if resistance is out of range.",
        },
        "difficulty": 2,
    },
    "P0420": {
        "causes": {
            "fr": ["Catalyseur usé/inefficace", "Sonde lambda aval défectueuse", "Fuite d'échappement avant catalyseur"],
            "en": ["Worn/inefficient catalytic converter", "Faulty downstream O2 sensor", "Exhaust leak before cat"],
        },
        "quick_check": {
            "fr": "Vérifier l'âge du catalyseur (>150 000 km = usure normale). Contrôler la sonde lambda aval.",
            "en": "Check catalyst age (>150,000 km = normal wear). Check downstream O2 sensor.",
        },
        "difficulty": 3,
    },
    "P0480": {
        "causes": {
            "fr": ["Ventilateur de refroidissement défectueux", "Relais ventilateur HS", "Fusible grillé", "Câblage coupé"],
            "en": ["Faulty cooling fan", "Bad fan relay", "Blown fuse", "Cut wiring"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible du ventilateur. Tester le relais. Brancher le ventilateur en direct sur la batterie.",
            "en": "Check fan fuse. Test relay. Wire fan directly to battery to test.",
        },
        "difficulty": 1,
    },
    "P0504": {
        "causes": {
            "fr": ["Contacteur de frein mal réglé", "Contacteur défectueux", "Câblage contacteur"],
            "en": ["Brake switch misadjusted", "Faulty brake switch", "Switch wiring"],
        },
        "quick_check": {
            "fr": "Régler ou remplacer le contacteur de frein (pièce ~10€, sous la pédale).",
            "en": "Adjust or replace brake light switch (~10€ part, under the pedal).",
        },
        "difficulty": 1,
    },
    "P0562": {
        "causes": {
            "fr": ["Batterie faible/en fin de vie", "Alternateur défectueux", "Mauvaise masse", "Câblage corrodé"],
            "en": ["Weak/dying battery", "Faulty alternator", "Bad ground", "Corroded wiring"],
        },
        "quick_check": {
            "fr": "Mesurer la tension batterie : >12.4V moteur coupé, >13.8V moteur tournant. Si <12V, remplacer la batterie.",
            "en": "Measure battery voltage: >12.4V engine off, >13.8V engine running. If <12V, replace battery.",
        },
        "difficulty": 1,
    },
    "U0100": {
        "causes": {
            "fr": ["Problème réseau CAN", "ECU moteur ne communique pas", "Câblage CAN endommagé"],
            "en": ["CAN network issue", "Engine ECU not communicating", "Damaged CAN wiring"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs de l'ECU moteur. Contrôler les résistances de terminaison CAN (120 ohms).",
            "en": "Check engine ECU connectors. Check CAN termination resistors (120 ohms).",
        },
        "difficulty": 3,
    },
    "P0016": {
        "causes": {
            "fr": ["Chaîne/courroie de distribution étirée", "Capteur arbre à cames défectueux", "Capteur vilebrequin défectueux", "Huile moteur sale"],
            "en": ["Stretched timing chain/belt", "Faulty camshaft sensor", "Faulty crankshaft sensor", "Dirty engine oil"],
        },
        "quick_check": {
            "fr": "Vérifier le niveau et l'état de l'huile moteur. Écouter un bruit de chaîne au démarrage à froid.",
            "en": "Check engine oil level and condition. Listen for chain noise on cold start.",
        },
        "difficulty": 2,
    },
    "P0299": {
        "causes": {
            "fr": ["Turbo sous-performant", "Fuite sur durite de suralimentation", "Vanne EGR bloquée ouverte", "Wastegate bloquée"],
            "en": ["Underperforming turbo", "Boost hose leak", "EGR valve stuck open", "Stuck wastegate"],
        },
        "quick_check": {
            "fr": "Vérifier visuellement les durites de suralimentation (fissures, déboîtements). Vérifier la vanne EGR.",
            "en": "Visually check boost hoses (cracks, disconnections). Check EGR valve.",
        },
        "difficulty": 2,
    },
    "P0401": {
        "causes": {
            "fr": ["Vanne EGR encrassée", "Passages EGR bouchés", "Capteur de pression différentielle EGR"],
            "en": ["Clogged EGR valve", "Blocked EGR passages", "EGR differential pressure sensor"],
        },
        "quick_check": {
            "fr": "Démonter et nettoyer la vanne EGR (calamine). Vérifier les passages dans le collecteur.",
            "en": "Remove and clean EGR valve (carbon buildup). Check intake manifold passages.",
        },
        "difficulty": 2,
    },
    "P0442": {
        "causes": {
            "fr": ["Bouchon du réservoir mal fermé", "Fuite circuit EVAP (petite)", "Vanne de purge EVAP"],
            "en": ["Loose fuel cap", "Small EVAP system leak", "EVAP purge valve"],
        },
        "quick_check": {
            "fr": "Resserrer ou remplacer le bouchon de réservoir. Effacer le code et voir s'il revient.",
            "en": "Tighten or replace fuel cap. Clear code and see if it returns.",
        },
        "difficulty": 1,
    },
    "P0507": {
        "causes": {
            "fr": ["Prise d'air au collecteur d'admission", "Vanne de ralenti encrassée", "Corps de papillon sale"],
            "en": ["Air leak at intake manifold", "Dirty idle control valve", "Dirty throttle body"],
        },
        "quick_check": {
            "fr": "Nettoyer le corps de papillon et la vanne de ralenti. Chercher les fuites d'air.",
            "en": "Clean throttle body and idle control valve. Look for air leaks.",
        },
        "difficulty": 1,
    },
}

# Forum URLs by manufacturer (for repair guide searches)
MANUFACTURER_FORUMS = {
    "Peugeot": "https://www.forum-peugeot.com/",
    "Citroën": "https://www.forum-peugeot.com/",
    "Citroen": "https://www.forum-peugeot.com/",
    "Renault": "https://www.planeterenault.com/",
    "Dacia": "https://www.planeterenault.com/",
    "Volkswagen": "https://www.forum-vw.com/",
    "Audi": "https://www.forum-audi.com/",
    "BMW": "https://www.forum-bmw.fr/",
    "Mercedes": "https://www.forum-mercedes.com/",
    "Ford": "https://www.fordfocus.org/",
    "Toyota": "https://www.forum-toyota.fr/",
    "Hyundai": "https://www.forum-hyundai.com/",
    "Fiat": "https://www.fiat-bravo.org/",
    "Opel": "https://www.forum-opel.com/",
}


def get_repair_tips(code: str) -> dict | None:
    """Get repair tips for a DTC code, or None if not available."""
    return REPAIR_TIPS.get(code)


def get_forum_url(make: str) -> str | None:
    """Get the community forum URL for a manufacturer."""
    return MANUFACTURER_FORUMS.get(make)
