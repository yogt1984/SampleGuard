use sample_guard::reader::{RFIDReader, ReaderConfig, ReaderFrequency};

/// Test suite for RFID hardware evaluation
/// This demonstrates testing capabilities for RFID labels and reader hardware

#[test]
fn test_reader_configuration() {
    let config = ReaderConfig {
        frequency: ReaderFrequency::HighFrequency,
        power_level: 75,
        read_timeout_ms: 2000,
        antenna_gain: 6.0,
    };
    
    assert_eq!(config.frequency, ReaderFrequency::HighFrequency);
    assert_eq!(config.power_level, 75);
}

#[test]
fn test_reader_capabilities() {
    use sample_guard::reader::MockRFIDReader;
    
    let mut reader = MockRFIDReader::new();
    reader.initialize().unwrap();
    
    let capabilities = reader.get_capabilities();
    assert!(capabilities.supports_encryption);
    assert!(capabilities.max_tag_memory >= 512);
    assert!(capabilities.supported_frequencies.len() > 0);
}

#[test]
fn test_reader_connection() {
    use sample_guard::reader::MockRFIDReader;
    
    let mut reader = MockRFIDReader::new();
    let connected = reader.test_connection().unwrap();
    assert!(connected);
}

#[test]
fn test_tag_read_write_cycle() {
    use sample_guard::reader::MockRFIDReader;
    use sample_guard::tag::TagData;
    
    let mut reader = MockRFIDReader::new();
    reader.initialize().unwrap();
    
    // Write test data
    let test_data = TagData::new(b"Test RFID tag data".to_vec());
    reader.write_tag(&test_data).unwrap();
    
    // Read back
    let read_data = reader.read_tag().unwrap();
    assert_eq!(test_data.as_bytes(), read_data.as_bytes());
}

/// Performance test for RFID read operations
#[test]
fn test_read_performance() {
    use sample_guard::reader::MockRFIDReader;
    use sample_guard::tag::TagData;
    use std::time::Instant;
    
    let mut reader = MockRFIDReader::new();
    reader.initialize().unwrap();
    
    let test_data = TagData::new(vec![0u8; 256]); // 256 byte tag
    reader.write_tag(&test_data).unwrap();
    
    let start = Instant::now();
    for _ in 0..100 {
        reader.read_tag().unwrap();
    }
    let duration = start.elapsed();
    
    let avg_time = duration.as_millis() / 100;
    println!("Average read time: {}ms", avg_time);
    
    // In production, this would verify against SLA requirements
    assert!(avg_time < 100); // Should be under 100ms
}

/// Test for evaluating different RFID label types
#[test]
fn test_label_type_evaluation() {
    // This test framework allows evaluation of different RFID label types
    // In production, this would test:
    // - Different IC types (NXP, Impinj, etc.)
    // - Memory capacity
    // - Read/write performance
    // - Environmental durability
    
    let label_types = vec![
        ("UHF Gen2", 512),
        ("HF ISO15693", 256),
        ("HF ISO14443", 1024),
    ];
    
    for (label_type, memory_size) in label_types {
        println!("Evaluating {} label with {} bytes memory", label_type, memory_size);
        // In production: actual hardware testing would occur here
        assert!(memory_size > 0);
    }
}

