"""Basic anomaly detection for vehicle diagnostic data."""

from dataclasses import dataclass
from typing import List, Optional
from i18n import t


@dataclass
class Anomaly:
    """Detected anomaly in vehicle data."""
    severity: str  # "warning", "critical"
    parameter: str
    value: float
    threshold: float
    message_fr: str
    message_en: str

    @property
    def message(self) -> str:
        from i18n import get_lang
        return self.message_fr if get_lang() == "fr" else self.message_en


# Threshold definitions for anomaly detection
THRESHOLDS = {
    0x05: {  # Coolant temperature
        "warning": 100,
        "critical": 110,
        "msg_warn_fr": "Température liquide de refroidissement élevée ({value}°C)",
        "msg_warn_en": "High coolant temperature ({value}°C)",
        "msg_crit_fr": "SURCHAUFFE MOTEUR ! Température critique ({value}°C)",
        "msg_crit_en": "ENGINE OVERHEATING! Critical temperature ({value}°C)",
    },
    0x04: {  # Engine load
        "warning_at_idle": 60,  # High load at idle RPM is suspicious
        "msg_warn_fr": "Charge moteur anormalement élevée au ralenti ({value}%)",
        "msg_warn_en": "Abnormally high engine load at idle ({value}%)",
    },
    0x42: {  # Battery voltage
        "low_warning": 11.5,
        "high_warning": 15.5,
        "msg_low_fr": "Tension batterie faible ({value}V) - Alternateur ou batterie défaillant",
        "msg_low_en": "Low battery voltage ({value}V) - Alternator or battery issue",
        "msg_high_fr": "Tension batterie trop élevée ({value}V) - Régulateur défaillant",
        "msg_high_en": "High battery voltage ({value}V) - Voltage regulator issue",
    },
    0x5C: {  # Oil temperature
        "warning": 130,
        "critical": 150,
        "msg_warn_fr": "Température huile moteur élevée ({value}°C)",
        "msg_warn_en": "High engine oil temperature ({value}°C)",
        "msg_crit_fr": "TEMPÉRATURE HUILE CRITIQUE ({value}°C) - Arrêtez le moteur !",
        "msg_crit_en": "CRITICAL OIL TEMPERATURE ({value}°C) - Stop the engine!",
    },
    0x0C: {  # RPM
        "warning": 6500,
        "critical": 7500,
        "msg_warn_fr": "Régime moteur élevé ({value} RPM)",
        "msg_warn_en": "High engine RPM ({value} RPM)",
        "msg_crit_fr": "RÉGIME MOTEUR CRITIQUE ({value} RPM) - Risque de casse !",
        "msg_crit_en": "CRITICAL RPM ({value} RPM) - Engine damage risk!",
    },
}


class AnomalyDetector:
    """Monitors vehicle parameters and detects anomalies."""

    def __init__(self):
        self.active_anomalies: List[Anomaly] = []
        self._previous_values = {}

    def check_value(self, pid: int, value: float, rpm: float = None) -> Optional[Anomaly]:
        """Check a single PID value for anomalies.

        Args:
            pid: OBD-II PID code
            value: Current value
            rpm: Current RPM (for idle-detection context)

        Returns:
            Anomaly if detected, None otherwise
        """
        if pid not in THRESHOLDS:
            return None

        thresh = THRESHOLDS[pid]

        # Temperature-based checks (coolant, oil)
        if "critical" in thresh and value >= thresh["critical"]:
            anomaly = Anomaly(
                severity="critical",
                parameter=f"PID 0x{pid:02X}",
                value=value,
                threshold=thresh["critical"],
                message_fr=thresh["msg_crit_fr"].format(value=f"{value:.1f}"),
                message_en=thresh["msg_crit_en"].format(value=f"{value:.1f}"),
            )
            self._add_anomaly(anomaly)
            return anomaly

        if "warning" in thresh and value >= thresh["warning"]:
            anomaly = Anomaly(
                severity="warning",
                parameter=f"PID 0x{pid:02X}",
                value=value,
                threshold=thresh["warning"],
                message_fr=thresh["msg_warn_fr"].format(value=f"{value:.1f}"),
                message_en=thresh["msg_warn_en"].format(value=f"{value:.1f}"),
            )
            self._add_anomaly(anomaly)
            return anomaly

        # Load at idle check
        if "warning_at_idle" in thresh and rpm is not None and rpm < 1000:
            if value >= thresh["warning_at_idle"]:
                anomaly = Anomaly(
                    severity="warning",
                    parameter=f"PID 0x{pid:02X}",
                    value=value,
                    threshold=thresh["warning_at_idle"],
                    message_fr=thresh["msg_warn_fr"].format(value=f"{value:.1f}"),
                    message_en=thresh["msg_warn_en"].format(value=f"{value:.1f}"),
                )
                self._add_anomaly(anomaly)
                return anomaly

        # Battery voltage range check
        if "low_warning" in thresh and value <= thresh["low_warning"]:
            anomaly = Anomaly(
                severity="warning",
                parameter=f"PID 0x{pid:02X}",
                value=value,
                threshold=thresh["low_warning"],
                message_fr=thresh["msg_low_fr"].format(value=f"{value:.1f}"),
                message_en=thresh["msg_low_en"].format(value=f"{value:.1f}"),
            )
            self._add_anomaly(anomaly)
            return anomaly

        if "high_warning" in thresh and value >= thresh["high_warning"]:
            anomaly = Anomaly(
                severity="warning",
                parameter=f"PID 0x{pid:02X}",
                value=value,
                threshold=thresh["high_warning"],
                message_fr=thresh["msg_high_fr"].format(value=f"{value:.1f}"),
                message_en=thresh["msg_high_en"].format(value=f"{value:.1f}"),
            )
            self._add_anomaly(anomaly)
            return anomaly

        # Clear anomaly for this PID if value is now normal
        self.active_anomalies = [a for a in self.active_anomalies if a.parameter != f"PID 0x{pid:02X}"]
        return None

    def _add_anomaly(self, anomaly: Anomaly):
        """Add or update anomaly in active list."""
        # Replace existing anomaly for same parameter
        self.active_anomalies = [a for a in self.active_anomalies if a.parameter != anomaly.parameter]
        self.active_anomalies.append(anomaly)

    def get_active_anomalies(self) -> List[Anomaly]:
        """Get all currently active anomalies."""
        return self.active_anomalies

    def has_critical(self) -> bool:
        """Check if any critical anomalies are active."""
        return any(a.severity == "critical" for a in self.active_anomalies)

    def clear(self):
        """Clear all active anomalies."""
        self.active_anomalies.clear()
