use aes::Aes256;
use cbc::{cipher::BlockEncryptMut, Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use sha2::{Digest, Sha256};
use rand::RngCore;
use crate::error::{SampleGuardError, Result};

/// Secure encryption module for RFID tag data
/// Implements AES-256-CBC encryption for medical device security compliance

pub struct RFIDEncryption {
    key: [u8; 32],
}

impl RFIDEncryption {
    /// Create a new encryption instance with a derived key
    pub fn new(master_key: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(master_key);
        let key = hasher.finalize();
        
        Self {
            key: key.into(),
        }
    }

    /// Encrypt data for RFID tag storage
    /// Uses AES-256-CBC with a random IV for each encryption
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Generate random IV
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        // Create encryptor
        let encryptor = Encryptor::<Aes256>::new_from_slices(&self.key, &iv)
            .map_err(|e| SampleGuardError::EncryptionError(format!("Encryptor creation failed: {}", e)))?;

        // Pad plaintext to block size (16 bytes) - even empty data gets padded
        let padded = self.pad_pkcs7(plaintext, 16);
        
        // Encrypt
        let mut buffer = padded;
        encryptor.encrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buffer, plaintext.len())
            .map_err(|e| SampleGuardError::EncryptionError(format!("Encryption failed: {}", e)))?;

        // Prepend IV to ciphertext
        let mut result = iv.to_vec();
        result.extend_from_slice(&buffer);
        
        Ok(result)
    }

    /// Decrypt data from RFID tag
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        // Minimum size: 16 bytes IV + 16 bytes encrypted block
        if ciphertext.len() < 32 {
            return Err(SampleGuardError::EncryptionError(
                "Ciphertext too short (need at least 32 bytes: 16 IV + 16 encrypted)".to_string()
            ));
        }

        // Extract IV (first 16 bytes)
        let iv = &ciphertext[0..16];
        let encrypted_data = &ciphertext[16..];

        // Create decryptor
        let decryptor = Decryptor::<Aes256>::new_from_slices(&self.key, iv)
            .map_err(|e| SampleGuardError::EncryptionError(format!("Decryptor creation failed: {}", e)))?;

        // Decrypt
        let mut buffer = encrypted_data.to_vec();
        let decrypted = decryptor.decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buffer)
            .map_err(|e| SampleGuardError::EncryptionError(format!("Decryption failed: {}", e)))?;

        Ok(decrypted.to_vec())
    }

    /// PKCS7 padding implementation
    fn pad_pkcs7(&self, data: &[u8], block_size: usize) -> Vec<u8> {
        let mut padded = data.to_vec();
        let pad_len = block_size - (data.len() % block_size);
        padded.extend(vec![pad_len as u8; pad_len]);
        padded
    }

    /// Generate a secure hash for integrity verification
    pub fn hash(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = b"test_master_key_32_bytes_long!!";
        let encryption = RFIDEncryption::new(key);
        
        let plaintext = b"Sample data for RFID tag";
        let ciphertext = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_hash_consistency() {
        let key = b"test_master_key_32_bytes_long!!";
        let encryption = RFIDEncryption::new(key);
        
        let data = b"test data";
        let hash1 = encryption.hash(data);
        let hash2 = encryption.hash(data);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_empty_data() {
        let key = b"test_master_key_32_bytes_long!!";
        let encryption = RFIDEncryption::new(key);
        
        let empty = b"";
        let encrypted = encryption.encrypt(empty).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        
        assert_eq!(empty, decrypted.as_slice());
    }
}

