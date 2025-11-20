use sample_guard::audit::{AuditLogger, AuditEventType, AuditSeverity};
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
    Sample::new(id.to_string(), metadata, None)
}

#[test]
fn test_audit_logging() {
    let mut logger = AuditLogger::new();
    
    logger.log_event(
        AuditEventType::SampleCreated,
        Some("USER-001".to_string()),
        Some("SAMPLE-001".to_string()),
        serde_json::json!({}),
        AuditSeverity::Info,
    ).unwrap();
    
    assert_eq!(logger.get_all_events().len(), 1);
}

#[test]
fn test_audit_sample_operations() {
    let mut logger = AuditLogger::new();
    let sample = create_test_sample("AUDIT-001");
    
    logger.log_sample_created(&sample, Some("USER-001".to_string())).unwrap();
    logger.log_sample_read(&sample, Some("USER-001".to_string())).unwrap();
    logger.log_sample_written(&sample, Some("USER-001".to_string())).unwrap();
    
    let created = logger.get_events_by_type(&AuditEventType::SampleCreated);
    let read = logger.get_events_by_type(&AuditEventType::SampleRead);
    let written = logger.get_events_by_type(&AuditEventType::SampleWritten);
    
    assert_eq!(created.len(), 1);
    assert_eq!(read.len(), 1);
    assert_eq!(written.len(), 1);
}

#[test]
fn test_audit_status_change() {
    let mut logger = AuditLogger::new();
    
    logger.log_status_change(
        "AUDIT-002",
        SampleStatus::InProduction,
        SampleStatus::InTransit,
        Some("USER-001".to_string()),
    ).unwrap();
    
    let events = logger.get_events_by_type(&AuditEventType::StatusChanged);
    assert_eq!(events.len(), 1);
}

#[test]
fn test_audit_violation_logging() {
    let mut logger = AuditLogger::new();
    
    logger.log_integrity_violation(
        "AUDIT-003",
        vec!["Expired".to_string()],
        None,
    ).unwrap();
    
    let violations = logger.get_events_by_type(&AuditEventType::ViolationDetected);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, AuditSeverity::Error);
}

#[test]
fn test_audit_query_by_sample() {
    let mut logger = AuditLogger::new();
    let sample = create_test_sample("AUDIT-004");
    
    logger.log_sample_created(&sample, None).unwrap();
    logger.log_sample_read(&sample, None).unwrap();
    
    let events = logger.get_events_by_sample("AUDIT-004");
    assert_eq!(events.len(), 2);
}

#[test]
fn test_audit_statistics() {
    let mut logger = AuditLogger::new();
    
    logger.log_event(
        AuditEventType::SampleCreated,
        None,
        None,
        serde_json::json!({}),
        AuditSeverity::Info,
    ).unwrap();
    
    logger.log_event(
        AuditEventType::ViolationDetected,
        None,
        None,
        serde_json::json!({}),
        AuditSeverity::Error,
    ).unwrap();
    
    let stats = logger.get_statistics();
    assert_eq!(stats.total_events, 2);
}

#[test]
fn test_audit_export_json() {
    let mut logger = AuditLogger::new();
    
    logger.log_event(
        AuditEventType::SystemStartup,
        None,
        None,
        serde_json::json!({}),
        AuditSeverity::Info,
    ).unwrap();
    
    let json = logger.export_json().unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("SystemStartup"));
}

