use crate::models::PidValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anomaly {
    pub pid_name: String,
    pub level: AnomalyLevel,
    pub message_key: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnomalyLevel {
    Warning,
    Critical,
}

/// Check PID values for anomalies — returns i18n keys for messages
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
                        message_key: "anomaly.coolant.critical".to_string(),
                        value: pid.value,
                        threshold: 110.0,
                    });
                } else if pid.value > 100.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message_key: "anomaly.coolant.warning".to_string(),
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
                        message_key: "anomaly.battery.low".to_string(),
                        value: pid.value,
                        threshold: 11.5,
                    });
                } else if pid.value > 15.5 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message_key: "anomaly.battery.high".to_string(),
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
                        message_key: "anomaly.rpm.critical".to_string(),
                        value: pid.value,
                        threshold: 7500.0,
                    });
                } else if pid.value > 6500.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message_key: "anomaly.rpm.warning".to_string(),
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
                        message_key: "anomaly.oil.critical".to_string(),
                        value: pid.value,
                        threshold: 150.0,
                    });
                } else if pid.value > 130.0 {
                    anomalies.push(Anomaly {
                        pid_name: pid.name.clone(),
                        level: AnomalyLevel::Warning,
                        message_key: "anomaly.oil.warning".to_string(),
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
