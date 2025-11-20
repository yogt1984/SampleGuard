use crate::sample::Sample;
use crate::inventory::TagScanResult;
use crate::temperature::TemperatureReading;
use crate::audit::AuditEvent;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Request to create a new sample
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSampleRequest {
    pub sample_id: String,
    pub batch_number: String,
    pub production_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub temperature_range: Option<(f32, f32)>,
    pub storage_conditions: String,
    pub manufacturer: String,
    pub product_line: String,
    pub location: Option<String>,
}

/// Request to update sample status
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSampleStatusRequest {
    pub status: String,
    pub location: Option<String>,
}

/// Response for sample operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SampleResponse {
    pub id: String,
    pub sample_id: String,
    pub status: String,
    pub batch_number: String,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub read_count: u64,
}

impl From<&Sample> for SampleResponse {
    fn from(sample: &Sample) -> Self {
        Self {
            id: sample.id.to_string(),
            sample_id: sample.sample_id.clone(),
            status: format!("{:?}", sample.status),
            batch_number: sample.metadata.batch_number.clone(),
            location: sample.location.clone(),
            created_at: sample.created_at,
            last_updated: sample.last_updated,
            read_count: sample.read_count,
        }
    }
}

/// Response for inventory scan
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryScanResponse {
    pub tags: Vec<TagScanResult>,
    pub count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Response for temperature reading
#[derive(Debug, Serialize, Deserialize)]
pub struct TemperatureResponse {
    pub reading: TemperatureReading,
    pub within_range: bool,
    pub violations: usize,
}

/// Response for audit query
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditQueryResponse {
    pub events: Vec<AuditEvent>,
    pub total: usize,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

/// Statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub samples: usize,
    pub inventory_tags: usize,
    pub temperature_readings: usize,
    pub audit_events: usize,
}

