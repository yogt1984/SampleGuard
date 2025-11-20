use serde::{Deserialize, Serialize};
use crate::encryption::RFIDEncryption;
use crate::error::{SampleGuardError, Result};

/// RFID Tag memory layout specification
/// Optimized for medical device sample tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMemoryLayout {
    /// Header section (16 bytes): Tag type, version, flags
    pub header: [u8; 16],
    /// Encrypted payload section (variable, typically 64-128 bytes)
    pub payload: Vec<u8>,
    /// Integrity hash (32 bytes): SHA-256 hash of encrypted payload
    pub integrity_hash: [u8; 32],
    /// Metadata section (16 bytes): Timestamp, read count, etc.
    pub metadata: [u8; 16],
}

/// RFID Tag data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RFIDTag {
    pub tag_id: String,
    pub memory_layout: TagMemoryLayout,
    pub encryption_enabled: bool,
}

/// Raw tag data for reader/writer operations
#[derive(Debug, Clone)]
pub struct TagData {
    pub bytes: Vec<u8>,
}

impl RFIDTag {
    /// Create a new RFID tag with encrypted payload
    pub fn new(tag_id: String, payload: &[u8], encryption: &RFIDEncryption) -> Result<Self> {
        // Encrypt payload
        let encrypted_payload = encryption.encrypt(payload)?;
        
        // Calculate integrity hash
        let integrity_hash = encryption.hash(&encrypted_payload);
        
        // Create header
        let mut header = [0u8; 16];
        header[0] = 0x01; // Tag type: Sample tracking
        header[1] = 0x01; // Version
        header[2] = 0x01; // Encryption enabled flag
        
        // Create metadata (timestamp, read count)
        let mut metadata = [0u8; 16];
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        metadata[0..8].copy_from_slice(&timestamp.to_be_bytes());
        
        Ok(Self {
            tag_id,
            memory_layout: TagMemoryLayout {
                header,
                payload: encrypted_payload,
                integrity_hash,
                metadata,
            },
            encryption_enabled: true,
        })
    }

    /// Convert tag to bytes for writing to RFID hardware
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        
        // Serialize to JSON for structured storage
        let json = serde_json::to_vec(self)
            .map_err(|e| SampleGuardError::TagParseError(format!("Serialization failed: {}", e)))?;
        
        // Add length prefix
        let len = json.len() as u32;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&json);
        
        Ok(bytes)
    }

    /// Create tag from bytes read from RFID hardware
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(SampleGuardError::TagParseError(
                "Data too short for tag".to_string()
            ));
        }
        
        // Read length prefix
        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        
        if data.len() < 4 + len {
            return Err(SampleGuardError::TagParseError(
                "Incomplete tag data".to_string()
            ));
        }
        
        // Deserialize JSON
        let tag: RFIDTag = serde_json::from_slice(&data[4..4+len])
            .map_err(|e| SampleGuardError::TagParseError(format!("Deserialization failed: {}", e)))?;
        
        Ok(tag)
    }

    /// Decrypt and verify tag payload
    pub fn decrypt_payload(&self, encryption: &RFIDEncryption) -> Result<Vec<u8>> {
        // Verify integrity hash
        let calculated_hash = encryption.hash(&self.memory_layout.payload);
        if calculated_hash != self.memory_layout.integrity_hash {
            return Err(SampleGuardError::TagMemoryError(
                "Integrity hash mismatch - tag may be corrupted".to_string()
            ));
        }
        
        // Decrypt payload
        encryption.decrypt(&self.memory_layout.payload)
    }

    /// Update read count in metadata
    pub fn increment_read_count(&mut self) {
        let read_count = u64::from_be_bytes([
            self.memory_layout.metadata[8],
            self.memory_layout.metadata[9],
            self.memory_layout.metadata[10],
            self.memory_layout.metadata[11],
            self.memory_layout.metadata[12],
            self.memory_layout.metadata[13],
            self.memory_layout.metadata[14],
            self.memory_layout.metadata[15],
        ]);
        
        let new_count = read_count + 1;
        self.memory_layout.metadata[8..16].copy_from_slice(&new_count.to_be_bytes());
    }
}

impl TagData {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let encryption = RFIDEncryption::new(b"test_key_32_bytes_long_for_aes256!!");
        let payload = b"test sample data";
        
        let tag = RFIDTag::new("TAG001".to_string(), payload, &encryption).unwrap();
        assert_eq!(tag.tag_id, "TAG001");
        assert!(tag.encryption_enabled);
    }

    #[test]
    fn test_tag_serialization() {
        let encryption = RFIDEncryption::new(b"test_key_32_bytes_long_for_aes256!!");
        let payload = b"test sample data";
        
        let tag = RFIDTag::new("TAG001".to_string(), payload, &encryption).unwrap();
        let bytes = tag.to_bytes().unwrap();
        let restored = RFIDTag::from_bytes(&bytes).unwrap();
        
        assert_eq!(tag.tag_id, restored.tag_id);
        assert_eq!(tag.memory_layout.integrity_hash, restored.memory_layout.integrity_hash);
    }

    #[test]
    fn test_tag_decryption() {
        let encryption = RFIDEncryption::new(b"test_key_32_bytes_long_for_aes256!!");
        let payload = b"test sample data";
        
        let tag = RFIDTag::new("TAG001".to_string(), payload, &encryption).unwrap();
        let decrypted = tag.decrypt_payload(&encryption).unwrap();
        
        assert_eq!(payload, decrypted.as_slice());
    }
}

