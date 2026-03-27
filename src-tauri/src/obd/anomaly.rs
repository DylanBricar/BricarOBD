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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coolant_temp_normal() {
        let pids = vec![PidValue {
            pid: 0x05,
            name: "Coolant Temperature".to_string(),
            value: 50.0,
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_coolant_temp_warning() {
        let pids = vec![PidValue {
            pid: 0x05,
            name: "Coolant Temperature".to_string(),
            value: 105.0,
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Warning);
        assert_eq!(anomalies[0].message_key, "anomaly.coolant.warning");
    }

    #[test]
    fn test_coolant_temp_critical() {
        let pids = vec![PidValue {
            pid: 0x05,
            name: "Coolant Temperature".to_string(),
            value: 115.0,
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Critical);
        assert_eq!(anomalies[0].message_key, "anomaly.coolant.critical");
    }

    #[test]
    fn test_battery_voltage_normal() {
        let pids = vec![PidValue {
            pid: 0x42,
            name: "Battery Voltage".to_string(),
            value: 13.5,
            unit: "V".to_string(),
            min: 0.0,
            max: 65.535,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_battery_voltage_low() {
        let pids = vec![PidValue {
            pid: 0x42,
            name: "Battery Voltage".to_string(),
            value: 10.5,
            unit: "V".to_string(),
            min: 0.0,
            max: 65.535,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Critical);
        assert_eq!(anomalies[0].message_key, "anomaly.battery.low");
    }

    #[test]
    fn test_battery_voltage_high() {
        let pids = vec![PidValue {
            pid: 0x42,
            name: "Battery Voltage".to_string(),
            value: 16.0,
            unit: "V".to_string(),
            min: 0.0,
            max: 65.535,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Warning);
        assert_eq!(anomalies[0].message_key, "anomaly.battery.high");
    }

    #[test]
    fn test_rpm_normal() {
        let pids = vec![PidValue {
            pid: 0x0C,
            name: "Engine RPM".to_string(),
            value: 3000.0,
            unit: "RPM".to_string(),
            min: 0.0,
            max: 8000.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_rpm_warning() {
        let pids = vec![PidValue {
            pid: 0x0C,
            name: "Engine RPM".to_string(),
            value: 7000.0,
            unit: "RPM".to_string(),
            min: 0.0,
            max: 8000.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Warning);
        assert_eq!(anomalies[0].message_key, "anomaly.rpm.warning");
    }

    #[test]
    fn test_rpm_critical() {
        let pids = vec![PidValue {
            pid: 0x0C,
            name: "Engine RPM".to_string(),
            value: 8000.0,
            unit: "RPM".to_string(),
            min: 0.0,
            max: 8000.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Critical);
        assert_eq!(anomalies[0].message_key, "anomaly.rpm.critical");
    }

    #[test]
    fn test_oil_temperature_normal() {
        let pids = vec![PidValue {
            pid: 0x5C,
            name: "Oil Temperature".to_string(),
            value: 90.0,
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_oil_temperature_warning() {
        let pids = vec![PidValue {
            pid: 0x5C,
            name: "Oil Temperature".to_string(),
            value: 135.0,
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Warning);
        assert_eq!(anomalies[0].message_key, "anomaly.oil.warning");
    }

    #[test]
    fn test_oil_temperature_critical() {
        let pids = vec![PidValue {
            pid: 0x5C,
            name: "Oil Temperature".to_string(),
            value: 155.0,
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].level, AnomalyLevel::Critical);
        assert_eq!(anomalies[0].message_key, "anomaly.oil.critical");
    }

    #[test]
    fn test_unknown_pid() {
        let pids = vec![PidValue {
            pid: 0xFF,
            name: "Unknown".to_string(),
            value: 50.0,
            unit: "?".to_string(),
            min: 0.0,
            max: 100.0,
            history: vec![],
            timestamp: 0,
        }];
        let anomalies = check_anomalies(&pids);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_empty_input() {
        let pids = vec![];
        let anomalies = check_anomalies(&pids);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_multiple_anomalies() {
        let pids = vec![
            PidValue {
                pid: 0x05,
                name: "Coolant Temperature".to_string(),
                value: 120.0,
                unit: "°C".to_string(),
                min: -40.0,
                max: 215.0,
                history: vec![],
                timestamp: 0,
            },
            PidValue {
                pid: 0x42,
                name: "Battery Voltage".to_string(),
                value: 16.5,
                unit: "V".to_string(),
                min: 0.0,
                max: 65.535,
                history: vec![],
                timestamp: 0,
            },
        ];
        let anomalies = check_anomalies(&pids);
        assert_eq!(anomalies.len(), 2);
    }
}
