use crate::error::{SampleGuardError, Result};
use crate::sample::{Sample, SampleStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Audit event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    SampleCreated,
    SampleRead,
    SampleWritten,
    SampleUpdated,
    SampleDeleted,
    StatusChanged,
    LocationChanged,
    IntegrityCheck,
    ViolationDetected,
    TemperatureReading,
    TemperatureViolation,
    SystemStartup,
    SystemShutdown,
    UserAction,
    ConfigurationChanged,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: uuid::Uuid,
    pub event_type: AuditEventType,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub sample_id: Option<String>,
    pub details: serde_json::Value,
    pub severity: AuditSeverity,
}

/// Audit severity level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit logger for tracking all system operations
pub struct AuditLogger {
    events: VecDeque<AuditEvent>,
    max_events: usize,
    file_writer: Option<BufWriter<File>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            max_events: 10000,
            file_writer: None,
        }
    }

    /// Create audit logger with file output
    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| SampleGuardError::IoError(e))?;

        Ok(Self {
            events: VecDeque::new(),
            max_events: 10000,
            file_writer: Some(BufWriter::new(file)),
        })
    }

    /// Log an audit event
    pub fn log_event(
        &mut self,
        event_type: AuditEventType,
        user_id: Option<String>,
        sample_id: Option<String>,
        details: serde_json::Value,
        severity: AuditSeverity,
    ) -> Result<()> {
        let event = AuditEvent {
            event_id: uuid::Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            user_id,
            sample_id,
            details,
            severity,
        };

        // Store in memory
        self.events.push_back(event.clone());
        if self.events.len() > self.max_events {
            self.events.pop_front();
        }

        // Write to file if configured
        if let Some(writer) = &mut self.file_writer {
            let json = serde_json::to_string(&event)
                .map_err(|e| SampleGuardError::SerializationError(e))?;
            writeln!(writer, "{}", json)
                .map_err(|e| SampleGuardError::IoError(e))?;
            writer.flush()
                .map_err(|e| SampleGuardError::IoError(e))?;
        }

        Ok(())
    }

    /// Log sample creation
    pub fn log_sample_created(&mut self, sample: &Sample, user_id: Option<String>) -> Result<()> {
        let details = serde_json::json!({
            "sample_id": sample.sample_id,
            "batch_number": sample.metadata.batch_number,
            "status": format!("{:?}", sample.status),
        });

        self.log_event(
            AuditEventType::SampleCreated,
            user_id,
            Some(sample.sample_id.clone()),
            details,
            AuditSeverity::Info,
        )
    }

    /// Log sample read
    pub fn log_sample_read(&mut self, sample: &Sample, user_id: Option<String>) -> Result<()> {
        let details = serde_json::json!({
            "sample_id": sample.sample_id,
            "read_count": sample.read_count,
        });

        self.log_event(
            AuditEventType::SampleRead,
            user_id,
            Some(sample.sample_id.clone()),
            details,
            AuditSeverity::Info,
        )
    }

    /// Log sample write
    pub fn log_sample_written(&mut self, sample: &Sample, user_id: Option<String>) -> Result<()> {
        let details = serde_json::json!({
            "sample_id": sample.sample_id,
        });

        self.log_event(
            AuditEventType::SampleWritten,
            user_id,
            Some(sample.sample_id.clone()),
            details,
            AuditSeverity::Info,
        )
    }

    /// Log status change
    pub fn log_status_change(
        &mut self,
        sample_id: &str,
        old_status: SampleStatus,
        new_status: SampleStatus,
        user_id: Option<String>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "old_status": format!("{:?}", old_status),
            "new_status": format!("{:?}", new_status),
        });

        self.log_event(
            AuditEventType::StatusChanged,
            user_id,
            Some(sample_id.to_string()),
            details,
            AuditSeverity::Info,
        )
    }

    /// Log integrity violation
    pub fn log_integrity_violation(
        &mut self,
        sample_id: &str,
        violations: Vec<String>,
        user_id: Option<String>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "violations": violations,
        });

        self.log_event(
            AuditEventType::ViolationDetected,
            user_id,
            Some(sample_id.to_string()),
            details,
            AuditSeverity::Error,
        )
    }

    /// Log temperature violation
    pub fn log_temperature_violation(
        &mut self,
        sample_id: Option<String>,
        temperature: f32,
        expected_range: (f32, f32),
        user_id: Option<String>,
    ) -> Result<()> {
        let details = serde_json::json!({
            "temperature": temperature,
            "expected_min": expected_range.0,
            "expected_max": expected_range.1,
        });

        self.log_event(
            AuditEventType::TemperatureViolation,
            user_id,
            sample_id,
            details,
            AuditSeverity::Warning,
        )
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<&AuditEvent> {
        self.events.iter().collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &AuditEventType) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == *event_type)
            .collect()
    }

    /// Get events by sample ID
    pub fn get_events_by_sample(&self, sample_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.sample_id.as_ref().map(|s| s == sample_id).unwrap_or(false))
            .collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: &AuditSeverity) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.severity == *severity)
            .collect()
    }

    /// Get recent events
    pub fn get_recent_events(&self, count: usize) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Query events with filters
    pub fn query_events(
        &self,
        event_type: Option<&AuditEventType>,
        sample_id: Option<&str>,
        severity: Option<&AuditSeverity>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Some(et) = event_type {
                    if e.event_type != *et {
                        return false;
                    }
                }
                if let Some(sid) = sample_id {
                    if e.sample_id.as_ref().map(|s| s != sid).unwrap_or(true) {
                        return false;
                    }
                }
                if let Some(sev) = severity {
                    if e.severity != *sev {
                        return false;
                    }
                }
                if let Some(start) = start_time {
                    if e.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = end_time {
                    if e.timestamp > end {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Get audit statistics
    pub fn get_statistics(&self) -> AuditStatistics {
        let total_events = self.events.len();
        let mut type_counts = std::collections::HashMap::new();
        let mut severity_counts = std::collections::HashMap::new();

        for event in &self.events {
            *type_counts.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;
            *severity_counts.entry(format!("{:?}", event.severity)).or_insert(0) += 1;
        }

        AuditStatistics {
            total_events,
            type_counts,
            severity_counts,
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Export events to JSON
    pub fn export_json(&self) -> Result<String> {
        let events: Vec<&AuditEvent> = self.events.iter().collect();
        serde_json::to_string(&events)
            .map_err(|e| SampleGuardError::SerializationError(e))
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub type_counts: std::collections::HashMap<String, usize>,
    pub severity_counts: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::SampleMetadata;
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
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert_eq!(logger.get_all_events().len(), 0);
    }

    #[test]
    fn test_log_event() {
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
    fn test_log_sample_created() {
        let mut logger = AuditLogger::new();
        let sample = create_test_sample("TEST-001");
        logger.log_sample_created(&sample, Some("USER-001".to_string())).unwrap();

        let events = logger.get_events_by_type(&AuditEventType::SampleCreated);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sample_id, Some("TEST-001".to_string()));
    }

    #[test]
    fn test_log_sample_read() {
        let mut logger = AuditLogger::new();
        let sample = create_test_sample("TEST-002");
        logger.log_sample_read(&sample, None).unwrap();

        let events = logger.get_events_by_type(&AuditEventType::SampleRead);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_log_sample_written() {
        let mut logger = AuditLogger::new();
        let sample = create_test_sample("TEST-003");
        logger.log_sample_written(&sample, None).unwrap();

        let events = logger.get_events_by_type(&AuditEventType::SampleWritten);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_log_status_change() {
        let mut logger = AuditLogger::new();
        logger.log_status_change(
            "TEST-004",
            SampleStatus::InProduction,
            SampleStatus::InTransit,
            Some("USER-001".to_string()),
        ).unwrap();

        let events = logger.get_events_by_type(&AuditEventType::StatusChanged);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_log_integrity_violation() {
        let mut logger = AuditLogger::new();
        logger.log_integrity_violation(
            "TEST-005",
            vec!["Expired".to_string(), "ChecksumMismatch".to_string()],
            None,
        ).unwrap();

        let events = logger.get_events_by_type(&AuditEventType::ViolationDetected);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, AuditSeverity::Error);
    }

    #[test]
    fn test_log_temperature_violation() {
        let mut logger = AuditLogger::new();
        logger.log_temperature_violation(
            Some("TEST-006".to_string()),
            15.0,
            (2.0, 8.0),
            None,
        ).unwrap();

        let events = logger.get_events_by_type(&AuditEventType::TemperatureViolation);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, AuditSeverity::Warning);
    }

    #[test]
    fn test_get_events_by_sample() {
        let mut logger = AuditLogger::new();
        let sample = create_test_sample("TEST-007");
        logger.log_sample_created(&sample, None).unwrap();
        logger.log_sample_read(&sample, None).unwrap();

        let events = logger.get_events_by_sample("TEST-007");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_get_events_by_severity() {
        let mut logger = AuditLogger::new();
        logger.log_integrity_violation("TEST-008", vec!["Error".to_string()], None).unwrap();
        let sample = create_test_sample("TEST-009");
        logger.log_sample_created(&sample, None).unwrap();

        let error_events = logger.get_events_by_severity(&AuditSeverity::Error);
        assert_eq!(error_events.len(), 1);
    }

    #[test]
    fn test_get_recent_events() {
        let mut logger = AuditLogger::new();
        for i in 0..5 {
            logger.log_event(
                AuditEventType::UserAction,
                None,
                None,
                serde_json::json!({"action": i}),
                AuditSeverity::Info,
            ).unwrap();
        }

        let recent = logger.get_recent_events(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_query_events() {
        let mut logger = AuditLogger::new();
        let sample = create_test_sample("TEST-010");
        logger.log_sample_created(&sample, None).unwrap();
        logger.log_sample_read(&sample, None).unwrap();

        let created_events = logger.query_events(
            Some(&AuditEventType::SampleCreated),
            None,
            None,
            None,
            None,
        );
        assert_eq!(created_events.len(), 1);
    }

    #[test]
    fn test_get_statistics() {
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
    fn test_clear() {
        let mut logger = AuditLogger::new();
        logger.log_event(
            AuditEventType::UserAction,
            None,
            None,
            serde_json::json!({}),
            AuditSeverity::Info,
        ).unwrap();
        logger.clear();

        assert_eq!(logger.get_all_events().len(), 0);
    }

    #[test]
    fn test_export_json() {
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
}

