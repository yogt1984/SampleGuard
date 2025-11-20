use crate::sample::{Sample, SampleStatus};
use crate::error::Result;
use chrono::Utc;

/// Validation result for sample integrity checks
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<Violation>,
    pub warnings: Vec<Warning>,
}

/// Types of integrity violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    ChecksumMismatch,
    Expired,
    StatusInvalid,
    TemperatureOutOfRange,
    ReadCountAnomaly,
    TimestampAnomaly,
}

/// Types of warnings (non-critical issues)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    HighReadCount,
    ApproachingExpiry,
    LocationChanged,
}

/// Integrity validator for medical device samples
/// Implements comprehensive validation rules for sample integrity
pub struct IntegrityValidator {
    max_read_count: u64,
    #[allow(dead_code)]
    temperature_tolerance: f32, // Reserved for future temperature validation
}

impl IntegrityValidator {
    pub fn new() -> Self {
        Self {
            max_read_count: 1000,
            temperature_tolerance: 2.0, // ±2°C tolerance
        }
    }

    /// Validate a sample's integrity
    pub fn validate(&self, sample: &Sample) -> Result<ValidationResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Check integrity checksum
        if !sample.verify_integrity() {
            violations.push(Violation::ChecksumMismatch);
        }

        // Check expiry
        if sample.is_expired() {
            violations.push(Violation::Expired);
        } else if let Some(expiry) = sample.metadata.expiry_date {
            let days_until_expiry = (expiry - Utc::now()).num_days();
            if days_until_expiry <= 30 && days_until_expiry > 0 {
                warnings.push(Warning::ApproachingExpiry);
            }
        }

        // Check status validity
        if matches!(sample.status, SampleStatus::Compromised) {
            violations.push(Violation::StatusInvalid);
        }

        // Check read count anomalies
        if sample.read_count > self.max_read_count {
            violations.push(Violation::ReadCountAnomaly);
        } else if sample.read_count > self.max_read_count / 2 {
            warnings.push(Warning::HighReadCount);
        }

        // Check timestamp anomalies
        if sample.last_updated > Utc::now() {
            violations.push(Violation::TimestampAnomaly);
        }

        // Temperature range validation (if applicable)
        if let Some((min_temp, max_temp)) = sample.metadata.temperature_range {
            // In a real system, we would check actual temperature logs
            // For now, we validate the range is reasonable
            if min_temp >= max_temp {
                violations.push(Violation::TemperatureOutOfRange);
            }
        }

        let is_valid = violations.is_empty();

        Ok(ValidationResult {
            is_valid,
            violations,
            warnings,
        })
    }

    /// Check if validation result indicates valid sample
    pub fn is_valid(&self, result: &ValidationResult) -> bool {
        result.is_valid
    }
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Default for IntegrityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::SampleMetadata;
    use chrono::Utc;

    fn create_valid_sample() -> Sample {
        let metadata = SampleMetadata {
            batch_number: "BATCH001".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test Pharma".to_string(),
            product_line: "Vaccines".to_string(),
        };
        
        Sample::new("SAMPLE001".to_string(), metadata, None)
    }

    #[test]
    fn test_valid_sample_validation() {
        let validator = IntegrityValidator::new();
        let sample = create_valid_sample();
        let result = validator.validate(&sample).unwrap();
        
        assert!(result.is_valid());
        assert!(!result.has_violations());
    }

    #[test]
    fn test_expired_sample_validation() {
        let validator = IntegrityValidator::new();
        let mut sample = create_valid_sample();
        sample.metadata.expiry_date = Some(Utc::now() - chrono::Duration::days(1));
        
        let result = validator.validate(&sample).unwrap();
        
        assert!(!result.is_valid());
        assert!(result.violations.contains(&Violation::Expired));
    }

    #[test]
    fn test_compromised_sample_validation() {
        let validator = IntegrityValidator::new();
        let mut sample = create_valid_sample();
        sample.update_status(SampleStatus::Compromised);
        
        let result = validator.validate(&sample).unwrap();
        
        assert!(!result.is_valid());
        assert!(result.violations.contains(&Violation::StatusInvalid));
    }
}

