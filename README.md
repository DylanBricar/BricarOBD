# BricarOBD

**Outil de diagnostic OBD-II professionnel** — Connectez-vous à n'importe quel véhicule via un adaptateur ELM327 et accédez à toutes les données de diagnostic en temps réel.

![BricarOBD Screenshot](docs/screenshot.png)

## Fonctionnalités

- **Détection automatique du véhicule** — Lit le VIN, identifie la marque (39 constructeurs supportés) et charge le profil ECU correspondant
- **Tableau de bord temps réel** — Jauges RPM, vitesse, température, charge moteur + 6 cartes de données avec seuils d'alerte
- **Monitoring PIDs** — 70 paramètres OBD-II avec suivi min/max et barre de progression
- **Codes défaut (DTC)** — Lecture, sauvegarde, export JSON et effacement avec double confirmation + backup automatique
- **Scan ECU étendu** — 7 adresses standard + adresses constructeur spécifiques (jusqu'à 20 ECUs pour PSA)
- **Détection d'anomalies** — Alertes automatiques surchauffe, batterie faible, régime critique
- **Bilingue FR/EN** — Interface complète en français et anglais
- **Mode démo** — Testez l'application sans adaptateur avec des données simulées

## Constructeurs supportés

| Groupe | Marques |
|--------|---------|
| **PSA/Stellantis** | Peugeot, Citroën, DS, Opel |
| **VAG** | Volkswagen, Audi, Seat, Škoda, Porsche |
| **Allemand** | BMW, Mini, Mercedes-Benz |
| **Américain** | Ford, Lincoln |
| **Japonais** | Toyota, Lexus, Honda, Acura, Mazda, Subaru, Nissan, Infiniti |
| **Coréen** | Hyundai, Kia, Genesis |
| **Italien** | Fiat, Alfa Romeo, Lancia, Abarth, Maserati |
| **Suédois** | Volvo |
| **Français** | Renault, Dacia |

## Installation

### Prérequis

- Python 3.9+
- tkinter (inclus avec Python sur macOS/Windows)
- Adaptateur ELM327 USB ou Bluetooth

### Setup

```bash
git clone git@github.com:DylanBricar/BricarOBD.git
cd BricarOBD
pip install -r requirements.txt
```

### Lancement

```bash
# Mode normal (avec adaptateur ELM327 branché)
python main.py

# Mode démo (sans adaptateur, données simulées)
python main.py --demo
```

## Architecture

```
BricarOBD/
├── main.py                    # Point d'entrée
├── config.py                  # Configuration application
├── i18n.py                    # Traductions FR/EN (149 clés)
├── obd_core/                  # Couche protocole OBD
│   ├── connection.py          # Gestion ELM327 (série, AT commands, P3 timing)
│   ├── obd_reader.py          # Lecture OBD-II (Modes 01-0A, 70 PIDs)
│   ├── uds_client.py          # Client UDS (Services 0x10-0x3E)
│   ├── dtc_manager.py         # Gestion DTC (lecture/effacement/export)
│   ├── safety.py              # Garde-fous sécurité (default-deny, 11 services bloqués)
│   ├── pid_definitions.py     # 70 PIDs avec formules SAE J1979
│   ├── ecu_database.py        # 39 constructeurs, 230 DIDs étendus
│   ├── vin_decoder.py         # Décodeur VIN (50+ WMIs)
│   ├── demo_mode.py           # Simulation véhicule (Peugeot 207)
│   └── anomaly_detector.py    # Détection surchauffe/batterie/régime
├── gui/                       # Interface CustomTkinter
│   ├── app.py                 # Fenêtre principale + navigation
│   ├── theme.py               # Thème sombre + polices cross-platform
│   ├── connection_frame.py    # Connexion + auto-détection VIN
│   ├── dashboard_frame.py     # Tableau de bord temps réel
│   ├── live_data_frame.py     # Monitoring PIDs
│   ├── dtc_frame.py           # Codes défaut
│   ├── ecu_info_frame.py      # Informations ECU
│   ├── history_frame.py       # Historique sessions
│   └── dialogs.py             # Dialogues de confirmation
├── data/
│   └── dtc_descriptions.py    # 514 codes DTC avec descriptions
├── utils/
│   ├── logger.py              # Journalisation audit
│   └── web_search.py          # Recherche DTC en ligne
└── assets/
    ├── logo.png               # Logo BricarOBD
    └── icon.png               # Icône application
```

## Sécurité

BricarOBD est conçu pour être **read-only par défaut** :

- **11 services UDS bloqués** (écriture, flash, reset ECU, sécurité)
- **Default-deny** pour tout service inconnu
- **Mode 04 (effacement DTC)** nécessite double confirmation + backup automatique
- **Validation hex** sur toutes les commandes envoyées
- **Cooldown 5s** entre les effacements DTC
- **Aucune opération d'écriture** n'est possible sauf l'effacement DTC

## Compatibilité

| Plateforme | Polices | Statut |
|-----------|---------|--------|
| **macOS** | SF Pro Display, Menlo | ✅ Testé |
| **Windows** | Segoe UI, Consolas | ✅ Compatible |
| **Linux** | Helvetica, Courier | ✅ Compatible |

## Licence

[MIT License](LICENSE) — Dylan Bricar © 2026
