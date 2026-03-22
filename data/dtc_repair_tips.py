"""Common DTC repair tips for frequent codes.

Community-sourced troubleshooting guides for the most common OBD-II codes.
Each entry provides common causes, quick checks, and estimated difficulty.
"""

# difficulty: 1=DIY easy, 2=DIY intermediate, 3=mechanic recommended
REPAIR_TIPS = {
    "P0010": {
        "causes": {
            "fr": ["Capteur position arbre à cames défectueux", "Huile moteur sale", "Chaîne de distribution étirée"],
            "en": ["Faulty camshaft position sensor", "Dirty engine oil", "Stretched timing chain"],
        },
        "quick_check": {
            "fr": "Vérifier l'état de l'huile moteur. Tester le capteur AAC avec un multimètre.",
            "en": "Check engine oil condition. Test camshaft sensor with multimeter.",
        },
        "difficulty": 2,
    },
    "P0011": {
        "causes": {
            "fr": ["Solenoïde VVT défectueuse", "Huile moteur usée", "Chaîne timing étirée", "Phaser camshaft bloqué"],
            "en": ["Faulty VVT solenoid", "Worn engine oil", "Stretched timing chain", "Stuck camshaft phaser"],
        },
        "quick_check": {
            "fr": "Remplacer l'huile et le filtre. Vérifier le connecteur de la solenoïde VVT.",
            "en": "Replace oil and filter. Check VVT solenoid connector.",
        },
        "difficulty": 2,
    },
    "P0012": {
        "causes": {
            "fr": ["Solenoïde VVT timing trop retardé", "Chaîne de distribution usée", "Capteur AAC défectueux"],
            "en": ["VVT solenoid advanced too late", "Worn timing chain", "Faulty camshaft sensor"],
        },
        "quick_check": {
            "fr": "Vérifier l'avance de la distribution. Tester la solenoïde avec un scanner.",
            "en": "Check timing advance. Test solenoid with scanner.",
        },
        "difficulty": 2,
    },
    "P0013": {
        "causes": {
            "fr": ["Solenoïde VVT banc 2 défectueuse", "Câblage endommagé", "Chaîne timing étirée"],
            "en": ["Bank 2 VVT solenoid faulty", "Damaged wiring", "Stretched timing chain"],
        },
        "quick_check": {
            "fr": "Vérifier la solenoïde VVT banc 2. Contrôler le câblage.",
            "en": "Check bank 2 VVT solenoid. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "P0014": {
        "causes": {
            "fr": ["Solenoïde VVT avance retardée banc 2", "Huile moteur sale", "Phaser défectueux"],
            "en": ["Bank 2 VVT advanced too late", "Dirty engine oil", "Faulty phaser"],
        },
        "quick_check": {
            "fr": "Changer l'huile. Vérifier la solenoïde VVT banc 2.",
            "en": "Change oil. Check bank 2 VVT solenoid.",
        },
        "difficulty": 2,
    },
    "P0015": {
        "causes": {
            "fr": ["Capteur AAC banc 2 défectueux", "Câblage capteur AAC endommagé", "Distribution décalée"],
            "en": ["Bank 2 camshaft sensor faulty", "Damaged camshaft sensor wiring", "Timing misaligned"],
        },
        "quick_check": {
            "fr": "Tester le capteur AAC banc 2. Vérifier la distribution.",
            "en": "Test bank 2 camshaft sensor. Check timing alignment.",
        },
        "difficulty": 2,
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
    "P0017": {
        "causes": {
            "fr": ["Distribution banc 2 décalée", "Chaîne timing étirée", "Capteurs AAC/vilebrequin défectueux"],
            "en": ["Bank 2 timing misaligned", "Stretched timing chain", "Faulty camshaft/crankshaft sensors"],
        },
        "quick_check": {
            "fr": "Vérifier l'alignement de la distribution banc 2.",
            "en": "Check bank 2 timing alignment.",
        },
        "difficulty": 3,
    },
    "P0018": {
        "causes": {
            "fr": ["Solenoïde VVT banc 2 avancée", "Chaîne de distribution usée", "Phaser bloqué"],
            "en": ["Bank 2 VVT solenoid advanced", "Worn timing chain", "Stuck phaser"],
        },
        "quick_check": {
            "fr": "Tester la solenoïde VVT banc 2.",
            "en": "Test bank 2 VVT solenoid.",
        },
        "difficulty": 2,
    },
    "P0019": {
        "causes": {
            "fr": ["Solenoïde VVT banc 2 retardée", "Saleté dans le système VVT", "Capteur défectueux"],
            "en": ["Bank 2 VVT solenoid retarded", "Contamination in VVT system", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Vérifier la solenoïde VVT banc 2 et le capteur AAC.",
            "en": "Check bank 2 VVT solenoid and camshaft sensor.",
        },
        "difficulty": 2,
    },
    "P0020": {
        "causes": {
            "fr": ["Solenoïde VVT banc 2 position timeout", "Électrique endommagée", "Chaîne timing étirée"],
            "en": ["Bank 2 VVT solenoid position timeout", "Damaged electrical", "Stretched timing chain"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur et le câblage de la solenoïde VVT.",
            "en": "Check VVT solenoid connector and wiring.",
        },
        "difficulty": 2,
    },
    "P0101": {
        "causes": {
            "fr": ["Capteur MAF sale ou défectueux", "Fuite d'air après MAF", "Filtre à air cloggé"],
            "en": ["Dirty or faulty MAF sensor", "Air leak after MAF", "Clogged air filter"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur MAF avec un spray spécialisé. Remplacer le filtre à air si nécessaire.",
            "en": "Clean MAF sensor with specialized spray. Replace air filter if needed.",
        },
        "difficulty": 1,
    },
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
    "P0102": {
        "causes": {
            "fr": ["Capteur MAF signal trop bas", "Câblage MAF endommagé", "Air excessif après MAF"],
            "en": ["MAF sensor signal too low", "Damaged MAF wiring", "Excessive air after MAF"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur MAF. Vérifier les connecteurs.",
            "en": "Clean MAF sensor. Check connectors.",
        },
        "difficulty": 1,
    },
    "P0103": {
        "causes": {
            "fr": ["Capteur MAF signal trop élevé", "Fuite d'air importante", "MAF défectueux"],
            "en": ["MAF sensor signal too high", "Major air leak", "Faulty MAF"],
        },
        "quick_check": {
            "fr": "Vérifier les fuites d'air. Tester le capteur MAF.",
            "en": "Check for air leaks. Test MAF sensor.",
        },
        "difficulty": 1,
    },
    "P0104": {
        "causes": {
            "fr": ["Capteur MAF intermittent", "Connecteur instable", "Câblage défectueux"],
            "en": ["Intermittent MAF sensor", "Unstable connector", "Faulty wiring"],
        },
        "quick_check": {
            "fr": "Nettoyer les connecteurs MAF. Vérifier le câblage.",
            "en": "Clean MAF connectors. Check wiring.",
        },
        "difficulty": 1,
    },
    "P0110": {
        "causes": {
            "fr": ["Capteur température air défectueux", "Câblage IAT endommagé", "Connecteur desserré"],
            "en": ["Faulty air temperature sensor", "Damaged IAT wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur IAT avec un multimètre. Vérifier les connecteurs.",
            "en": "Test IAT sensor with multimeter. Check connectors.",
        },
        "difficulty": 1,
    },
    "P0111": {
        "causes": {
            "fr": ["Capteur IAT signal intermittent", "Câblage rongé", "Connecteur corrodé"],
            "en": ["Intermittent IAT sensor signal", "Chewed wiring", "Corroded connector"],
        },
        "quick_check": {
            "fr": "Nettoyer les connecteurs. Vérifier l'intégrité du câblage.",
            "en": "Clean connectors. Check wiring integrity.",
        },
        "difficulty": 1,
    },
    "P0112": {
        "causes": {
            "fr": ["Capteur IAT signal trop bas", "Circuit court à la masse", "Capteur noyé"],
            "en": ["IAT sensor signal too low", "Short circuit to ground", "Submerged sensor"],
        },
        "quick_check": {
            "fr": "Vérifier si l'eau a pénétré le capteur IAT. Tester la résistance.",
            "en": "Check if water entered IAT sensor. Test resistance.",
        },
        "difficulty": 2,
    },
    "P0113": {
        "causes": {
            "fr": ["Capteur IAT signal trop élevé", "Circuit ouvert", "Connecteur endommagé"],
            "en": ["IAT sensor signal too high", "Open circuit", "Damaged connector"],
        },
        "quick_check": {
            "fr": "Vérifier la continuité du câblage. Tester les connecteurs.",
            "en": "Check wiring continuity. Test connectors.",
        },
        "difficulty": 1,
    },
    "P0114": {
        "causes": {
            "fr": ["Capteur IAT intermittent", "Connecteur mal engagé", "Câblage intermittent"],
            "en": ["Intermittent IAT sensor", "Poorly seated connector", "Intermittent wiring"],
        },
        "quick_check": {
            "fr": "Enfoncer les connecteurs. Vérifier le câblage visually.",
            "en": "Seat connectors properly. Visually check wiring.",
        },
        "difficulty": 1,
    },
    "P0115": {
        "causes": {
            "fr": ["Capteur température moteur défectueux", "Thermostat bloqué", "Câblage ECT endommagé"],
            "en": ["Faulty engine temperature sensor", "Stuck thermostat", "Damaged ECT wiring"],
        },
        "quick_check": {
            "fr": "Tester le capteur de température ECT. Vérifier le thermostat.",
            "en": "Test ECT sensor. Check thermostat.",
        },
        "difficulty": 1,
    },
    "P0120": {
        "causes": {
            "fr": ["Capteur position papillon défectueux", "Connecteur TPS endommagé", "Câblage coupé"],
            "en": ["Faulty throttle position sensor", "Damaged TPS connector", "Cut wiring"],
        },
        "quick_check": {
            "fr": "Tester le capteur TPS avec un multimètre en tournant le papillon.",
            "en": "Test TPS sensor with multimeter while moving throttle.",
        },
        "difficulty": 1,
    },
    "P0121": {
        "causes": {
            "fr": ["Signal TPS intermittent", "Connecteur mal engagé", "Potentiomètre usé"],
            "en": ["Intermittent TPS signal", "Loose connector", "Worn potentiometer"],
        },
        "quick_check": {
            "fr": "Nettoyer le connecteur TPS. Vérifier la résistance en bougeant le papillon.",
            "en": "Clean TPS connector. Check resistance while moving throttle.",
        },
        "difficulty": 1,
    },
    "P0122": {
        "causes": {
            "fr": ["Signal TPS trop bas", "Circuit court à la masse", "Connecteur endommagé"],
            "en": ["TPS signal too low", "Short to ground", "Damaged connector"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs TPS. Tester la continuité.",
            "en": "Check TPS connectors. Test continuity.",
        },
        "difficulty": 1,
    },
    "P0123": {
        "causes": {
            "fr": ["Signal TPS trop élevé", "Circuit ouvert", "Capteur défectueux"],
            "en": ["TPS signal too high", "Open circuit", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Tester l'intégrité du circuit TPS avec un multimètre.",
            "en": "Test TPS circuit integrity with multimeter.",
        },
        "difficulty": 1,
    },
    "P0125": {
        "causes": {
            "fr": ["Temps de chauffe moteur anormal", "Thermostat bloqué", "Capteur ECT défectueux"],
            "en": ["Abnormal engine warmup time", "Stuck thermostat", "Faulty ECT sensor"],
        },
        "quick_check": {
            "fr": "Remplacer le thermostat. Vérifier le capteur ECT.",
            "en": "Replace thermostat. Check ECT sensor.",
        },
        "difficulty": 2,
    },
    "P0128": {
        "causes": {
            "fr": ["Thermostat bloqué en position ouverte", "Capteur ECT défectueux", "Problème de circulation"],
            "en": ["Thermostat stuck open", "Faulty ECT sensor", "Circulation issue"],
        },
        "quick_check": {
            "fr": "Remplacer le thermostat. Vérifier les durites de refroidissement.",
            "en": "Replace thermostat. Check coolant hoses.",
        },
        "difficulty": 2,
    },
    "P0130": {
        "causes": {
            "fr": ["Problème général circuit sonde lambda", "Câblage endommagé", "Connecteur desserré"],
            "en": ["General O2 sensor circuit issue", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs des sondes lambda. Inspecter le câblage.",
            "en": "Check O2 sensor connectors. Inspect wiring.",
        },
        "difficulty": 1,
    },
    "P0131": {
        "causes": {
            "fr": ["Sonde lambda amont signal bas", "Sonde défectueuse", "Câblage circuit ouvert"],
            "en": ["Upstream O2 sensor low signal", "Faulty sensor", "Open circuit wiring"],
        },
        "quick_check": {
            "fr": "Tester la sonde lambda amont avec un voltmètre. Vérifier le câblage.",
            "en": "Test upstream O2 sensor with voltmeter. Check wiring.",
        },
        "difficulty": 2,
    },
    "P0132": {
        "causes": {
            "fr": ["Sonde lambda amont signal élevé", "Sonde défectueuse", "Problème chauffage"],
            "en": ["Upstream O2 sensor high signal", "Faulty sensor", "Heater issue"],
        },
        "quick_check": {
            "fr": "Vérifier le chauffage sonde lambda. Remplacer si nécessaire.",
            "en": "Check O2 sensor heater. Replace if needed.",
        },
        "difficulty": 2,
    },
    "P0133": {
        "causes": {
            "fr": ["Réponse sonde lambda amont lente", "Sonde usée", "Dépôts d'essence et huile"],
            "en": ["Upstream O2 sensor slow response", "Worn sensor", "Fuel and oil deposits"],
        },
        "quick_check": {
            "fr": "Remplacer la sonde lambda amont si l'âge dépasse 80 000 km.",
            "en": "Replace upstream O2 sensor if age exceeds 80,000 km.",
        },
        "difficulty": 2,
    },
    "P0134": {
        "causes": {
            "fr": ["Sonde lambda amont pas de signal", "Connecteur endommagé", "Sonde noyée"],
            "en": ["Upstream O2 sensor no signal", "Damaged connector", "Submerged sensor"],
        },
        "quick_check": {
            "fr": "Vérifier l'étanchéité du connecteur de la sonde lambda.",
            "en": "Check O2 sensor connector seal.",
        },
        "difficulty": 2,
    },
    "P0135": {
        "causes": {
            "fr": ["Chauffage sonde lambda amont défectueux", "Fusible grillé", "Câblage endommagé"],
            "en": ["Upstream O2 sensor heater fault", "Blown fuse", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible du chauffage O2. Tester la résistance du chauffage.",
            "en": "Check O2 heater fuse. Test heater resistance.",
        },
        "difficulty": 2,
    },
    "P0136": {
        "causes": {
            "fr": ["Problème circuit sonde lambda aval", "Connecteur desserré", "Câblage rongé"],
            "en": ["Downstream O2 sensor circuit issue", "Loose connector", "Chewed wiring"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs sonde aval. Inspecter pour les dégâts.",
            "en": "Check downstream O2 connectors. Inspect for damage.",
        },
        "difficulty": 1,
    },
    "P0137": {
        "causes": {
            "fr": ["Sonde lambda aval signal bas", "Sonde défectueuse", "Circuit ouvert"],
            "en": ["Downstream O2 sensor low signal", "Faulty sensor", "Open circuit"],
        },
        "quick_check": {
            "fr": "Tester la sonde lambda aval. Vérifier le câblage.",
            "en": "Test downstream O2 sensor. Check wiring.",
        },
        "difficulty": 2,
    },
    "P0138": {
        "causes": {
            "fr": ["Sonde lambda aval signal élevé", "Sonde défectueuse", "Court-circuit"],
            "en": ["Downstream O2 sensor high signal", "Faulty sensor", "Short circuit"],
        },
        "quick_check": {
            "fr": "Remplacer la sonde lambda aval si les tests échouent.",
            "en": "Replace downstream O2 sensor if tests fail.",
        },
        "difficulty": 2,
    },
    "P0141": {
        "causes": {
            "fr": ["Chauffage sonde lambda aval défectueux", "Fusible grillé", "Connecteur endommagé"],
            "en": ["Downstream O2 sensor heater fault", "Blown fuse", "Damaged connector"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible et le câblage du chauffage sonde aval.",
            "en": "Check fuse and wiring for downstream O2 heater.",
        },
        "difficulty": 2,
    },
    "P0150": {
        "causes": {
            "fr": ["Problème circuit sonde lambda banc 2", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Bank 2 O2 sensor circuit issue", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs sonde lambda banc 2.",
            "en": "Check bank 2 O2 sensor connectors.",
        },
        "difficulty": 1,
    },
    "P0151": {
        "causes": {
            "fr": ["Sonde lambda banc 2 amont signal bas", "Sonde défectueuse", "Câblage circuit ouvert"],
            "en": ["Bank 2 upstream O2 sensor low signal", "Faulty sensor", "Open circuit wiring"],
        },
        "quick_check": {
            "fr": "Tester la sonde lambda banc 2 amont.",
            "en": "Test bank 2 upstream O2 sensor.",
        },
        "difficulty": 2,
    },
    "P0152": {
        "causes": {
            "fr": ["Sonde lambda banc 2 amont signal élevé", "Sonde défectueuse", "Court-circuit"],
            "en": ["Bank 2 upstream O2 sensor high signal", "Faulty sensor", "Short circuit"],
        },
        "quick_check": {
            "fr": "Remplacer la sonde lambda banc 2 amont si défectueuse.",
            "en": "Replace bank 2 upstream O2 sensor if faulty.",
        },
        "difficulty": 2,
    },
    "P0153": {
        "causes": {
            "fr": ["Réponse sonde lambda banc 2 amont lente", "Sonde usée", "Dépôts de carbone"],
            "en": ["Bank 2 upstream O2 sensor slow response", "Worn sensor", "Carbon deposits"],
        },
        "quick_check": {
            "fr": "Remplacer la sonde lambda banc 2 amont.",
            "en": "Replace bank 2 upstream O2 sensor.",
        },
        "difficulty": 2,
    },
    "P0155": {
        "causes": {
            "fr": ["Chauffage sonde lambda banc 2 amont défectueux", "Fusible grillé", "Câblage endommagé"],
            "en": ["Bank 2 upstream O2 sensor heater fault", "Blown fuse", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible et le chauffage de la sonde banc 2.",
            "en": "Check fuse and heater for bank 2 O2 sensor.",
        },
        "difficulty": 2,
    },
    "P0156": {
        "causes": {
            "fr": ["Problème circuit sonde lambda banc 2 aval", "Connecteur desserré", "Câblage rongé"],
            "en": ["Bank 2 downstream O2 sensor circuit issue", "Loose connector", "Chewed wiring"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs sonde lambda banc 2 aval.",
            "en": "Check bank 2 downstream O2 connectors.",
        },
        "difficulty": 1,
    },
    "P0157": {
        "causes": {
            "fr": ["Sonde lambda banc 2 aval signal bas", "Sonde défectueuse", "Circuit ouvert"],
            "en": ["Bank 2 downstream O2 sensor low signal", "Faulty sensor", "Open circuit"],
        },
        "quick_check": {
            "fr": "Tester la sonde lambda banc 2 aval.",
            "en": "Test bank 2 downstream O2 sensor.",
        },
        "difficulty": 2,
    },
    "P0158": {
        "causes": {
            "fr": ["Sonde lambda banc 2 aval signal élevé", "Sonde défectueuse", "Court-circuit"],
            "en": ["Bank 2 downstream O2 sensor high signal", "Faulty sensor", "Short circuit"],
        },
        "quick_check": {
            "fr": "Remplacer la sonde lambda banc 2 aval.",
            "en": "Replace bank 2 downstream O2 sensor.",
        },
        "difficulty": 2,
    },
    "P0160": {
        "causes": {
            "fr": ["Problème circuit sonde lambda banc 2 aval", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Bank 2 downstream O2 sensor circuit issue", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Inspecter le circuit de la sonde lambda banc 2 aval.",
            "en": "Inspect bank 2 downstream O2 sensor circuit.",
        },
        "difficulty": 1,
    },
    "P0161": {
        "causes": {
            "fr": ["Chauffage sonde lambda banc 2 aval défectueux", "Fusible grillé", "Connecteur endommagé"],
            "en": ["Bank 2 downstream O2 sensor heater fault", "Blown fuse", "Damaged connector"],
        },
        "quick_check": {
            "fr": "Vérifier le chauffage de la sonde banc 2 aval.",
            "en": "Check bank 2 downstream O2 sensor heater.",
        },
        "difficulty": 2,
    },
    "P0170": {
        "causes": {
            "fr": ["Correction du carburant système trop riche", "Sonde lambda défectueuse", "Injecteur qui fuit"],
            "en": ["System fuel correction too rich", "Faulty O2 sensor", "Leaking injector"],
        },
        "quick_check": {
            "fr": "Vérifier l'aspect des bougies (noir = riche). Nettoyer les injecteurs si nécessaire.",
            "en": "Check spark plug color (black = rich). Clean injectors if needed.",
        },
        "difficulty": 2,
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
    "P0172": {
        "causes": {
            "fr": ["Système trop riche banc 1", "Sonde lambda défectueuse", "Injecteur qui fuit"],
            "en": ["System too rich bank 1", "Faulty O2 sensor", "Leaking injector"],
        },
        "quick_check": {
            "fr": "Vérifier les bougies pour richesse. Contrôler la sonde lambda.",
            "en": "Check spark plugs for richness. Test O2 sensor.",
        },
        "difficulty": 2,
    },
    "P0173": {
        "causes": {
            "fr": ["Système trop riche banc 2", "Sonde lambda banc 2 défectueuse", "Injecteur qui fuit"],
            "en": ["System too rich bank 2", "Faulty bank 2 O2 sensor", "Leaking injector"],
        },
        "quick_check": {
            "fr": "Vérifier la sonde lambda banc 2 et les injecteurs.",
            "en": "Check bank 2 O2 sensor and injectors.",
        },
        "difficulty": 2,
    },
    "P0174": {
        "causes": {
            "fr": ["Système trop pauvre banc 1", "Fuite d'air importante", "Pompe carburant faible"],
            "en": ["System too lean bank 1", "Major air leak", "Weak fuel pump"],
        },
        "quick_check": {
            "fr": "Chercher les fuites d'air. Vérifier la pression carburant.",
            "en": "Look for air leaks. Check fuel pressure.",
        },
        "difficulty": 2,
    },
    "P0175": {
        "causes": {
            "fr": ["Système trop pauvre banc 2", "Fuite d'air banc 2", "Pompe carburant faible"],
            "en": ["System too lean bank 2", "Air leak bank 2", "Weak fuel pump"],
        },
        "quick_check": {
            "fr": "Vérifier les fuites d'air spécifiques au banc 2.",
            "en": "Check for air leaks specific to bank 2.",
        },
        "difficulty": 2,
    },
    "P0200": {
        "causes": {
            "fr": ["Problème général circuit injecteurs", "Fusible grillé", "Relais endommagé"],
            "en": ["General injector circuit issue", "Blown fuse", "Damaged relay"],
        },
        "quick_check": {
            "fr": "Vérifier les fusibles et relais injecteurs. Écouter les injecteurs qui doivent claquer.",
            "en": "Check injector fuses and relays. Listen for injector clicks.",
        },
        "difficulty": 1,
    },
    "P0201": {
        "causes": {
            "fr": ["Injecteur cylindre 1 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 1 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 1 avec un multimètre. Vérifier le câblage.",
            "en": "Test cylinder 1 injector with multimeter. Check wiring.",
        },
        "difficulty": 2,
    },
    "P0202": {
        "causes": {
            "fr": ["Injecteur cylindre 2 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 2 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 2.",
            "en": "Test cylinder 2 injector.",
        },
        "difficulty": 2,
    },
    "P0203": {
        "causes": {
            "fr": ["Injecteur cylindre 3 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 3 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 3.",
            "en": "Test cylinder 3 injector.",
        },
        "difficulty": 2,
    },
    "P0204": {
        "causes": {
            "fr": ["Injecteur cylindre 4 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 4 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 4.",
            "en": "Test cylinder 4 injector.",
        },
        "difficulty": 2,
    },
    "P0205": {
        "causes": {
            "fr": ["Injecteur cylindre 5 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 5 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 5.",
            "en": "Test cylinder 5 injector.",
        },
        "difficulty": 2,
    },
    "P0206": {
        "causes": {
            "fr": ["Injecteur cylindre 6 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 6 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 6.",
            "en": "Test cylinder 6 injector.",
        },
        "difficulty": 2,
    },
    "P0207": {
        "causes": {
            "fr": ["Injecteur cylindre 7 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 7 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 7.",
            "en": "Test cylinder 7 injector.",
        },
        "difficulty": 2,
    },
    "P0208": {
        "causes": {
            "fr": ["Injecteur cylindre 8 défectueux", "Câblage endommagé", "Problème d'impulsion"],
            "en": ["Cylinder 8 injector faulty", "Damaged wiring", "Drive circuit issue"],
        },
        "quick_check": {
            "fr": "Tester l'injecteur cylindre 8.",
            "en": "Test cylinder 8 injector.",
        },
        "difficulty": 2,
    },
    "P0220": {
        "causes": {
            "fr": ["Capteur position papillon secondaire défectueux", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Faulty secondary throttle position sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur TPS secondaire.",
            "en": "Test secondary TPS sensor.",
        },
        "difficulty": 1,
    },
    "P0221": {
        "causes": {
            "fr": ["Signal TPS secondaire intermittent", "Connecteur mal engagé", "Câblage intermittent"],
            "en": ["Intermittent secondary TPS signal", "Loose connector", "Intermittent wiring"],
        },
        "quick_check": {
            "fr": "Nettoyer et resserrer les connecteurs TPS secondaire.",
            "en": "Clean and seat secondary TPS connectors.",
        },
        "difficulty": 1,
    },
    "P0222": {
        "causes": {
            "fr": ["Signal TPS secondaire trop bas", "Circuit court à la masse", "Capteur défectueux"],
            "en": ["Secondary TPS signal too low", "Short to ground", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs et la continuité du circuit TPS secondaire.",
            "en": "Check secondary TPS connectors and circuit continuity.",
        },
        "difficulty": 1,
    },
    "P0223": {
        "causes": {
            "fr": ["Signal TPS secondaire trop élevé", "Circuit ouvert", "Connecteur endommagé"],
            "en": ["Secondary TPS signal too high", "Open circuit", "Damaged connector"],
        },
        "quick_check": {
            "fr": "Tester l'intégrité du circuit TPS secondaire.",
            "en": "Test secondary TPS circuit integrity.",
        },
        "difficulty": 1,
    },
    "P0230": {
        "causes": {
            "fr": ["Pompe à carburant primaire défectueuse", "Fusible grillé", "Relais pompe endommagé"],
            "en": ["Faulty primary fuel pump", "Blown fuse", "Damaged pump relay"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible de la pompe. Écouter la pompe au démarrage. Tester le relais.",
            "en": "Check fuel pump fuse. Listen for pump at startup. Test relay.",
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
    "P0305": {
        "causes": {
            "fr": ["Bougie cylindre 5 usée", "Bobine cylindre 5 défectueuse", "Injecteur cylindre 5"],
            "en": ["Cylinder 5 worn spark plug", "Cylinder 5 faulty coil", "Cylinder 5 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 5.", "en": "Same procedure as P0301, on cylinder 5."},
        "difficulty": 1,
    },
    "P0306": {
        "causes": {
            "fr": ["Bougie cylindre 6 usée", "Bobine cylindre 6 défectueuse", "Injecteur cylindre 6"],
            "en": ["Cylinder 6 worn spark plug", "Cylinder 6 faulty coil", "Cylinder 6 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 6.", "en": "Same procedure as P0301, on cylinder 6."},
        "difficulty": 1,
    },
    "P0307": {
        "causes": {
            "fr": ["Bougie cylindre 7 usée", "Bobine cylindre 7 défectueuse", "Injecteur cylindre 7"],
            "en": ["Cylinder 7 worn spark plug", "Cylinder 7 faulty coil", "Cylinder 7 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 7.", "en": "Same procedure as P0301, on cylinder 7."},
        "difficulty": 1,
    },
    "P0308": {
        "causes": {
            "fr": ["Bougie cylindre 8 usée", "Bobine cylindre 8 défectueuse", "Injecteur cylindre 8"],
            "en": ["Cylinder 8 worn spark plug", "Cylinder 8 faulty coil", "Cylinder 8 injector"],
        },
        "quick_check": {"fr": "Même procédure que P0301, sur le cylindre 8.", "en": "Same procedure as P0301, on cylinder 8."},
        "difficulty": 1,
    },
    "P0325": {
        "causes": {
            "fr": ["Capteur de détonation défectueux", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Faulty knock sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur de détonation avec un multimètre. Vérifier les connecteurs.",
            "en": "Test knock sensor with multimeter. Check connectors.",
        },
        "difficulty": 2,
    },
    "P0326": {
        "causes": {
            "fr": ["Signal capteur de détonation bas", "Câblage endommagé", "Capteur défectueux"],
            "en": ["Knock sensor signal too low", "Damaged wiring", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Vérifier la continuité du circuit de détonation.",
            "en": "Check knock sensor circuit continuity.",
        },
        "difficulty": 2,
    },
    "P0327": {
        "causes": {
            "fr": ["Signal capteur de détonation élevé", "Court-circuit", "Capteur défectueux"],
            "en": ["Knock sensor signal too high", "Short circuit", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Tester les connecteurs du capteur de détonation.",
            "en": "Test knock sensor connectors.",
        },
        "difficulty": 2,
    },
    "P0328": {
        "causes": {
            "fr": ["Signal capteur de détonation intermittent", "Connecteur mal engagé", "Câblage intermittent"],
            "en": ["Intermittent knock sensor signal", "Loose connector", "Intermittent wiring"],
        },
        "quick_check": {
            "fr": "Resserrer les connecteurs du capteur de détonation.",
            "en": "Tighten knock sensor connectors.",
        },
        "difficulty": 1,
    },
    "P0329": {
        "causes": {
            "fr": ["Signal capteur détonation banc 2 hors plage", "Câblage endommagé", "Capteur défectueux"],
            "en": ["Bank 2 knock sensor signal out of range", "Damaged wiring", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Tester le capteur de détonation banc 2.",
            "en": "Test bank 2 knock sensor.",
        },
        "difficulty": 2,
    },
    "P0330": {
        "causes": {
            "fr": ["Capteur de détonation banc 2 défectueux", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Bank 2 knock sensor faulty", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur de détonation banc 2.",
            "en": "Test bank 2 knock sensor.",
        },
        "difficulty": 2,
    },
    "P0332": {
        "causes": {
            "fr": ["Signal capteur détonation banc 2 bas", "Capteur défectueux", "Circuit ouvert"],
            "en": ["Bank 2 knock sensor signal too low", "Faulty sensor", "Open circuit"],
        },
        "quick_check": {
            "fr": "Vérifier la continuité du circuit détonation banc 2.",
            "en": "Check bank 2 knock sensor circuit continuity.",
        },
        "difficulty": 2,
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
    "P0336": {
        "causes": {
            "fr": ["Signal capteur vilebrequin intermittent", "Connecteur desserré", "Câblage défectueux"],
            "en": ["Intermittent crankshaft sensor signal", "Loose connector", "Faulty wiring"],
        },
        "quick_check": {
            "fr": "Nettoyer les connecteurs du capteur vilebrequin. Vérifier le câblage.",
            "en": "Clean crankshaft sensor connectors. Check wiring.",
        },
        "difficulty": 1,
    },
    "P0337": {
        "causes": {
            "fr": ["Signal capteur vilebrequin bas", "Capteur défectueux", "Circuit ouvert"],
            "en": ["Crankshaft sensor signal too low", "Faulty sensor", "Open circuit"],
        },
        "quick_check": {
            "fr": "Tester le capteur vilebrequin avec un multimètre.",
            "en": "Test crankshaft sensor with multimeter.",
        },
        "difficulty": 2,
    },
    "P0338": {
        "causes": {
            "fr": ["Signal capteur vilebrequin élevé", "Court-circuit", "Capteur défectueux"],
            "en": ["Crankshaft sensor signal too high", "Short circuit", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Vérifier la continuité du circuit du capteur vilebrequin.",
            "en": "Check crankshaft sensor circuit continuity.",
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
    "P0341": {
        "causes": {
            "fr": ["Signal capteur AAC intermittent", "Connecteur mal engagé", "Câblage intermittent"],
            "en": ["Intermittent camshaft sensor signal", "Loose connector", "Intermittent wiring"],
        },
        "quick_check": {
            "fr": "Nettoyer les connecteurs du capteur AAC.",
            "en": "Clean camshaft sensor connectors.",
        },
        "difficulty": 1,
    },
    "P0342": {
        "causes": {
            "fr": ["Signal capteur AAC bas", "Capteur défectueux", "Circuit ouvert"],
            "en": ["Camshaft sensor signal too low", "Faulty sensor", "Open circuit"],
        },
        "quick_check": {
            "fr": "Tester le capteur AAC avec un multimètre.",
            "en": "Test camshaft sensor with multimeter.",
        },
        "difficulty": 2,
    },
    "P0343": {
        "causes": {
            "fr": ["Signal capteur AAC élevé", "Court-circuit", "Capteur défectueux"],
            "en": ["Camshaft sensor signal too high", "Short circuit", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Vérifier la continuité du circuit AAC.",
            "en": "Check camshaft sensor circuit continuity.",
        },
        "difficulty": 2,
    },
    "P0351": {
        "causes": {
            "fr": ["Bobine d'allumage 1 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 1 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {
            "fr": "Intervertir la bobine 1 avec une autre. Si le raté suit la bobine, la remplacer.",
            "en": "Swap coil 1 with another. If misfire follows the coil, replace it.",
        },
        "difficulty": 1,
    },
    "P0352": {
        "causes": {
            "fr": ["Bobine d'allumage 2 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 2 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 2.", "en": "Same procedure as P0351, on coil 2."},
        "difficulty": 1,
    },
    "P0353": {
        "causes": {
            "fr": ["Bobine d'allumage 3 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 3 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 3.", "en": "Same procedure as P0351, on coil 3."},
        "difficulty": 1,
    },
    "P0354": {
        "causes": {
            "fr": ["Bobine d'allumage 4 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 4 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 4.", "en": "Same procedure as P0351, on coil 4."},
        "difficulty": 1,
    },
    "P0355": {
        "causes": {
            "fr": ["Bobine d'allumage 5 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 5 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 5.", "en": "Same procedure as P0351, on coil 5."},
        "difficulty": 1,
    },
    "P0356": {
        "causes": {
            "fr": ["Bobine d'allumage 6 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 6 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 6.", "en": "Same procedure as P0351, on coil 6."},
        "difficulty": 1,
    },
    "P0357": {
        "causes": {
            "fr": ["Bobine d'allumage 7 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 7 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 7.", "en": "Same procedure as P0351, on coil 7."},
        "difficulty": 1,
    },
    "P0358": {
        "causes": {
            "fr": ["Bobine d'allumage 8 défectueuse", "Câblage endommagé", "Problème module d'allumage"],
            "en": ["Coil 8 faulty", "Damaged wiring", "Ignition module issue"],
        },
        "quick_check": {"fr": "Même procédure que P0351, sur la bobine 8.", "en": "Same procedure as P0351, on coil 8."},
        "difficulty": 1,
    },
    "P0400": {
        "causes": {
            "fr": ["Vanne EGR encrassée ou bloquée", "Passages EGR bouchés", "Capteur de débit EGR défectueux"],
            "en": ["Clogged or stuck EGR valve", "Blocked EGR passages", "Faulty EGR flow sensor"],
        },
        "quick_check": {
            "fr": "Démonter et nettoyer la vanne EGR (calamine importante). Vérifier les passages.",
            "en": "Remove and clean EGR valve (heavy carbon). Check passages.",
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
    "P0402": {
        "causes": {
            "fr": ["Flux EGR excessif", "Vanne EGR bloquée ouverte", "Capteur flux défectueux"],
            "en": ["Excessive EGR flow", "EGR valve stuck open", "Faulty flow sensor"],
        },
        "quick_check": {
            "fr": "Remplacer la vanne EGR si elle est bloquée ouvertement.",
            "en": "Replace EGR valve if stuck open.",
        },
        "difficulty": 2,
    },
    "P0403": {
        "causes": {
            "fr": ["Circuit de commande EGR ouvert", "Câblage endommagé", "Connecteur desserré"],
            "en": ["EGR control circuit open", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur et le câblage de la vanne EGR.",
            "en": "Check EGR valve connector and wiring.",
        },
        "difficulty": 1,
    },
    "P0404": {
        "causes": {
            "fr": ["Plage de mouvement vanne EGR insuffisante", "Vanne partiellement bloquée", "Capteur position EGR"],
            "en": ["EGR valve motion range insufficient", "Valve partially stuck", "EGR position sensor"],
        },
        "quick_check": {
            "fr": "Vérifier le mouvement libre de la vanne EGR.",
            "en": "Check EGR valve moves freely.",
        },
        "difficulty": 2,
    },
    "P0410": {
        "causes": {
            "fr": ["Système d'injection air secondaire défaillant", "Soupape d'air bloquée", "Pompe air défaillante"],
            "en": ["Secondary air injection system failure", "Stuck air valve", "Air pump failure"],
        },
        "quick_check": {
            "fr": "Tester la pompe d'air secondaire. Vérifier les soupapes.",
            "en": "Test secondary air pump. Check air valves.",
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
    "P0430": {
        "causes": {
            "fr": ["Catalyseur banc 2 usé/inefficace", "Sonde lambda aval banc 2 défectueuse", "Fuite avant catalyseur"],
            "en": ["Bank 2 catalyst worn/inefficient", "Faulty bank 2 downstream O2 sensor", "Exhaust leak before cat"],
        },
        "quick_check": {
            "fr": "Vérifier le catalyseur banc 2 et la sonde lambda aval banc 2.",
            "en": "Check bank 2 catalyst and bank 2 downstream O2 sensor.",
        },
        "difficulty": 3,
    },
    "P0440": {
        "causes": {
            "fr": ["Système EVAP défectueux", "Fuite circuit EVAP", "Vanne de purge bloquée"],
            "en": ["Faulty EVAP system", "EVAP system leak", "Stuck purge valve"],
        },
        "quick_check": {
            "fr": "Vérifier le bouchon de réservoir. Inspecter les durites EVAP.",
            "en": "Check fuel cap. Inspect EVAP hoses.",
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
    "P0455": {
        "causes": {
            "fr": ["Fuite système EVAP importante", "Durite EVAP déboîtée", "Vanne de purge endommagée"],
            "en": ["Major EVAP system leak", "Disconnected EVAP hose", "Damaged purge valve"],
        },
        "quick_check": {
            "fr": "Inspecter visuellement le circuit EVAP pour les fuites.",
            "en": "Visually inspect EVAP circuit for leaks.",
        },
        "difficulty": 2,
    },
    "P0460": {
        "causes": {
            "fr": ["Capteur niveau carburant défectueux", "Flotteur bloqué", "Câblage endommagé"],
            "en": ["Faulty fuel level sensor", "Stuck float", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Tester le capteur de niveau carburant. Vérifier le flotteur.",
            "en": "Test fuel level sensor. Check float.",
        },
        "difficulty": 2,
    },
    "P0500": {
        "causes": {
            "fr": ["Capteur vitesse défectueux", "Câblage VSS endommagé", "Problème transmission"],
            "en": ["Faulty speed sensor", "Damaged VSS wiring", "Transmission issue"],
        },
        "quick_check": {
            "fr": "Tester le capteur de vitesse. Vérifier le câblage de la transmission.",
            "en": "Test speed sensor. Check transmission wiring.",
        },
        "difficulty": 2,
    },
    "P0505": {
        "causes": {
            "fr": ["Ralenti moteur instable", "Vanne IAC encrassée", "Prise d'air au collecteur"],
            "en": ["Unstable engine idle", "Clogged IAC valve", "Air leak at manifold"],
        },
        "quick_check": {
            "fr": "Nettoyer la vanne de ralenti IAC. Chercher les fuites d'air.",
            "en": "Clean IAC valve. Look for air leaks.",
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
    "P0520": {
        "causes": {
            "fr": ["Capteur pression huile défectueux", "Câblage endommagé", "Pression huile réelle basse"],
            "en": ["Faulty oil pressure sensor", "Damaged wiring", "Low actual oil pressure"],
        },
        "quick_check": {
            "fr": "Vérifier le niveau d'huile moteur. Tester le capteur de pression.",
            "en": "Check engine oil level. Test pressure sensor.",
        },
        "difficulty": 1,
    },
    "P0530": {
        "causes": {
            "fr": ["Capteur pression A/C défectueux", "Câblage endommagé", "Problème climatisation"],
            "en": ["Faulty A/C pressure sensor", "Damaged wiring", "A/C system issue"],
        },
        "quick_check": {
            "fr": "Vérifier le circuit de climatisation et le capteur de pression.",
            "en": "Check A/C circuit and pressure sensor.",
        },
        "difficulty": 2,
    },
    "P0560": {
        "causes": {
            "fr": ["Tension système électrique hors plage", "Alternateur défectueux", "Batterie faible"],
            "en": ["System voltage out of range", "Faulty alternator", "Weak battery"],
        },
        "quick_check": {
            "fr": "Mesurer la tension batterie et l'alternateur. Remplacer si nécessaire.",
            "en": "Measure battery and alternator voltage. Replace if needed.",
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
    "P0565": {
        "causes": {
            "fr": ["Problème circuit régulateur de vitesse", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Cruise control circuit issue", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs du régulateur de vitesse.",
            "en": "Check cruise control connectors.",
        },
        "difficulty": 1,
    },
    "P0600": {
        "causes": {
            "fr": ["Erreur interne ECU", "Connecteur ECU desserré", "Problème alimentation ECU"],
            "en": ["Internal ECU error", "Loose ECU connector", "ECU power supply issue"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur principal de l'ECU. Vérifier l'alimentation.",
            "en": "Check main ECU connector. Verify power supply.",
        },
        "difficulty": 2,
    },
    "P0606": {
        "causes": {
            "fr": ["Problème interne module PCM/ECU", "Firmware corrompu", "Problème mémoire"],
            "en": ["Internal PCM/ECU module issue", "Corrupted firmware", "Memory issue"],
        },
        "quick_check": {
            "fr": "Réinitialiser l'ECU en débranchant la batterie 30 secondes.",
            "en": "Reset ECU by disconnecting battery for 30 seconds.",
        },
        "difficulty": 2,
    },
    "P0700": {
        "causes": {
            "fr": ["Problème transmission général", "Code de transmission enregistré", "Problème TCM"],
            "en": ["General transmission issue", "Transmission code set", "TCM issue"],
        },
        "quick_check": {
            "fr": "Lire les codes de transmission. Vérifier le niveau d'huile transmission.",
            "en": "Read transmission codes. Check transmission fluid level.",
        },
        "difficulty": 2,
    },
    "P0750": {
        "causes": {
            "fr": ["Relais solenoïde transmission défectueux", "Câblage endommagé", "Problème solenoïde"],
            "en": ["Transmission solenoid relay faulty", "Damaged wiring", "Solenoid issue"],
        },
        "quick_check": {
            "fr": "Tester le relais de solenoïde transmission.",
            "en": "Test transmission solenoid relay.",
        },
        "difficulty": 2,
    },
    "P1300": {
        "causes": {
            "fr": ["Problème détection ratés d'allumage", "Capteur vitesse moteur intermittent", "Problème module allumage"],
            "en": ["Misfire detection issue", "Intermittent engine speed sensor", "Ignition module issue"],
        },
        "quick_check": {
            "fr": "Vérifier les capteurs de détection ratés et l'électronique d'allumage.",
            "en": "Check misfire detection sensors and ignition electronics.",
        },
        "difficulty": 2,
    },
    "P1350": {
        "causes": {
            "fr": ["Signal détection ratés anormal", "Capteur vitesse moteur défectueux", "Câblage électronique"],
            "en": ["Abnormal misfire detection signal", "Faulty engine speed sensor", "Electronics wiring"],
        },
        "quick_check": {
            "fr": "Vérifier le capteur de vitesse moteur et l'électronique d'allumage.",
            "en": "Check engine speed sensor and ignition electronics.",
        },
        "difficulty": 2,
    },
    "P2000": {
        "causes": {
            "fr": ["Problème système réduction NOx", "Système AdBlue défectueux", "Capteur NOx endommagé"],
            "en": ["NOx reduction system issue", "Faulty AdBlue system", "Damaged NOx sensor"],
        },
        "quick_check": {
            "fr": "Vérifier le niveau et la qualité du liquide AdBlue.",
            "en": "Check AdBlue fluid level and quality.",
        },
        "difficulty": 3,
    },
    "P2100": {
        "causes": {
            "fr": ["Problème actionnaire papillon électronique", "Câblage endommagé", "Capteur papillon"],
            "en": ["Electronic throttle actuator issue", "Damaged wiring", "Throttle sensor"],
        },
        "quick_check": {
            "fr": "Vérifier l'électronique du papillon moteur.",
            "en": "Check electronic throttle electronics.",
        },
        "difficulty": 2,
    },
    "P2199": {
        "causes": {
            "fr": ["Problème général capteur température", "Câblage endommagé", "Connecteur desserré"],
            "en": ["General temperature sensor issue", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester les capteurs de température du moteur.",
            "en": "Test engine temperature sensors.",
        },
        "difficulty": 1,
    },
    "C0035": {
        "causes": {
            "fr": ["Capteur vitesse roue avant gauche défectueux", "Câblage ABS endommagé", "Connecteur desserré"],
            "en": ["Front left wheel speed sensor faulty", "Damaged ABS wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur ABS. Vérifier le câblage de la roue avant gauche.",
            "en": "Clean ABS sensor. Check front left wheel wiring.",
        },
        "difficulty": 1,
    },
    "C0040": {
        "causes": {
            "fr": ["Capteur vitesse roue avant droit défectueux", "Câblage ABS endommagé", "Connecteur desserré"],
            "en": ["Front right wheel speed sensor faulty", "Damaged ABS wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur ABS. Vérifier le câblage de la roue avant droit.",
            "en": "Clean ABS sensor. Check front right wheel wiring.",
        },
        "difficulty": 1,
    },
    "C0050": {
        "causes": {
            "fr": ["Capteur vitesse roue arrière gauche défectueux", "Câblage ABS endommagé", "Connecteur desserré"],
            "en": ["Rear left wheel speed sensor faulty", "Damaged ABS wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur ABS. Vérifier le câblage de la roue arrière gauche.",
            "en": "Clean ABS sensor. Check rear left wheel wiring.",
        },
        "difficulty": 1,
    },
    "C0055": {
        "causes": {
            "fr": ["Capteur vitesse roue arrière droit défectueux", "Câblage ABS endommagé", "Connecteur desserré"],
            "en": ["Rear right wheel speed sensor faulty", "Damaged ABS wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur ABS. Vérifier le câblage de la roue arrière droit.",
            "en": "Clean ABS sensor. Check rear right wheel wiring.",
        },
        "difficulty": 1,
    },
    "C0300": {
        "causes": {
            "fr": ["Problème système direction assistée", "Capteur couple direction défectueux", "Problème module direction"],
            "en": ["Power steering system issue", "Faulty steering torque sensor", "Steering module issue"],
        },
        "quick_check": {
            "fr": "Vérifier le niveau de liquide de direction assistée. Tester le capteur.",
            "en": "Check power steering fluid level. Test sensor.",
        },
        "difficulty": 2,
    },
    "C1200": {
        "causes": {
            "fr": ["Problème système ABS moteur de pompe", "Capteur défectueux", "Problème module ABS"],
            "en": ["ABS system pump motor issue", "Faulty sensor", "ABS module issue"],
        },
        "quick_check": {
            "fr": "Tester le module ABS et les capteurs de vitesse.",
            "en": "Test ABS module and wheel speed sensors.",
        },
        "difficulty": 3,
    },
    "C1300": {
        "causes": {
            "fr": ["Problème général ABS électronique", "Capteurs vitesse roues défectueux", "Câblage endommagé"],
            "en": ["General ABS electronics issue", "Faulty wheel speed sensors", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier tous les capteurs ABS et le câblage.",
            "en": "Check all ABS sensors and wiring.",
        },
        "difficulty": 2,
    },
    "B0001": {
        "causes": {
            "fr": ["Actionneur porte avant gauche défectueux", "Câblage endommagé", "Batterie faible"],
            "en": ["Front left door actuator faulty", "Damaged wiring", "Low battery"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie. Tester le moteur de porte avant gauche.",
            "en": "Check battery. Test front left door motor.",
        },
        "difficulty": 1,
    },
    "B0005": {
        "causes": {
            "fr": ["Actionneur porte avant droit défectueux", "Câblage endommagé", "Batterie faible"],
            "en": ["Front right door actuator faulty", "Damaged wiring", "Low battery"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie. Tester le moteur de porte avant droit.",
            "en": "Check battery. Test front right door motor.",
        },
        "difficulty": 1,
    },
    "B0010": {
        "causes": {
            "fr": ["Actionneur porte arrière défectueux", "Câblage endommagé", "Batterie faible"],
            "en": ["Rear door actuator faulty", "Damaged wiring", "Low battery"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie. Tester le moteur de porte arrière.",
            "en": "Check battery. Test rear door motor.",
        },
        "difficulty": 1,
    },
    "B1000": {
        "causes": {
            "fr": ["Problème circuit d'éclairage avant", "Ampoule grillée", "Câblage endommagé"],
            "en": ["Headlight circuit issue", "Burnt out bulb", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Remplacer les ampoules avant. Vérifier le câblage.",
            "en": "Replace headlight bulbs. Check wiring.",
        },
        "difficulty": 1,
    },
    "B1010": {
        "causes": {
            "fr": ["Problème circuit feu de position avant", "Ampoule grillée", "Connecteur desserré"],
            "en": ["Parking light circuit issue", "Burnt out bulb", "Loose connector"],
        },
        "quick_check": {
            "fr": "Remplacer les ampoules de position. Vérifier les connecteurs.",
            "en": "Replace parking light bulbs. Check connectors.",
        },
        "difficulty": 1,
    },
    "B1020": {
        "causes": {
            "fr": ["Problème circuit éclairage arrière", "Ampoule grillée", "Câblage endommagé"],
            "en": ["Rear light circuit issue", "Burnt out bulb", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Remplacer les ampoules arrière. Vérifier le câblage.",
            "en": "Replace rear light bulbs. Check wiring.",
        },
        "difficulty": 1,
    },
    "B1030": {
        "causes": {
            "fr": ["Problème circuit feu stop", "Ampoule grillée", "Connecteur de frein defectueux"],
            "en": ["Brake light circuit issue", "Burnt out bulb", "Faulty brake connector"],
        },
        "quick_check": {
            "fr": "Remplacer les ampoules de frein. Vérifier le contacteur de frein.",
            "en": "Replace brake light bulbs. Check brake switch.",
        },
        "difficulty": 1,
    },
    "B1050": {
        "causes": {
            "fr": ["Problème circuit feu de brouillard", "Ampoule grillée", "Relais endommagé"],
            "en": ["Fog light circuit issue", "Burnt out bulb", "Damaged relay"],
        },
        "quick_check": {
            "fr": "Remplacer les ampoules de brouillard. Vérifier le relais.",
            "en": "Replace fog light bulbs. Check relay.",
        },
        "difficulty": 1,
    },
    "U0001": {
        "causes": {
            "fr": ["Problème réseau CAN général", "Câblage CAN endommagé", "Nœud CAN ne répond pas"],
            "en": ["General CAN network issue", "Damaged CAN wiring", "CAN node not responding"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs CAN. Tester les résistances de terminaison CAN.",
            "en": "Check CAN connectors. Test CAN termination resistors.",
        },
        "difficulty": 3,
    },
    "U0010": {
        "causes": {
            "fr": ["Problème de communication réseau CAN", "Câblage CAN rongé", "Noeud CAN défectueux"],
            "en": ["CAN network communication issue", "Chewed CAN wiring", "Faulty CAN node"],
        },
        "quick_check": {
            "fr": "Inspecter le câblage CAN pour les dégâts. Vérifier les connecteurs.",
            "en": "Inspect CAN wiring for damage. Check connectors.",
        },
        "difficulty": 3,
    },
    "U0100": {
        "causes": {
            "fr": ["Perte de communication ECU moteur", "Câblage CAN endommagé", "ECU moteur ne répond pas"],
            "en": ["Lost communication with engine ECU", "Damaged CAN wiring", "Engine ECU not responding"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs de l'ECU moteur. Contrôler les résistances de terminaison CAN (120 ohms).",
            "en": "Check engine ECU connectors. Check CAN termination resistors (120 ohms).",
        },
        "difficulty": 3,
    },
    "U0110": {
        "causes": {
            "fr": ["Perte de communication TCM transmission", "Câblage CAN endommagé", "TCM ne répond pas"],
            "en": ["Lost communication with TCM", "Damaged CAN wiring", "TCM not responding"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs du module de transmission.",
            "en": "Check transmission module connectors.",
        },
        "difficulty": 3,
    },
    "U0155": {
        "causes": {
            "fr": ["Perte de communication module ABS", "Câblage CAN ABS endommagé", "Module ABS défectueux"],
            "en": ["Lost communication with ABS module", "Damaged ABS CAN wiring", "Faulty ABS module"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs du module ABS.",
            "en": "Check ABS module connectors.",
        },
        "difficulty": 3,
    },
    "U0400": {
        "causes": {
            "fr": ["Données invalides reçues sur CAN", "Capteur envoie des données corrompues", "Problème réseau CAN"],
            "en": ["Invalid data received on CAN", "Sensor sending corrupted data", "CAN network issue"],
        },
        "quick_check": {
            "fr": "Vérifier les connecteurs et le câblage CAN.",
            "en": "Check CAN connectors and wiring.",
        },
        "difficulty": 2,
    },
    "U0410": {
        "causes": {
            "fr": ["Données invalides du capteur", "Capteur défectueux", "Câblage endommagé"],
            "en": ["Invalid sensor data", "Faulty sensor", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Identifier le capteur et vérifier ses données.",
            "en": "Identify the sensor and check its data.",
        },
        "difficulty": 2,
    },
    "U0426": {
        "causes": {
            "fr": ["Données de configuration invalides", "ECU configuration corrompue", "Problème logiciel"],
            "en": ["Invalid configuration data", "Corrupted ECU configuration", "Software issue"],
        },
        "quick_check": {
            "fr": "Réinitialiser l'ECU. Vérifier la configuration des modules.",
            "en": "Reset ECU. Check module configuration.",
        },
        "difficulty": 3,
    },
    "P0140": {
        "causes": {
            "fr": ["Sonde lambda banc 1 capteur 2 défectueuse", "Circuit de la sonde endommagé", "Connecteur desserré"],
            "en": ["Bank 1 O2 sensor 2 faulty", "Damaged sensor circuit", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur de la sonde lambda banc 1 aval. Inspecter le câblage.",
            "en": "Check bank 1 downstream O2 sensor connector. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "P0191": {
        "causes": {
            "fr": ["Capteur pression carburant rampe défectueux", "Câblage endommagé", "Connecteur desserré", "Problème pompe carburant"],
            "en": ["Fuel rail pressure sensor faulty", "Damaged wiring", "Loose connector", "Fuel pump issue"],
        },
        "quick_check": {
            "fr": "Tester le capteur de pression carburant. Vérifier la tension à l'ECU.",
            "en": "Test fuel rail pressure sensor. Check voltage at ECU.",
        },
        "difficulty": 2,
    },
    "P0230": {
        "causes": {
            "fr": ["Relais pompe carburant défectueux", "Fusible grillé", "Câblage endommagé", "Pompe carburant en panne"],
            "en": ["Fuel pump relay faulty", "Blown fuse", "Damaged wiring", "Fuel pump failure"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible de la pompe carburant. Écouter un bruit au démarrage.",
            "en": "Check fuel pump fuse. Listen for pump noise on startup.",
        },
        "difficulty": 2,
    },
    "P0305": {
        "causes": {
            "fr": ["Cylindre 5 raté d'allumage", "Bougie d'allumage usée", "Bobine d'allumage défectueuse", "Compression faible"],
            "en": ["Cylinder 5 misfire", "Worn spark plug", "Faulty ignition coil", "Low compression"],
        },
        "quick_check": {
            "fr": "Remplacer la bougie et la bobine du cylindre 5. Vérifier la compression.",
            "en": "Replace spark plug and coil for cylinder 5. Check compression.",
        },
        "difficulty": 2,
    },
    "P0306": {
        "causes": {
            "fr": ["Cylindre 6 raté d'allumage", "Bougie d'allumage usée", "Bobine d'allumage défectueuse", "Injecteur encrassé"],
            "en": ["Cylinder 6 misfire", "Worn spark plug", "Faulty ignition coil", "Clogged injector"],
        },
        "quick_check": {
            "fr": "Remplacer la bougie et la bobine du cylindre 6. Nettoyer l'injecteur.",
            "en": "Replace spark plug and coil for cylinder 6. Clean injector.",
        },
        "difficulty": 2,
    },
    "P0307": {
        "causes": {
            "fr": ["Cylindre 7 raté d'allumage", "Huile moteur sale", "Bobine d'allumage défectueuse", "Compression faible"],
            "en": ["Cylinder 7 misfire", "Dirty engine oil", "Faulty ignition coil", "Low compression"],
        },
        "quick_check": {
            "fr": "Vérifier l'huile moteur. Remplacer la bobine et la bougie du cylindre 7.",
            "en": "Check engine oil. Replace cylinder 7 coil and spark plug.",
        },
        "difficulty": 2,
    },
    "P0308": {
        "causes": {
            "fr": ["Cylindre 8 raté d'allumage", "Problème d'injection", "Bobine d'allumage défectueuse", "Soupape encrassée"],
            "en": ["Cylinder 8 misfire", "Fuel injection issue", "Faulty ignition coil", "Clogged valve"],
        },
        "quick_check": {
            "fr": "Remplacer la bougie du cylindre 8. Nettoyer l'injecteur. Vérifier la bobine.",
            "en": "Replace cylinder 8 spark plug. Clean injector. Check coil.",
        },
        "difficulty": 2,
    },
    "P0341": {
        "causes": {
            "fr": ["Capteur position arbre à cames hors plage", "Chaîne timing étirée", "Capteur défectueux", "Câblage endommagé"],
            "en": ["Camshaft position sensor out of range", "Stretched timing chain", "Faulty sensor", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Tester le capteur AAC. Vérifier l'alignement de la distribution.",
            "en": "Test camshaft sensor. Check timing alignment.",
        },
        "difficulty": 2,
    },
    "P0342": {
        "causes": {
            "fr": ["Capteur position arbre à cames signal bas", "Connecteur desserré", "Câblage endommagé", "Capteur défectueux"],
            "en": ["Camshaft position sensor low signal", "Loose connector", "Damaged wiring", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Inspecter le connecteur du capteur AAC. Vérifier la tension.",
            "en": "Inspect camshaft sensor connector. Check voltage.",
        },
        "difficulty": 1,
    },
    "P0343": {
        "causes": {
            "fr": ["Capteur position arbre à cames signal élevé", "Court-circuit", "Capteur défectueux", "Problème ECU"],
            "en": ["Camshaft position sensor high signal", "Short circuit", "Faulty sensor", "ECU issue"],
        },
        "quick_check": {
            "fr": "Vérifier le court-circuit. Tester le capteur AAC.",
            "en": "Check for short circuit. Test camshaft sensor.",
        },
        "difficulty": 2,
    },
    "P0410": {
        "causes": {
            "fr": ["Injection air secondaire défectueuse", "Tuyau d'air endommagé", "Soupape SAI bloquée", "Pompe SAI en panne"],
            "en": ["Secondary air injection faulty", "Damaged air hose", "Stuck SAI valve", "SAI pump failure"],
        },
        "quick_check": {
            "fr": "Vérifier les tuyaux SAI. Contrôler la pompe d'air secondaire.",
            "en": "Check SAI hoses. Test secondary air pump.",
        },
        "difficulty": 2,
    },
    "P0411": {
        "causes": {
            "fr": ["Système injection air secondaire chronométrage incorrect", "Soupape SAI défectueuse", "Solenoïde SAI endommagée", "Câblage endommagé"],
            "en": ["Secondary air injection timing incorrect", "Faulty SAI valve", "Damaged SAI solenoid", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier la solenoïde SAI. Tester le système d'injection d'air.",
            "en": "Check SAI solenoid. Test secondary air system.",
        },
        "difficulty": 2,
    },
    "P0443": {
        "causes": {
            "fr": ["Circuit soupape purge EVAP défectueux", "Soupape purge encrassée", "Câblage endommagé", "Connecteur desserré"],
            "en": ["EVAP purge control valve circuit faulty", "Clogged purge valve", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Inspecter la soupape de purge EVAP. Vérifier le câblage.",
            "en": "Inspect EVAP purge valve. Check wiring.",
        },
        "difficulty": 2,
    },
    "P0444": {
        "causes": {
            "fr": ["Soupape purge EVAP ouverture impossible", "Solenoïde défectueuse", "Tuyau bloqué", "Valve encrassée"],
            "en": ["EVAP purge control valve stuck closed", "Faulty solenoid", "Blocked hose", "Clogged valve"],
        },
        "quick_check": {
            "fr": "Vérifier la soupape de purge EVAP. Nettoyer ou remplacer.",
            "en": "Check EVAP purge valve. Clean or replace.",
        },
        "difficulty": 2,
    },
    "P0445": {
        "causes": {
            "fr": ["Soupape purge EVAP fermeture impossible", "Connecteur endommagé", "Solenoïde collée", "Problème électrique"],
            "en": ["EVAP purge control valve stuck open", "Damaged connector", "Stuck solenoid", "Electrical issue"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur de la soupape EVAP. Tester la solenoïde.",
            "en": "Check EVAP valve connector. Test solenoid.",
        },
        "difficulty": 2,
    },
    "P0456": {
        "causes": {
            "fr": ["Fuite très petite système EVAP", "Capuchon réservoir desserré", "Tuyau EVAP fissure mineure", "Canister EVAP fissure"],
            "en": ["EVAP system very small leak", "Loose gas cap", "Minor EVAP hose crack", "Cracked EVAP canister"],
        },
        "quick_check": {
            "fr": "Serrer ou remplacer le capuchon du réservoir. Inspecter les tuyaux EVAP.",
            "en": "Tighten or replace gas cap. Inspect EVAP hoses.",
        },
        "difficulty": 1,
    },
    "P0461": {
        "causes": {
            "fr": ["Capteur niveau carburant signal variable", "Capteur défectueux", "Connexion endommagée", "Float réservoir bloqué"],
            "en": ["Fuel level sensor range erratic", "Faulty sensor", "Damaged connection", "Stuck fuel tank float"],
        },
        "quick_check": {
            "fr": "Tester le capteur de niveau carburant. Vérifier le flotteur.",
            "en": "Test fuel level sensor. Check tank float.",
        },
        "difficulty": 2,
    },
    "P0462": {
        "causes": {
            "fr": ["Capteur niveau carburant signal bas", "Connecteur desserré", "Câblage endommagé", "Capteur défectueux"],
            "en": ["Fuel level sensor low signal", "Loose connector", "Damaged wiring", "Faulty sensor"],
        },
        "quick_check": {
            "fr": "Vérifier le connecteur du capteur. Inspecter le câblage.",
            "en": "Check sensor connector. Inspect wiring.",
        },
        "difficulty": 1,
    },
    "P0463": {
        "causes": {
            "fr": ["Capteur niveau carburant signal élevé", "Court-circuit", "Capteur défectueux", "Problème alimentation"],
            "en": ["Fuel level sensor high signal", "Short circuit", "Faulty sensor", "Power supply issue"],
        },
        "quick_check": {
            "fr": "Vérifier les courts-circuits. Tester la tension du capteur.",
            "en": "Check for shorts. Test sensor voltage.",
        },
        "difficulty": 2,
    },
    "P0563": {
        "causes": {
            "fr": ["Tension système élevée", "Alternateur surchargeant", "Régulateur tension défectueux", "Batterie problème"],
            "en": ["System voltage high", "Overcharging alternator", "Faulty voltage regulator", "Battery issue"],
        },
        "quick_check": {
            "fr": "Tester l'alternateur avec un multimètre. Vérifier la batterie.",
            "en": "Test alternator with multimeter. Check battery.",
        },
        "difficulty": 2,
    },
    "P0571": {
        "causes": {
            "fr": ["Interrupteur frein régulateur de vitesse défectueux", "Câblage endommagé", "Connecteur desserré", "Frein électromagnétique"],
            "en": ["Cruise control brake switch faulty", "Damaged wiring", "Loose connector", "Electromagnetic brake"],
        },
        "quick_check": {
            "fr": "Vérifier l'interrupteur frein. Inspecter le câblage.",
            "en": "Check brake switch. Inspect wiring.",
        },
        "difficulty": 1,
    },
    "P0601": {
        "causes": {
            "fr": ["ECM/PCM somme de contrôle défectueuse", "Mémoire ECU corrompue", "Problème programmation", "Défaut électronique"],
            "en": ["ECM/PCM checksum error", "Corrupted ECU memory", "Programming issue", "Electronic fault"],
        },
        "quick_check": {
            "fr": "Réinitialiser l'ECU. Reprogrammer si nécessaire.",
            "en": "Reset ECU. Reprogram if necessary.",
        },
        "difficulty": 3,
    },
    "P0602": {
        "causes": {
            "fr": ["ECM/PCM ne peut pas écrire en mémoire", "Mémoire flash défectueuse", "Problème programmation", "Batterie faible"],
            "en": ["ECM/PCM programming error", "Faulty flash memory", "Programming issue", "Low battery"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie. Réinitialiser l'ECU.",
            "en": "Check battery. Reset ECU.",
        },
        "difficulty": 3,
    },
    "P0603": {
        "causes": {
            "fr": ["ECM/PCM perte du Keep-Alive Power", "Batterie faible ou déchargée", "Câblage endommagé", "Problème alternateur"],
            "en": ["ECM/PCM keep-alive memory error", "Low or dead battery", "Damaged wiring", "Alternator issue"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie et l'alternateur. Vérifier les connexions.",
            "en": "Check battery and alternator. Verify connections.",
        },
        "difficulty": 2,
    },
    "P0604": {
        "causes": {
            "fr": ["ECM/PCM erreur mémoire RAM", "Mémoire RAM défectueuse", "Court-circuit", "Problème alimentation ECU"],
            "en": ["ECM/PCM RAM error", "Faulty RAM memory", "Short circuit", "ECU power supply issue"],
        },
        "quick_check": {
            "fr": "Vérifier l'alimentation de l'ECU. Réinitialiser l'ECU.",
            "en": "Check ECU power supply. Reset ECU.",
        },
        "difficulty": 3,
    },
    "P0605": {
        "causes": {
            "fr": ["ECM/PCM erreur mémoire ROM", "Mémoire ROM corrompue", "Problème de lecture", "Défaut électronique"],
            "en": ["ECM/PCM ROM error", "Corrupted ROM memory", "Read error", "Electronic fault"],
        },
        "quick_check": {
            "fr": "Réinitialiser l'ECU. Reprogrammer si nécessaire.",
            "en": "Reset ECU. Reprogram if necessary.",
        },
        "difficulty": 3,
    },
    "P0606": {
        "causes": {
            "fr": ["ECM/PCM circuit processeur défectueux", "Processeur ECU endommagé", "Problème matériel", "Court-circuit"],
            "en": ["ECM/PCM processor fault", "Damaged ECU processor", "Hardware issue", "Short circuit"],
        },
        "quick_check": {
            "fr": "Réinitialiser l'ECU. Remplacer l'ECU si nécessaire.",
            "en": "Reset ECU. Replace ECU if necessary.",
        },
        "difficulty": 3,
    },
    "P0715": {
        "causes": {
            "fr": ["Capteur vitesse transmission signal défectueux", "Capteur endommagé", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Transmission input/turbine speed sensor faulty", "Damaged sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur de vitesse transmission. Vérifier le câblage.",
            "en": "Test transmission speed sensor. Check wiring.",
        },
        "difficulty": 2,
    },
    "P0720": {
        "causes": {
            "fr": ["Capteur vitesse arbre de sortie transmission défectueux", "Capteur endommagé", "Roulements transmission usés", "Câblage endommagé"],
            "en": ["Transmission output speed sensor faulty", "Damaged sensor", "Worn transmission bearings", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Tester le capteur de vitesse arbre de sortie. Vérifier la transmission.",
            "en": "Test output speed sensor. Check transmission.",
        },
        "difficulty": 2,
    },
    "P2096": {
        "causes": {
            "fr": ["Correction carburant post-catalyseur trop pauvre banc 1", "Sonde lambda catalyseur défectueuse", "Fuite post-catalyseur", "Problème injection"],
            "en": ["Post-catalyst fuel trim too lean bank 1", "Faulty catalyst O2 sensor", "Post-catalyst leak", "Fuel injection issue"],
        },
        "quick_check": {
            "fr": "Vérifier la sonde lambda post-catalyseur. Inspecter le catalyseur.",
            "en": "Check post-catalyst O2 sensor. Inspect catalytic converter.",
        },
        "difficulty": 2,
    },
    "P2097": {
        "causes": {
            "fr": ["Correction carburant post-catalyseur trop riche banc 1", "Injecteur qui fuit", "Sonde lambda défectueuse", "Problème carburant"],
            "en": ["Post-catalyst fuel trim too rich bank 1", "Leaking injector", "Faulty O2 sensor", "Fuel system issue"],
        },
        "quick_check": {
            "fr": "Vérifier les injecteurs. Tester la sonde lambda post-catalyseur.",
            "en": "Check injectors. Test post-catalyst O2 sensor.",
        },
        "difficulty": 2,
    },
    "P2187": {
        "causes": {
            "fr": ["Correction carburant système trop pauvre au ralenti banc 1", "Fuite d'air d'admission", "Sonde lambda défectueuse", "Problème injecteur"],
            "en": ["System fuel trim too lean at idle bank 1", "Intake air leak", "Faulty O2 sensor", "Injector issue"],
        },
        "quick_check": {
            "fr": "Vérifier les fuites d'air d'admission. Tester la sonde lambda.",
            "en": "Check intake air leaks. Test O2 sensor.",
        },
        "difficulty": 2,
    },
    "P2188": {
        "causes": {
            "fr": ["Correction carburant système trop riche au ralenti banc 1", "Injecteur encrassé ou fuiteur", "Débit d'air incorrect", "Sonde lambda défectueuse"],
            "en": ["System fuel trim too rich at idle bank 1", "Clogged or leaking injector", "Incorrect airflow", "Faulty O2 sensor"],
        },
        "quick_check": {
            "fr": "Nettoyer ou remplacer les injecteurs. Vérifier le débit d'air.",
            "en": "Clean or replace injectors. Check airflow.",
        },
        "difficulty": 2,
    },
    "C0040": {
        "causes": {
            "fr": ["Capteur vitesse roue gauche avant défectueux", "Capteur endommagé", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Left front wheel speed sensor faulty", "Damaged sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur ABS roue gauche avant. Vérifier le câblage.",
            "en": "Test left front wheel ABS sensor. Check wiring.",
        },
        "difficulty": 2,
    },
    "C0045": {
        "causes": {
            "fr": ["Capteur vitesse roue droite avant défectueux", "Capteur endommagé", "Câblage endommagé", "Roulement usé"],
            "en": ["Right front wheel speed sensor faulty", "Damaged sensor", "Damaged wiring", "Worn bearing"],
        },
        "quick_check": {
            "fr": "Tester le capteur ABS roue droite avant. Vérifier le roulement.",
            "en": "Test right front wheel ABS sensor. Check bearing.",
        },
        "difficulty": 2,
    },
    "C0050": {
        "causes": {
            "fr": ["Capteur vitesse roue gauche arrière défectueux", "Capteur endommagé", "Câblage endommagé", "Roulement usé"],
            "en": ["Left rear wheel speed sensor faulty", "Damaged sensor", "Damaged wiring", "Worn bearing"],
        },
        "quick_check": {
            "fr": "Tester le capteur ABS roue gauche arrière. Vérifier le câblage.",
            "en": "Test left rear wheel ABS sensor. Check wiring.",
        },
        "difficulty": 2,
    },
    "C0060": {
        "causes": {
            "fr": ["Capteur vitesse roue droite arrière signal erratique", "Capteur endommagé ou sale", "Câblage défectueux", "Roulement usé"],
            "en": ["Right rear wheel speed sensor erratic", "Dirty or damaged sensor", "Faulty wiring", "Worn bearing"],
        },
        "quick_check": {
            "fr": "Nettoyer le capteur ABS. Vérifier le câblage et la connexion.",
            "en": "Clean ABS sensor. Check wiring and connection.",
        },
        "difficulty": 2,
    },
    "C0065": {
        "causes": {
            "fr": ["Signal capteur vitesse roue erratique", "Capteur sale ou endommagé", "Connecteur desserré", "Roulement défectueux"],
            "en": ["Wheel speed sensor signal erratic", "Dirty or damaged sensor", "Loose connector", "Faulty bearing"],
        },
        "quick_check": {
            "fr": "Nettoyer les capteurs ABS. Vérifier tous les connecteurs.",
            "en": "Clean ABS sensors. Check all connectors.",
        },
        "difficulty": 2,
    },
    "C0710": {
        "causes": {
            "fr": ["Verrou colonne direction défectueux", "Solenoïde verrou endommagée", "Câblage endommagé", "Problème électrique"],
            "en": ["Steering column lock faulty", "Damaged lock solenoid", "Damaged wiring", "Electrical issue"],
        },
        "quick_check": {
            "fr": "Vérifier le solenoïde de verrou. Inspecter le câblage.",
            "en": "Check lock solenoid. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "C1095": {
        "causes": {
            "fr": ["Problème modulateur hydraulique ABS", "Vanne hydraulique défectueuse", "Pompe ABS en panne", "Fluide de frein faible"],
            "en": ["ABS hydraulic modulator issue", "Faulty hydraulic valve", "ABS pump failure", "Low brake fluid"],
        },
        "quick_check": {
            "fr": "Vérifier le niveau de fluide de frein. Tester le modulateur ABS.",
            "en": "Check brake fluid level. Test ABS modulator.",
        },
        "difficulty": 3,
    },
    "C1100": {
        "causes": {
            "fr": ["Vérin ABS défectueux", "Solenoïde ABS endommagée", "Câblage endommagé", "Problème pression hydraulique"],
            "en": ["ABS actuator faulty", "Damaged ABS solenoid", "Damaged wiring", "Hydraulic pressure issue"],
        },
        "quick_check": {
            "fr": "Vérifier la pression hydraulique. Tester les solenoïdes ABS.",
            "en": "Check hydraulic pressure. Test ABS solenoids.",
        },
        "difficulty": 3,
    },
    "C1105": {
        "causes": {
            "fr": ["Pompe moteur hydraulique ABS défectueuse", "Moteur pompe en panne", "Câblage endommagé", "Fusible grillé"],
            "en": ["ABS pump motor fault", "Failed pump motor", "Damaged wiring", "Blown fuse"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible pompe ABS. Tester le moteur de la pompe.",
            "en": "Check ABS pump fuse. Test pump motor.",
        },
        "difficulty": 3,
    },
    "C1515": {
        "causes": {
            "fr": ["Capteur couple direction asservie défectueux", "Capteur endommagé", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Power steering torque sensor faulty", "Damaged sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Tester le capteur couple direction. Vérifier le câblage.",
            "en": "Test steering torque sensor. Check wiring.",
        },
        "difficulty": 2,
    },
    "B0015": {
        "causes": {
            "fr": ["Capteur choc frontal défectueux", "Capteur endommagé", "Câblage endommagé", "Connecteur desserré"],
            "en": ["Front crash sensor faulty", "Damaged sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier le capteur d'impact frontal. Inspecter le câblage.",
            "en": "Check front impact sensor. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "B0020": {
        "causes": {
            "fr": ["Capteur choc arrière défectueux", "Capteur endommagé", "Câblage endommagé", "Problème électrique"],
            "en": ["Rear crash sensor faulty", "Damaged sensor", "Damaged wiring", "Electrical issue"],
        },
        "quick_check": {
            "fr": "Vérifier le capteur d'impact arrière. Inspecter le câblage.",
            "en": "Check rear impact sensor. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "B1015": {
        "causes": {
            "fr": ["Moteur vitre électrique gauche défectueux", "Moteur endommagé", "Câblage endommagé", "Interrupteur défectueux"],
            "en": ["Left power window motor faulty", "Damaged motor", "Damaged wiring", "Faulty switch"],
        },
        "quick_check": {
            "fr": "Tester le moteur de vitre. Vérifier l'interrupteur.",
            "en": "Test window motor. Check switch.",
        },
        "difficulty": 2,
    },
    "B1020": {
        "causes": {
            "fr": ["Moteur vitre électrique droite défectueux", "Moteur endommagé", "Câblage endommagé", "Fusible grillé"],
            "en": ["Right power window motor faulty", "Damaged motor", "Damaged wiring", "Blown fuse"],
        },
        "quick_check": {
            "fr": "Vérifier le fusible vitre droite. Tester le moteur.",
            "en": "Check right window fuse. Test motor.",
        },
        "difficulty": 2,
    },
    "B1200": {
        "causes": {
            "fr": ["Problème système climatisation", "Compresseur défectueux", "Câblage endommagé", "Réfrigérant manquant"],
            "en": ["Climate control system issue", "Faulty compressor", "Damaged wiring", "Missing refrigerant"],
        },
        "quick_check": {
            "fr": "Vérifier le niveau de réfrigérant. Tester le compresseur.",
            "en": "Check refrigerant level. Test compressor.",
        },
        "difficulty": 2,
    },
    "B1318": {
        "causes": {
            "fr": ["Batterie tension faible", "Batterie déchargée ou usée", "Alternateur défectueux", "Câblage endommagé"],
            "en": ["Battery voltage low", "Dead or worn battery", "Faulty alternator", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie. Tester l'alternateur.",
            "en": "Check battery. Test alternator.",
        },
        "difficulty": 1,
    },
    "B1352": {
        "causes": {
            "fr": ["Alimentation allumage manquante", "Fusible grillé", "Câblage endommagé", "Problème contacteur"],
            "en": ["Ignition power missing", "Blown fuse", "Damaged wiring", "Switch issue"],
        },
        "quick_check": {
            "fr": "Vérifier les fusibles allumage. Vérifier les connexions.",
            "en": "Check ignition fuses. Verify connections.",
        },
        "difficulty": 1,
    },
    "B1600": {
        "causes": {
            "fr": ["Système PATS/imobiliseur défectueux", "Clé PATS endommagée", "Câblage endommagé", "ECU défectueux"],
            "en": ["PATS/immobilizer system faulty", "Damaged PATS key", "Damaged wiring", "Faulty ECU"],
        },
        "quick_check": {
            "fr": "Vérifier la clé PATS. Faire programmer une nouvelle clé.",
            "en": "Check PATS key. Program new key.",
        },
        "difficulty": 3,
    },
    "B1601": {
        "causes": {
            "fr": ["Capteur PATS/imobiliseur défectueux", "Capteur endommagé", "Câblage endommagé", "Connecteur desserré"],
            "en": ["PATS/immobilizer sensor faulty", "Damaged sensor", "Damaged wiring", "Loose connector"],
        },
        "quick_check": {
            "fr": "Vérifier le capteur PATS. Inspecter le câblage.",
            "en": "Check PATS sensor. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "B1602": {
        "causes": {
            "fr": ["Lecteur PATS/imobiliseur défectueux", "Lecteur endommagé", "Câblage endommagé", "Batterie faible"],
            "en": ["PATS/immobilizer reader faulty", "Damaged reader", "Damaged wiring", "Low battery"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie. Tester le lecteur PATS.",
            "en": "Check battery. Test PATS reader.",
        },
        "difficulty": 2,
    },
    "B2799": {
        "causes": {
            "fr": ["Imobiliseur réserve de sécurité défectueux", "Batterie de secours en panne", "Problème électrique", "ECU défectueux"],
            "en": ["Immobilizer backup reserve faulty", "Dead backup battery", "Electrical issue", "Faulty ECU"],
        },
        "quick_check": {
            "fr": "Vérifier la batterie de secours. Réinitialiser l'ECU.",
            "en": "Check backup battery. Reset ECU.",
        },
        "difficulty": 3,
    },
    "U0001": {
        "causes": {
            "fr": ["Perte de communication bus CAN", "Câblage CAN endommagé", "Nœud CAN défectueux", "Problème ECU"],
            "en": ["CAN bus communication lost", "Damaged CAN wiring", "Faulty CAN node", "ECU issue"],
        },
        "quick_check": {
            "fr": "Vérifier les connexions CAN. Inspecter le câblage.",
            "en": "Check CAN connections. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "U0002": {
        "causes": {
            "fr": ["Problème communication bus CAN", "Court-circuit sur bus CAN", "Terminateur CAN endommagé", "Câblage défectueux"],
            "en": ["CAN bus problem", "CAN bus short circuit", "Damaged CAN terminator", "Faulty wiring"],
        },
        "quick_check": {
            "fr": "Vérifier les terminateurs CAN. Tester la résistance du bus.",
            "en": "Check CAN terminators. Test bus resistance.",
        },
        "difficulty": 2,
    },
    "U0003": {
        "causes": {
            "fr": ["Erreur CAN bus", "Données CAN invalides", "Nœud CAN en panne", "Problème réseau CAN"],
            "en": ["CAN bus error", "Invalid CAN data", "Failed CAN node", "CAN network issue"],
        },
        "quick_check": {
            "fr": "Scanner le réseau CAN. Identifier le nœud défectueux.",
            "en": "Scan CAN network. Identify faulty node.",
        },
        "difficulty": 2,
    },
    "U0073": {
        "causes": {
            "fr": ["Bus communication module éteint", "Module communication en panne", "Câblage bus endommagé", "Problème alimentation"],
            "en": ["Control module communication bus off", "Failed communication module", "Damaged bus wiring", "Power issue"],
        },
        "quick_check": {
            "fr": "Vérifier l'alimentation du module. Inspecter le câblage.",
            "en": "Check module power. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "U0101": {
        "causes": {
            "fr": ["Perte communication moteur ECM", "ECM en panne", "Câblage CAN endommagé", "Problème réseau"],
            "en": ["Lost communication with engine ECM", "Failed ECM", "Damaged CAN wiring", "Network issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion ECM. Tester le câblage CAN.",
            "en": "Check ECM connection. Test CAN wiring.",
        },
        "difficulty": 2,
    },
    "U0102": {
        "causes": {
            "fr": ["Perte communication transmission TCM", "TCM en panne", "Câblage endommagé", "Problème communication"],
            "en": ["Lost communication with transmission TCM", "Failed TCM", "Damaged wiring", "Communication issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion TCM. Inspecter le câblage.",
            "en": "Check TCM connection. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "U0103": {
        "causes": {
            "fr": ["Perte communication module ABS", "Module ABS en panne", "Câblage CAN endommagé", "Problème réseau"],
            "en": ["Lost communication with ABS module", "Failed ABS module", "Damaged CAN wiring", "Network issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion ABS. Tester le câblage CAN.",
            "en": "Check ABS connection. Test CAN wiring.",
        },
        "difficulty": 2,
    },
    "U0104": {
        "causes": {
            "fr": ["Perte communication module direction assistée", "Module direction en panne", "Câblage endommagé", "Problème alimentation"],
            "en": ["Lost communication with power steering module", "Failed steering module", "Damaged wiring", "Power issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion module direction. Inspecter le câblage.",
            "en": "Check steering module connection. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "U0105": {
        "causes": {
            "fr": ["Perte communication module airbag", "Module airbag en panne", "Câblage endommagé", "Court-circuit"],
            "en": ["Lost communication with airbag module", "Failed airbag module", "Damaged wiring", "Short circuit"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion airbag. Inspecter le câblage.",
            "en": "Check airbag connection. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "U0140": {
        "causes": {
            "fr": ["Perte communication module carrosserie", "Module carrosserie en panne", "Câblage endommagé", "Problème réseau"],
            "en": ["Lost communication with body module", "Failed body module", "Damaged wiring", "Network issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion module carrosserie. Tester le câblage.",
            "en": "Check body module connection. Test wiring.",
        },
        "difficulty": 2,
    },
    "U0141": {
        "causes": {
            "fr": ["Perte communication module climatisation", "Module climat en panne", "Câblage endommagé", "Problème communication"],
            "en": ["Lost communication with climate module", "Failed climate module", "Damaged wiring", "Communication issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion module climat. Inspecter le câblage.",
            "en": "Check climate module connection. Inspect wiring.",
        },
        "difficulty": 2,
    },
    "U0142": {
        "causes": {
            "fr": ["Perte communication module éclairage", "Module éclairage en panne", "Câblage endommagé", "Problème alimentation"],
            "en": ["Lost communication with lighting module", "Failed lighting module", "Damaged wiring", "Power issue"],
        },
        "quick_check": {
            "fr": "Vérifier la connexion module éclairage. Tester le câblage.",
            "en": "Check lighting module connection. Test wiring.",
        },
        "difficulty": 2,
    },
    "U0401": {
        "causes": {
            "fr": ["Données invalides reçues du ECM", "ECM envoie des données corrompues", "Problème communication CAN", "Câblage endommagé"],
            "en": ["Invalid data from ECM received", "ECM sending corrupt data", "CAN communication issue", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier les données ECM. Inspecter le câblage CAN.",
            "en": "Check ECM data. Inspect CAN wiring.",
        },
        "difficulty": 2,
    },
    "U0402": {
        "causes": {
            "fr": ["Données invalides reçues du TCM", "TCM envoie des données corrompues", "Problème transmission", "Câblage endommagé"],
            "en": ["Invalid data from TCM received", "TCM sending corrupt data", "Transmission issue", "Damaged wiring"],
        },
        "quick_check": {
            "fr": "Vérifier les données TCM. Tester la transmission.",
            "en": "Check TCM data. Test transmission.",
        },
        "difficulty": 2,
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
