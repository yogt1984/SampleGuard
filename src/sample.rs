use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::tag::RFIDTag;
use crate::encryption::RFIDEncryption;
use crate::error::{SampleGuardError, Result};

/// Sample status for tracking lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleStatus {
    /// Sample is in production/initial state
    InProduction,
    /// Sample is in transit
    InTransit,
    /// Sample is stored
    Stored,
    /// Sample is in use/testing
    InUse,
    /// Sample has been consumed/used
    Consumed,
    /// Sample has been discarded
    Discarded,
    /// Sample integrity is compromised
    Compromised,
}

/// Sample metadata for medical device tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    pub batch_number: String,
    pub production_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub temperature_range: Option<(f32, f32)>, // min, max in Celsius
    pub storage_conditions: String,
    pub manufacturer: String,
    pub product_line: String,
}

/// Sample entity representing a tracked medical sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub id: Uuid,
    pub sample_id: String,
    pub status: SampleStatus,
    pub metadata: SampleMetadata,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub read_count: u64,
    pub location: Option<String>,
    pub integrity_checksum: [u8; 32],
}

impl Sample {
    /// Create a new sample with metadata
    pub fn new(
        sample_id: String,
        metadata: SampleMetadata,
        location: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        // Calculate initial integrity checksum
        let integrity_checksum = Self::calculate_checksum(&sample_id, &metadata, &now);
        
        Self {
            id,
            sample_id,
            status: SampleStatus::InProduction,
            metadata,
            created_at: now,
            last_updated: now,
            read_count: 0,
            location,
            integrity_checksum,
        }
    }

    /// Convert sample to RFID tag for writing
    pub fn to_tag(&self) -> Result<RFIDTag> {
        let encryption = RFIDEncryption::new(b"default_master_key_32_bytes_long!!");
        
        // Serialize sample data
        let sample_data = serde_json::to_vec(self)
            .map_err(|e| SampleGuardError::InvalidSampleData(format!("Serialization failed: {}", e)))?;
        
        RFIDTag::new(self.sample_id.clone(), &sample_data, &encryption)
    }

    /// Create sample from RFID tag
    pub fn from_tag(tag: &RFIDTag) -> Result<Self> {
        let encryption = RFIDEncryption::new(b"default_master_key_32_bytes_long!!");
        
        // Decrypt payload
        let decrypted = tag.decrypt_payload(&encryption)?;
        
        // Deserialize sample
        let sample: Sample = serde_json::from_slice(&decrypted)
            .map_err(|e| SampleGuardError::InvalidSampleData(format!("Deserialization failed: {}", e)))?;
        
        Ok(sample)
    }

    /// Update sample status
    pub fn update_status(&mut self, new_status: SampleStatus) {
        self.status = new_status;
        self.last_updated = Utc::now();
        self.integrity_checksum = Self::calculate_checksum(
            &self.sample_id,
            &self.metadata,
            &self.last_updated,
        );
    }

    /// Update sample location
    pub fn update_location(&mut self, location: String) {
        self.location = Some(location);
        self.last_updated = Utc::now();
    }

    /// Increment read count (for tracking tag access)
    pub fn increment_read_count(&mut self) {
        self.read_count += 1;
        self.last_updated = Utc::now();
    }

    /// Verify sample integrity
    pub fn verify_integrity(&self) -> bool {
        let calculated = Self::calculate_checksum(
            &self.sample_id,
            &self.metadata,
            &self.last_updated,
        );
        calculated == self.integrity_checksum
    }

    /// Check if sample is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.metadata.expiry_date {
            Utc::now() > expiry
        } else {
            false
        }
    }

    /// Calculate integrity checksum
    fn calculate_checksum(
        sample_id: &str,
        metadata: &SampleMetadata,
        timestamp: &DateTime<Utc>,
    ) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(sample_id.as_bytes());
        hasher.update(metadata.batch_number.as_bytes());
        hasher.update(timestamp.timestamp().to_be_bytes());
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sample() -> Sample {
        let metadata = SampleMetadata {
            batch_number: "BATCH001".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test Pharma".to_string(),
            product_line: "Vaccines".to_string(),
        };
        
        Sample::new("SAMPLE001".to_string(), metadata, Some("Warehouse A".to_string()))
    }

    #[test]
    fn test_sample_creation() {
        let sample = create_test_sample();
        assert_eq!(sample.status, SampleStatus::InProduction);
        assert!(sample.verify_integrity());
    }

    #[test]
    fn test_sample_status_update() {
        let mut sample = create_test_sample();
        sample.update_status(SampleStatus::InTransit);
        assert_eq!(sample.status, SampleStatus::InTransit);
        assert!(sample.verify_integrity());
    }

    #[test]
    fn test_sample_to_tag_conversion() {
        let sample = create_test_sample();
        let tag = sample.to_tag().unwrap();
        let restored = Sample::from_tag(&tag).unwrap();
        
        assert_eq!(sample.sample_id, restored.sample_id);
        assert_eq!(sample.status, restored.status);
    }
}

