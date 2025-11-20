use sample_guard::*;
use sample_guard::database::Database;
use sample_guard::sample::{Sample, SampleMetadata, SampleStatus};
use chrono::Utc;

fn create_test_sample(id: &str) -> Sample {
    let metadata = SampleMetadata {
        batch_number: format!("BATCH-{}", id),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
    };
    Sample::new(id.to_string(), metadata, Some("Test Location".to_string()))
}

#[test]
fn test_database_store_and_retrieve() {
    let db = Database::in_memory().unwrap();
    let sample = create_test_sample("DB-001");
    
    db.store_sample(&sample).unwrap();
    let retrieved = db.get_sample("DB-001").unwrap();
    
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().sample_id, "DB-001");
}

#[test]
fn test_database_query_by_batch() {
    let db = Database::in_memory().unwrap();
    let sample1 = create_test_sample("DB-002");
    let sample2 = create_test_sample("DB-003");
    
    db.store_sample(&sample1).unwrap();
    db.store_sample(&sample2).unwrap();
    
    let batch_samples = db.get_samples_by_batch(&sample1.metadata.batch_number).unwrap();
    assert!(batch_samples.len() > 0);
}

#[test]
fn test_database_query_by_status() {
    let db = Database::in_memory().unwrap();
    let mut sample = create_test_sample("DB-004");
    sample.update_status(SampleStatus::InTransit);
    
    db.store_sample(&sample).unwrap();
    
    let transit_samples = db.get_samples_by_status(SampleStatus::InTransit).unwrap();
    assert!(transit_samples.len() > 0);
}

#[test]
fn test_database_history_tracking() {
    let db = Database::in_memory().unwrap();
    let mut sample = create_test_sample("DB-005");
    
    db.store_sample(&sample).unwrap();
    sample.update_status(SampleStatus::InTransit);
    db.store_sample(&sample).unwrap();
    
    let history = db.get_sample_history("DB-005").unwrap();
    assert!(history.len() >= 2);
}

#[test]
fn test_database_statistics() {
    let db = Database::in_memory().unwrap();
    let sample1 = create_test_sample("DB-006");
    let sample2 = create_test_sample("DB-007");
    
    db.store_sample(&sample1).unwrap();
    db.store_sample(&sample2).unwrap();
    
    let stats = db.get_statistics().unwrap();
    assert_eq!(stats.total_samples, 2);
}

#[test]
fn test_database_delete() {
    let db = Database::in_memory().unwrap();
    let sample = create_test_sample("DB-008");
    
    db.store_sample(&sample).unwrap();
    let deleted = db.delete_sample("DB-008").unwrap();
    assert!(deleted);
    
    let retrieved = db.get_sample("DB-008").unwrap();
    assert!(retrieved.is_none());
}

