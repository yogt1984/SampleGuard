use thiserror::Error;

/// Custom error types for SampleGuard system
#[derive(Error, Debug)]
pub enum SampleGuardError {
    #[error("RFID reader error: {0}")]
    ReaderError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Tag parsing error: {0}")]
    TagParseError(String),

    #[error("Sample integrity violation: {0:?}")]
    IntegrityViolation(crate::integrity::ValidationResult),

    #[error("Invalid sample data: {0}")]
    InvalidSampleData(String),

    #[error("Tag memory error: {0}")]
    TagMemoryError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type alias for SampleGuard operations
pub type Result<T> = std::result::Result<T, SampleGuardError>;

