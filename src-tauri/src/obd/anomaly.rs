use crate::models::PidValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub pid_name: String,
    pub level: AnomalyLevel,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnomalyLevel {
    Warning,
    Critical,
}

/// Check PID values for anomalies
pub fn check_anomalies(pids: &[PidValue]) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();

    for pid in pids {
        match pid.pid {
            // Coolant temperature
            0x05 => {
                if pid.value > 110.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Critical,
                        message: format!("Surchauffe moteur critique: {:.0}°C", pid.value),
                        value: pid.value,
                        threshold: 110.0,
                    });
                } else if pid.value > 100.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message: format!("Température élevée: {:.0}°C", pid.value),
                        value: pid.value,
                        threshold: 100.0,
                    });
                }
            }
            // Battery voltage
            0x42 => {
                if pid.value < 11.5 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Critical,
                        message: format!("Batterie faible: {:.1}V", pid.value),
                        value: pid.value,
                        threshold: 11.5,
                    });
                } else if pid.value > 15.5 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message: format!("Tension excessive: {:.1}V", pid.value),
                        value: pid.value,
                        threshold: 15.5,
                    });
                }
            }
            // RPM
            0x0C => {
                if pid.value > 7500.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Critical,
                        message: format!("RPM critique: {:.0}", pid.value),
                        value: pid.value,
                        threshold: 7500.0,
                    });
                } else if pid.value > 6500.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message: format!("RPM élevé: {:.0}", pid.value),
                        value: pid.value,
                        threshold: 6500.0,
                    });
                }
            }
            // Oil temperature
            0x5C => {
                if pid.value > 150.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Critical,
                        message: format!("Huile surchauffée: {:.0}°C", pid.value),
                        value: pid.value,
                        threshold: 150.0,
                    });
                } else if pid.value > 130.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message: format!("Huile chaude: {:.0}°C", pid.value),
                        value: pid.value,
                        threshold: 130.0,
                    });
                }
            }
            _ => {}
        }
    }

    anomalies
}
