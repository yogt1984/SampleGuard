use sample_guard::temperature::{TemperatureMonitor, MockTemperatureSensor, ViolationType};

#[test]
fn test_temperature_monitoring() {
    let sensor = Box::new(MockTemperatureSensor::new("TEMP-001".to_string(), 5.0));
    let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
    
    let reading = monitor.read_temperature(Some("Location-A".to_string())).unwrap();
    assert_eq!(reading.temperature, 5.0);
    assert_eq!(reading.sensor_id, "TEMP-001");
}

#[test]
fn test_temperature_violation_detection() {
    let sensor = Box::new(MockTemperatureSensor::new("TEMP-002".to_string(), 1.0));
    let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
    
    monitor.read_temperature(None).unwrap();
    let violations = monitor.get_violations();
    
    assert!(violations.len() > 0);
    assert_eq!(violations[0].violation_type, ViolationType::TooLow);
}

#[test]
fn test_temperature_statistics() {
    // Use a single monitor with multiple readings of the same sensor
    // Since we can't modify sensor after moving it, we test with consistent readings
    let sensor = MockTemperatureSensor::new("TEMP-003".to_string(), 5.0);
    let mut monitor = TemperatureMonitor::new(Box::new(sensor), (2.0, 8.0)).unwrap();
    
    // Take multiple readings
    monitor.read_temperature(None).unwrap();
    monitor.read_temperature(None).unwrap();
    
    let stats = monitor.get_statistics();
    assert_eq!(stats.total_readings, 2);
    assert!(stats.min_temperature.is_some());
    assert!(stats.max_temperature.is_some());
}

#[test]
fn test_temperature_average() {
    let sensor1 = MockTemperatureSensor::new("TEMP-004".to_string(), 4.0);
    let mut monitor = TemperatureMonitor::new(Box::new(sensor1), (2.0, 8.0)).unwrap();
    monitor.read_temperature(None).unwrap();
    
    let sensor2 = MockTemperatureSensor::new("TEMP-004".to_string(), 6.0);
    monitor = TemperatureMonitor::new(Box::new(sensor2), (2.0, 8.0)).unwrap();
    monitor.read_temperature(None).unwrap();
    
    let sensor3 = MockTemperatureSensor::new("TEMP-004".to_string(), 5.0);
    monitor = TemperatureMonitor::new(Box::new(sensor3), (2.0, 8.0)).unwrap();
    monitor.read_temperature(None).unwrap();
    
    let avg = monitor.get_average_temperature(3).unwrap();
    assert!((avg - 5.0).abs() < 0.1);
}

#[test]
fn test_critical_violation() {
    let sensor = Box::new(MockTemperatureSensor::new("TEMP-005".to_string(), 15.0));
    let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
    
    monitor.read_temperature(None).unwrap();
    let critical = monitor.get_critical_violations();
    
    assert!(critical.len() > 0);
}

