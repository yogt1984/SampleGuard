pub mod encryption;
pub mod reader;
pub mod sample;
pub mod tag;
pub mod error;
pub mod integrity;
pub mod inventory;
pub mod database;
pub mod temperature;
pub mod audit;
pub mod api;
pub mod hardware;

pub use error::{SampleGuardError, Result};
pub use sample::{Sample, SampleStatus, SampleMetadata};
pub use tag::{RFIDTag, TagData, TagMemoryLayout};
pub use reader::{RFIDReader, ReaderConfig, ReaderCapabilities};
pub use integrity::{IntegrityValidator, ValidationResult};
pub use inventory::{InventoryManager, InventoryFilter, TagScanResult, InventoryReport};
pub use database::{Database, HistoryEntry, DatabaseStatistics};
pub use temperature::{TemperatureMonitor, TemperatureSensor, TemperatureReading, TemperatureViolation, TemperatureStatistics};
pub use audit::{AuditLogger, AuditEventType, AuditEvent, AuditSeverity, AuditStatistics};
pub use hardware::{ImpinjSpeedwayReader, ZebraFX9600Reader, TagSimulator, SimulatedTag, HardwareDriver};
pub use hardware::protocol::{ReaderProtocol, ReaderCommand, ProtocolResponse, MemoryBank};

/// Main entry point for SampleGuard RFID system
pub struct SampleGuard {
    reader: Box<dyn RFIDReader>,
    validator: IntegrityValidator,
}

impl SampleGuard {
    /// Create a new SampleGuard instance with a configured RFID reader
    pub fn new(reader: Box<dyn RFIDReader>) -> Self {
        Self {
            reader,
            validator: IntegrityValidator::new(),
        }
    }

    /// Read and validate a sample from an RFID tag
    pub fn read_sample(&mut self) -> Result<Sample> {
        let tag_data = self.reader.read_tag()?;
        let tag = RFIDTag::from_bytes(tag_data.as_bytes())?;
        let sample = Sample::from_tag(&tag)?;
        
        // Validate integrity
        let validation = self.validator.validate(&sample)?;
        if !validation.is_valid() {
            return Err(SampleGuardError::IntegrityViolation(validation));
        }
        
        Ok(sample)
    }

    /// Write a sample to an RFID tag
    pub fn write_sample(&mut self, sample: &Sample) -> Result<()> {
        let tag = sample.to_tag()?;
        let tag_bytes = tag.to_bytes()?;
        let tag_data = TagData::new(tag_bytes);
        self.reader.write_tag(&tag_data)?;
        Ok(())
    }

    /// Perform integrity check on a sample
    pub fn check_integrity(&self, sample: &Sample) -> Result<ValidationResult> {
        self.validator.validate(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::MockRFIDReader;

    #[test]
    fn test_sample_guard_creation() {
        let reader = Box::new(MockRFIDReader::new());
        let _guard = SampleGuard::new(reader);
    }
}

