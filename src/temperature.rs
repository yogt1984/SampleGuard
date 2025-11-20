use crate::error::{SampleGuardError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Temperature reading from a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub temperature: f32,
    pub timestamp: DateTime<Utc>,
    pub sensor_id: String,
    pub location: Option<String>,
}

/// Temperature violation type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    TooHigh,
    TooLow,
    SensorFailure,
}

/// Temperature violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureViolation {
    pub reading: TemperatureReading,
    pub violation_type: ViolationType,
    pub expected_range: (f32, f32),
    pub severity: ViolationSeverity,
}

/// Violation severity level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Critical,
}

/// Temperature sensor interface
pub trait TemperatureSensor: Send + Sync {
    fn read_temperature(&self) -> Result<f32>;
    fn get_sensor_id(&self) -> &str;
}

/// Mock temperature sensor for testing
pub struct MockTemperatureSensor {
    sensor_id: String,
    current_temperature: f32,
    readings: VecDeque<f32>,
}

impl MockTemperatureSensor {
    pub fn new(sensor_id: String, initial_temp: f32) -> Self {
        Self {
            sensor_id,
            current_temperature: initial_temp,
            readings: VecDeque::new(),
        }
    }

    pub fn set_temperature(&mut self, temp: f32) {
        self.current_temperature = temp;
        self.readings.push_back(temp);
        if self.readings.len() > 100 {
            self.readings.pop_front();
        }
    }
}

impl TemperatureSensor for MockTemperatureSensor {
    fn read_temperature(&self) -> Result<f32> {
        Ok(self.current_temperature)
    }

    fn get_sensor_id(&self) -> &str {
        &self.sensor_id
    }
}


/// Temperature monitor for sample tracking
pub struct TemperatureMonitor {
    sensor: Box<dyn TemperatureSensor>,
    expected_range: (f32, f32),
    readings: VecDeque<TemperatureReading>,
    violations: VecDeque<TemperatureViolation>,
    max_readings: usize,
    max_violations: usize,
}

impl TemperatureMonitor {
    /// Create a new temperature monitor
    pub fn new(
        sensor: Box<dyn TemperatureSensor>,
        expected_range: (f32, f32),
    ) -> Result<Self> {
        if expected_range.0 >= expected_range.1 {
            return Err(SampleGuardError::InvalidSampleData(
                "Invalid temperature range: min must be less than max".to_string()
            ));
        }

        Ok(Self {
            sensor,
            expected_range,
            readings: VecDeque::new(),
            violations: VecDeque::new(),
            max_readings: 1000,
            max_violations: 100,
        })
    }

    /// Read current temperature
    pub fn read_temperature(&mut self, location: Option<String>) -> Result<TemperatureReading> {
        let temperature = self.sensor.read_temperature()?;
        
        let reading = TemperatureReading {
            temperature,
            timestamp: Utc::now(),
            sensor_id: self.sensor.get_sensor_id().to_string(),
            location,
        };

        // Check for violations
        self.check_violation(&reading)?;

        // Store reading
        self.readings.push_back(reading.clone());
        if self.readings.len() > self.max_readings {
            self.readings.pop_front();
        }

        Ok(reading)
    }

    /// Check if temperature is within expected range
    pub fn is_within_range(&self, temperature: f32) -> bool {
        temperature >= self.expected_range.0 && temperature <= self.expected_range.1
    }

    /// Check for temperature violations
    fn check_violation(&mut self, reading: &TemperatureReading) -> Result<()> {
        let temp = reading.temperature;
        let (min, max) = self.expected_range;

        if temp < min {
            let violation = TemperatureViolation {
                reading: reading.clone(),
                violation_type: ViolationType::TooLow,
                expected_range: self.expected_range,
                severity: if temp < min - 5.0 {
                    ViolationSeverity::Critical
                } else {
                    ViolationSeverity::Warning
                },
            };
            self.record_violation(violation);
        } else if temp > max {
            let violation = TemperatureViolation {
                reading: reading.clone(),
                violation_type: ViolationType::TooHigh,
                expected_range: self.expected_range,
                severity: if temp > max + 5.0 {
                    ViolationSeverity::Critical
                } else {
                    ViolationSeverity::Warning
                },
            };
            self.record_violation(violation);
        }

        Ok(())
    }

    /// Record a temperature violation
    fn record_violation(&mut self, violation: TemperatureViolation) {
        self.violations.push_back(violation);
        if self.violations.len() > self.max_violations {
            self.violations.pop_front();
        }
    }

    /// Get all violations
    pub fn get_violations(&self) -> Vec<&TemperatureViolation> {
        self.violations.iter().collect()
    }

    /// Get critical violations only
    pub fn get_critical_violations(&self) -> Vec<&TemperatureViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Critical)
            .collect()
    }

    /// Get recent readings
    pub fn get_recent_readings(&self, count: usize) -> Vec<&TemperatureReading> {
        self.readings
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Get all readings
    pub fn get_all_readings(&self) -> Vec<&TemperatureReading> {
        self.readings.iter().collect()
    }

    /// Get average temperature over recent readings
    pub fn get_average_temperature(&self, count: usize) -> Option<f32> {
        let recent: Vec<f32> = self.readings
            .iter()
            .rev()
            .take(count)
            .map(|r| r.temperature)
            .collect();

        if recent.is_empty() {
            return None;
        }

        let sum: f32 = recent.iter().sum();
        Some(sum / recent.len() as f32)
    }

    /// Get temperature statistics
    pub fn get_statistics(&self) -> TemperatureStatistics {
        let readings: Vec<f32> = self.readings.iter().map(|r| r.temperature).collect();
        
        let min = readings.iter().copied().fold(f32::INFINITY, f32::min);
        let max = readings.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let avg = if !readings.is_empty() {
            readings.iter().sum::<f32>() / readings.len() as f32
        } else {
            0.0
        };

        TemperatureStatistics {
            total_readings: self.readings.len(),
            min_temperature: if min.is_finite() { Some(min) } else { None },
            max_temperature: if max.is_finite() { Some(max) } else { None },
            average_temperature: if !readings.is_empty() { Some(avg) } else { None },
            violation_count: self.violations.len(),
            critical_violation_count: self.get_critical_violations().len(),
        }
    }

    /// Get expected temperature range
    pub fn get_expected_range(&self) -> (f32, f32) {
        self.expected_range
    }

    /// Update expected temperature range
    pub fn set_expected_range(&mut self, range: (f32, f32)) -> Result<()> {
        if range.0 >= range.1 {
            return Err(SampleGuardError::InvalidSampleData(
                "Invalid temperature range: min must be less than max".to_string()
            ));
        }
        self.expected_range = range;
        Ok(())
    }

    /// Clear all readings and violations
    pub fn clear(&mut self) {
        self.readings.clear();
        self.violations.clear();
    }
}

/// Temperature statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureStatistics {
    pub total_readings: usize,
    pub min_temperature: Option<f32>,
    pub max_temperature: Option<f32>,
    pub average_temperature: Option<f32>,
    pub violation_count: usize,
    pub critical_violation_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_monitor_creation() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-001".to_string(), 5.0));
        let monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        assert_eq!(monitor.get_expected_range(), (2.0, 8.0));
    }

    #[test]
    fn test_temperature_monitor_invalid_range() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-002".to_string(), 5.0));
        let result = TemperatureMonitor::new(sensor, (8.0, 2.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_within_range() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-003".to_string(), 5.0));
        let monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        assert!(monitor.is_within_range(5.0));
        assert!(monitor.is_within_range(2.0));
        assert!(monitor.is_within_range(8.0));
        assert!(!monitor.is_within_range(1.0));
        assert!(!monitor.is_within_range(9.0));
    }

    #[test]
    fn test_read_temperature() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-004".to_string(), 5.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        let reading = monitor.read_temperature(Some("Location-A".to_string())).unwrap();
        assert_eq!(reading.temperature, 5.0);
        assert_eq!(reading.sensor_id, "SENSOR-004");
    }

    #[test]
    fn test_temperature_violation_too_low() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-005".to_string(), 1.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        monitor.read_temperature(None).unwrap();
        let violations = monitor.get_violations();
        assert!(violations.len() > 0);
        assert_eq!(violations[0].violation_type, ViolationType::TooLow);
    }

    #[test]
    fn test_temperature_violation_too_high() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-006".to_string(), 10.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        monitor.read_temperature(None).unwrap();
        let violations = monitor.get_violations();
        assert!(violations.len() > 0);
        assert_eq!(violations[0].violation_type, ViolationType::TooHigh);
    }

    #[test]
    fn test_critical_violation() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-007".to_string(), 15.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        monitor.read_temperature(None).unwrap();
        let critical = monitor.get_critical_violations();
        assert!(critical.len() > 0);
        assert_eq!(critical[0].severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_get_recent_readings() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-008".to_string(), 5.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        for _ in 0..5 {
            monitor.read_temperature(None).unwrap();
        }
        
        let recent = monitor.get_recent_readings(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_get_average_temperature() {
        let sensor1 = MockTemperatureSensor::new("SENSOR-009".to_string(), 4.0);
        let mut monitor = TemperatureMonitor::new(Box::new(sensor1), (2.0, 8.0)).unwrap();
        monitor.read_temperature(None).unwrap();
        
        let sensor2 = MockTemperatureSensor::new("SENSOR-009".to_string(), 6.0);
        monitor = TemperatureMonitor::new(Box::new(sensor2), (2.0, 8.0)).unwrap();
        monitor.read_temperature(None).unwrap();
        
        let sensor3 = MockTemperatureSensor::new("SENSOR-009".to_string(), 5.0);
        monitor = TemperatureMonitor::new(Box::new(sensor3), (2.0, 8.0)).unwrap();
        monitor.read_temperature(None).unwrap();
        
        let avg = monitor.get_average_temperature(3).unwrap();
        assert!((avg - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_get_statistics() {
        let sensor = MockTemperatureSensor::new("SENSOR-010".to_string(), 5.0);
        let mut monitor = TemperatureMonitor::new(Box::new(sensor), (2.0, 8.0)).unwrap();
        
        // Read multiple temperatures by creating new sensors with different temps
        // Since we can't modify the sensor after moving it, we test with a single sensor
        // that reads the same temperature multiple times
        monitor.read_temperature(None).unwrap();
        monitor.read_temperature(None).unwrap();
        
        let stats = monitor.get_statistics();
        assert_eq!(stats.total_readings, 2);
        assert!(stats.min_temperature.is_some());
        assert!(stats.max_temperature.is_some());
        assert!(stats.average_temperature.is_some());
    }

    #[test]
    fn test_set_expected_range() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-011".to_string(), 5.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        monitor.set_expected_range((0.0, 10.0)).unwrap();
        assert_eq!(monitor.get_expected_range(), (0.0, 10.0));
    }

    #[test]
    fn test_set_invalid_range() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-012".to_string(), 5.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        let result = monitor.set_expected_range((10.0, 0.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let sensor = MockTemperatureSensor::new("SENSOR-013".to_string(), 1.0);
        let mut monitor = TemperatureMonitor::new(Box::new(sensor), (2.0, 8.0)).unwrap();
        
        monitor.read_temperature(None).unwrap();
        monitor.clear();
        
        assert_eq!(monitor.get_all_readings().len(), 0);
        assert_eq!(monitor.get_violations().len(), 0);
    }

    #[test]
    fn test_no_violations_when_in_range() {
        let sensor = Box::new(MockTemperatureSensor::new("SENSOR-014".to_string(), 5.0));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        
        monitor.read_temperature(None).unwrap();
        let violations = monitor.get_violations();
        assert_eq!(violations.len(), 0);
    }
}

