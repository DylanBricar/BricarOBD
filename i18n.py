"""Internationalization system for OBD Diagnostic Pro."""

_current_lang = "fr"
_listeners = []

TRANSLATIONS = {
    # App
    "app_name": {"fr": "OBD Diagnostic Pro", "en": "OBD Diagnostic Pro"},

    # Navigation
    "nav_connection": {"fr": "Connexion", "en": "Connection"},
    "nav_dashboard": {"fr": "Tableau de bord", "en": "Dashboard"},
    "nav_live_data": {"fr": "Données temps réel", "en": "Live Data"},
    "nav_dtc": {"fr": "Codes défaut", "en": "Fault Codes"},
    "nav_ecu": {"fr": "Info ECU", "en": "ECU Info"},
    "nav_history": {"fr": "Historique", "en": "History"},

    # Status bar
    "status_connected": {"fr": "Connecté", "en": "Connected"},
    "status_disconnected": {"fr": "Déconnecté", "en": "Disconnected"},
    "status_protocol": {"fr": "Protocole", "en": "Protocol"},
    "status_port": {"fr": "Port", "en": "Port"},

    # Connection frame
    "conn_title": {"fr": "Diagnostic du véhicule", "en": "Vehicle Diagnostic"},
    "conn_configuration": {"fr": "Configuration", "en": "Configuration"},
    "conn_port": {"fr": "Port série", "en": "Serial Port"},
    "conn_refresh": {"fr": "Actualiser", "en": "Refresh"},
    "conn_baud": {"fr": "Débit", "en": "Baud Rate"},
    "conn_connect": {"fr": "Connecter", "en": "Connect"},
    "conn_disconnect": {"fr": "Déconnecter", "en": "Disconnect"},
    "conn_status": {"fr": "État", "en": "Status"},
    "conn_protocol_label": {"fr": "Protocole détecté", "en": "Detected Protocol"},
    "conn_elm_version": {"fr": "Version ELM", "en": "ELM Version"},
    "conn_log": {"fr": "Journal de connexion", "en": "Connection Log"},
    "conn_connecting": {"fr": "Connexion en cours...", "en": "Connecting..."},
    "conn_success": {"fr": "Connexion réussie", "en": "Connected successfully"},
    "conn_failed": {"fr": "Échec de la connexion", "en": "Connection failed"},
    "conn_disconnecting": {"fr": "Déconnexion...", "en": "Disconnecting..."},
    "conn_no_port": {"fr": "Aucun port disponible", "en": "No ports available"},
    "conn_no_port_selected": {"fr": "Aucun port sélectionné", "en": "No port selected"},
    "conn_ports_refreshed": {"fr": "Ports actualisés", "en": "Ports refreshed"},

    # Dashboard
    "dash_title": {"fr": "Tableau de bord", "en": "Dashboard"},
    "dash_rpm": {"fr": "Régime moteur", "en": "Engine RPM"},
    "dash_speed": {"fr": "Vitesse", "en": "Speed"},
    "dash_coolant": {"fr": "Temp. liquide", "en": "Coolant Temp"},
    "dash_load": {"fr": "Charge moteur", "en": "Engine Load"},
    "dash_throttle": {"fr": "Position papillon", "en": "Throttle"},
    "dash_intake": {"fr": "Temp. admission", "en": "Intake Temp"},
    "dash_maf": {"fr": "Débit air", "en": "Air Flow"},
    "dash_fuel": {"fr": "Niveau carburant", "en": "Fuel Level"},
    "dash_voltage": {"fr": "Tension batterie", "en": "Battery"},
    "dash_timing": {"fr": "Avance allumage", "en": "Timing"},
    "dash_vehicle_info": {"fr": "Informations véhicule", "en": "Vehicle Information"},
    "dash_vin": {"fr": "N° de série (VIN)", "en": "VIN"},
    "dash_obd_protocol": {"fr": "Protocole OBD", "en": "OBD Protocol"},
    "dash_mil": {"fr": "Témoin moteur", "en": "Check Engine"},
    "dash_mil_on": {"fr": "Allumé", "en": "ON"},
    "dash_mil_off": {"fr": "Éteint", "en": "OFF"},
    "dash_start": {"fr": "Démarrer", "en": "Start"},
    "dash_stop": {"fr": "Arrêter", "en": "Stop"},
    "dash_help": {"fr": "Surveillance en temps réel des paramètres moteur. Connectez-vous d'abord, puis cliquez Démarrer.", "en": "Real-time engine parameter monitoring. Connect first, then click Start."},

    # DTC frame
    "dtc_title": {"fr": "Codes défaut (DTC)", "en": "Diagnostic Trouble Codes"},
    "dtc_read_all": {"fr": "Lire tous les DTC", "en": "Read All DTCs"},
    "dtc_read_pending": {"fr": "DTC en attente", "en": "Pending DTCs"},
    "dtc_read_permanent": {"fr": "DTC permanents", "en": "Permanent DTCs"},
    "dtc_clear_all": {"fr": "Effacer les DTC", "en": "Clear DTCs"},
    "dtc_code": {"fr": "Code", "en": "Code"},
    "dtc_description": {"fr": "Description", "en": "Description"},
    "dtc_status": {"fr": "État", "en": "Status"},
    "dtc_source": {"fr": "Source", "en": "Source"},
    "dtc_actions": {"fr": "Actions", "en": "Actions"},
    "dtc_search": {"fr": "Rechercher", "en": "Search"},
    "dtc_save": {"fr": "Sauvegarder", "en": "Save"},
    "dtc_save_all": {"fr": "Tout sauvegarder", "en": "Save All"},
    "dtc_export_txt": {"fr": "Exporter TXT", "en": "Export TXT"},
    "dtc_export_json": {"fr": "Exporter JSON", "en": "Export JSON"},
    "dtc_import": {"fr": "Importer", "en": "Import"},
    "dtc_no_codes": {"fr": "Aucun code défaut trouvé. Cliquez sur 'Lire tous les DTC'.", "en": "No DTCs found. Click 'Read All DTCs'."},
    "dtc_found": {"fr": "Trouvé : {count} DTC", "en": "Found: {count} DTCs"},
    "dtc_confirm_clear": {"fr": "Êtes-vous sûr ? Cela effacera tous les DTC et réinitialisera les moniteurs.", "en": "Are you sure? This will clear all DTCs and reset readiness monitors."},
    "dtc_confirm_type": {"fr": "Tapez 'EFFACER' pour confirmer", "en": "Type 'CLEAR' to confirm"},
    "dtc_confirm_word": {"fr": "EFFACER", "en": "CLEAR"},

    # Live data
    "live_title": {"fr": "Données temps réel", "en": "Live Data"},
    "live_start": {"fr": "Démarrer", "en": "Start"},
    "live_stop": {"fr": "Arrêter", "en": "Stop"},
    "live_refresh_rate": {"fr": "Fréquence :", "en": "Refresh Rate:"},
    "live_select_pids": {"fr": "Sélection PIDs", "en": "Select PIDs"},
    "live_select_all": {"fr": "Tout sélectionner", "en": "Select All"},
    "live_deselect_all": {"fr": "Tout désélectionner", "en": "Deselect All"},
    "live_pid": {"fr": "PID", "en": "PID"},
    "live_name": {"fr": "Paramètre", "en": "Parameter"},
    "live_value": {"fr": "Valeur", "en": "Value"},
    "live_unit": {"fr": "Unité", "en": "Unit"},
    "live_min": {"fr": "Min", "en": "Min"},
    "live_max": {"fr": "Max", "en": "Max"},
    "live_status": {"fr": "{pids} PIDs | {rate}ms | {samples} mesures", "en": "{pids} PIDs | {rate}ms | {samples} samples"},

    # ECU info
    "ecu_title": {"fr": "Informations ECU", "en": "ECU Information"},
    "ecu_scan": {"fr": "Scanner les ECU", "en": "Scan ECUs"},
    "ecu_profile": {"fr": "Profil véhicule :", "en": "Vehicle Profile:"},
    "ecu_ready": {"fr": "Prêt à scanner", "en": "Ready to scan"},
    "ecu_scanning": {"fr": "Scan en cours...", "en": "Scanning..."},
    "ecu_found": {"fr": "{count} ECU trouvé(s)", "en": "Found {count} ECU(s)"},
    "ecu_read_info": {"fr": "Lire les infos", "en": "Read Info"},
    "ecu_no_ecus": {"fr": "Aucun ECU trouvé", "en": "No ECUs found"},

    # History
    "hist_title": {"fr": "Historique des sessions", "en": "Session History"},
    "hist_refresh": {"fr": "Actualiser", "en": "Refresh"},
    "hist_sessions": {"fr": "Sessions", "en": "Sessions"},
    "hist_details": {"fr": "Détails de la session", "en": "Session Details"},
    "hist_load": {"fr": "Charger", "en": "Load"},
    "hist_delete": {"fr": "Supprimer", "en": "Delete"},
    "hist_export": {"fr": "Exporter", "en": "Export"},
    "hist_no_sessions": {"fr": "Aucune session sauvegardée.", "en": "No saved sessions."},
    "hist_no_dtcs": {"fr": "Aucun DTC dans la session", "en": "No DTCs in session"},

    # Dialogs
    "dialog_cancel": {"fr": "Annuler", "en": "Cancel"},
    "dialog_confirm": {"fr": "Confirmer", "en": "Confirm"},
    "dialog_proceed": {"fr": "Continuer", "en": "Proceed"},
    "dialog_back": {"fr": "Retour", "en": "Back"},
    "dialog_warning": {"fr": "Attention", "en": "Warning"},

    # Settings
    "settings_language": {"fr": "Langue", "en": "Language"},
    "settings_french": {"fr": "Français", "en": "French"},
    "settings_english": {"fr": "Anglais", "en": "English"},

    # Extra connection keys
    "conn_baud_rate": {"fr": "Débit (baud)", "en": "Baud Rate"},
    "conn_disconnected": {"fr": "Déconnecté", "en": "Disconnected"},
    "conn_invalid_port": {"fr": "Port invalide", "en": "Invalid port"},
    "conn_not_detected": {"fr": "Non détecté", "en": "Not detected"},
    "conn_demo": {"fr": "Mode démo (données simulées)", "en": "Demo mode (simulated data)"},

    # Extra dashboard keys
    "dash_parameters": {"fr": "Paramètres", "en": "Parameters"},

    # Extra DTC keys
    "dtc_cancel": {"fr": "Annuler", "en": "Cancel"},
    "dtc_confirm": {"fr": "Confirmer", "en": "Confirm"},
    "dtc_confirm_yes": {"fr": "Continuer", "en": "Proceed"},
    "dtc_desc": {"fr": "Description", "en": "Description"},
    "dtc_final_confirm": {"fr": "Confirmation finale", "en": "Final Confirmation"},
    "dtc_final_confirm_msg": {"fr": "Tapez le mot ci-dessous pour confirmer", "en": "Type the word below to confirm"},
    "dtc_found_zero": {"fr": "Aucun DTC trouvé", "en": "No DTCs found"},
    "dtc_incorrect": {"fr": "Texte incorrect. Réessayez.", "en": "Incorrect text. Try again."},

    # Extra ECU keys
    "ecu_click_read": {"fr": "Cliquez 'Lire les infos'", "en": "Click 'Read Info'"},
    "ecu_loading": {"fr": "Chargement...", "en": "Loading..."},
    "ecu_no_found": {"fr": "Aucun ECU trouvé", "en": "No ECUs found"},
    "ecu_read_failed": {"fr": "Lecture échouée", "en": "Read failed"},
    "ecu_scan_failed": {"fr": "Scan échoué", "en": "Scan failed"},

    # Extra history keys
    "hist_error": {"fr": "Erreur", "en": "Error"},

    # Extra live data keys
    "live_refresh": {"fr": "Fréquence :", "en": "Refresh Rate:"},
    "live_select_label": {"fr": "Sélection des PIDs à surveiller :", "en": "Select PIDs to monitor:"},
    "live_trend": {"fr": "Tendance", "en": "Trend"},

    # Help descriptions
    "conn_help": {"fr": "Configurez la connexion à votre adaptateur ELM327. Sélectionnez le port série et connectez-vous.", "en": "Configure connection to your ELM327 adapter. Select serial port and connect."},
    "dash_help": {"fr": "Surveillance temps réel des paramètres du moteur et de l'état du véhicule.", "en": "Real-time monitoring of engine parameters and vehicle status."},
    "ecu_help": {"fr": "Scanner et identifier les calculateurs (ECU) du véhicule. Affiche les informations de chaque module.", "en": "Scan and identify vehicle ECUs. Shows information for each module."},
    "hist_help": {"fr": "Historique des sessions de diagnostic sauvegardées. Chargez, exportez ou supprimez des sessions.", "en": "History of saved diagnostic sessions. Load, export, or delete sessions."},

    # Dialogs
    "hist_delete_confirm": {"fr": "Voulez-vous vraiment supprimer cette session ?", "en": "Are you sure you want to delete this session?"},

    # Help descriptions
    "live_help": {"fr": "Surveillance PIDs en temps réel. Sélectionnez les paramètres à surveiller et cliquez Démarrer.", "en": "Real-time PID monitoring. Select parameters to monitor and click Start."},
    "dtc_help": {"fr": "Lecture et gestion des codes défaut véhicule. Lisez, sauvegardez ou effacez les DTC.", "en": "Read and manage vehicle fault codes. Read, save, or clear DTCs."},

    # Dialog keys
    "dialog_confirm_operation": {"fr": "Confirmation requise", "en": "Confirmation Required"},
    "dialog_type_to_confirm": {"fr": "Tapez le mot ci-dessous pour confirmer :", "en": "Type the word below to confirm:"},
    "dialog_text_mismatch": {"fr": "Le texte ne correspond pas. Réessayez.", "en": "Text does not match. Try again."},
    "dialog_export_format": {"fr": "Format d'export", "en": "Export Format"},
    "dialog_json_format": {"fr": "Format JSON", "en": "JSON Format"},
    "dialog_txt_format": {"fr": "Format TXT", "en": "TXT Format"},
    "dialog_select_format": {"fr": "Sélectionnez le format d'export souhaité", "en": "Select desired export format"},
    "dialog_export": {"fr": "Exporter", "en": "Export"},
    "dialog_close": {"fr": "Fermer", "en": "Close"},

    # Vehicle detection
    "conn_vehicle_detected": {"fr": "Véhicule détecté : {make}", "en": "Vehicle detected: {make}"},
    "conn_vehicle_title": {"fr": "Diagnostic véhicule — {vehicle}", "en": "Vehicle Diagnostic — {vehicle}"},
}


def t(key: str, **kwargs) -> str:
    """Get translated string for current language.

    Args:
        key: Translation key
        **kwargs: Format arguments (e.g., count=5)

    Returns:
        Translated string, or key if not found
    """
    entry = TRANSLATIONS.get(key)
    if not entry:
        return key
    text = entry.get(_current_lang, entry.get("en", key))
    if kwargs:
        try:
            text = text.format(**kwargs)
        except (KeyError, IndexError):
            pass
    return text


def get_lang() -> str:
    """Get current language code."""
    return _current_lang


def set_lang(lang: str) -> None:
    """Set current language and notify listeners.

    Args:
        lang: Language code ('fr' or 'en')
    """
    global _current_lang
    if lang in ("fr", "en"):
        _current_lang = lang
        for callback in _listeners:
            try:
                callback(lang)
            except Exception:
                pass


def on_lang_change(callback) -> None:
    """Register a callback for language changes.

    Args:
        callback: Function(lang: str) called when language changes
    """
    if callback not in _listeners:
        _listeners.append(callback)
