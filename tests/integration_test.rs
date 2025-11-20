use sample_guard::*;
use sample_guard::reader::MockRFIDReader;
use sample_guard::sample::{Sample, SampleMetadata, SampleStatus};
use chrono::Utc;

#[test]
fn test_full_sample_lifecycle() {
    // Initialize system
    let reader = Box::new(MockRFIDReader::new());
    let mut guard = SampleGuard::new(reader);
    
    // Create sample
    let metadata = SampleMetadata {
        batch_number: "BATCH-TEST-001".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test Pharma".to_string(),
        product_line: "Vaccines".to_string(),
    };
    
    let mut sample = Sample::new(
        "TEST-SAMPLE-001".to_string(),
        metadata,
        Some("Test Location".to_string()),
    );
    
    // Write to tag
    guard.write_sample(&sample).unwrap();
    
    // Read from tag
    let read_sample = guard.read_sample().unwrap();
    assert_eq!(sample.sample_id, read_sample.sample_id);
    
    // Update status
    sample.update_status(SampleStatus::InTransit);
    guard.write_sample(&sample).unwrap();
    
    // Read updated sample
    let updated = guard.read_sample().unwrap();
    assert_eq!(updated.status, SampleStatus::InTransit);
    
    // Validate integrity
    let validation = guard.check_integrity(&updated).unwrap();
    assert!(validation.is_valid());
}

#[test]
fn test_integrity_validation() {
    let reader = Box::new(MockRFIDReader::new());
    let guard = SampleGuard::new(reader);
    
    // Create valid sample
    let metadata = SampleMetadata {
        batch_number: "BATCH-VALID".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(100)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
    };
    
    let sample = Sample::new("VALID-001".to_string(), metadata.clone(), None);
    let validation = guard.check_integrity(&sample).unwrap();
    assert!(validation.is_valid());
    
    // Create expired sample
    let mut expired_metadata = metadata.clone();
    expired_metadata.expiry_date = Some(Utc::now() - chrono::Duration::days(1));
    let expired_sample = Sample::new("EXPIRED-001".to_string(), expired_metadata, None);
    let expired_validation = guard.check_integrity(&expired_sample).unwrap();
    assert!(!expired_validation.is_valid());
    assert!(expired_validation.violations.contains(&integrity::Violation::Expired));
}

#[test]
fn test_encryption_roundtrip() {
    use sample_guard::encryption::RFIDEncryption;
    
    let encryption = RFIDEncryption::new(b"test_master_key_32_bytes_long_for_aes!!");
    let plaintext = b"Sample data for medical device tracking";
    
    let ciphertext = encryption.encrypt(plaintext).unwrap();
    let decrypted = encryption.decrypt(&ciphertext).unwrap();
    
    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_tag_memory_layout() {
    use sample_guard::tag::RFIDTag;
    use sample_guard::encryption::RFIDEncryption;
    
    let encryption = RFIDEncryption::new(b"test_key_32_bytes_long_for_aes256!!");
    let payload = b"Test payload data";
    
    let tag = RFIDTag::new("TAG001".to_string(), payload, &encryption).unwrap();
    
    // Verify tag structure
    assert_eq!(tag.tag_id, "TAG001");
    assert!(tag.encryption_enabled);
    assert!(!tag.memory_layout.payload.is_empty());
    assert_eq!(tag.memory_layout.integrity_hash.len(), 32);
    
    // Test serialization
    let bytes = tag.to_bytes().unwrap();
    let restored = RFIDTag::from_bytes(&bytes).unwrap();
    assert_eq!(tag.tag_id, restored.tag_id);
    
    // Test decryption
    let decrypted = restored.decrypt_payload(&encryption).unwrap();
    assert_eq!(payload, decrypted.as_slice());
}

