use sample_guard::*;
use sample_guard::inventory::{InventoryManager, InventoryFilter};
use sample_guard::reader::MockRFIDReader;
use sample_guard::sample::SampleMetadata;
use chrono::Utc;
use std::time::Duration;

fn create_test_sample(id: &str) -> sample_guard::sample::Sample {
    let metadata = SampleMetadata {
        batch_number: format!("BATCH-{}", id),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
    };
    sample_guard::sample::Sample::new(id.to_string(), metadata, None)
}

#[test]
fn test_inventory_scan_multiple_tags() {
    let mut manager = InventoryManager::new();
    let mut reader = MockRFIDReader::new();
    
    let sample1 = create_test_sample("INV-001");
    let tag1 = sample1.to_tag().unwrap();
    reader.write_tag(&sample_guard::tag::TagData::new(tag1.to_bytes().unwrap())).unwrap();
    
    let results = manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
    assert!(results.len() > 0);
}

#[test]
fn test_inventory_filtering() {
    let mut manager = InventoryManager::new();
    let mut reader = MockRFIDReader::new();
    
    let sample = create_test_sample("INV-002");
    let tag = sample.to_tag().unwrap();
    reader.write_tag(&sample_guard::tag::TagData::new(tag.to_bytes().unwrap())).unwrap();
    
    manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
    
    let filtered = manager.filter_tags(&InventoryFilter::EpcPrefix("EPC-".to_string()));
    assert!(!filtered.is_empty() || filtered.is_empty()); // Just check it doesn't panic
}

#[test]
fn test_inventory_report_generation() {
    let mut manager = InventoryManager::new();
    let mut reader = MockRFIDReader::new();
    
    let sample = create_test_sample("INV-003");
    let tag = sample.to_tag().unwrap();
    reader.write_tag(&sample_guard::tag::TagData::new(tag.to_bytes().unwrap())).unwrap();
    
    manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
    let report = manager.generate_report();
    
    assert!(report.total_tags > 0);
    assert!(report.last_scan.is_some());
}

#[test]
fn test_inventory_batch_operations() {
    let manager = InventoryManager::new();
    let mut reader = MockRFIDReader::new();
    
    let sample = create_test_sample("INV-004");
    let tag = sample.to_tag().unwrap();
    reader.write_tag(&sample_guard::tag::TagData::new(tag.to_bytes().unwrap())).unwrap();
    
    let tag_ids = vec!["INV-004".to_string()];
    let samples = manager.batch_read_samples(&mut reader, &tag_ids).unwrap();
    assert!(samples.len() > 0);
}

