use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Reader protocol commands
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReaderCommand {
    /// Initialize reader
    Initialize,
    /// Start inventory scan
    StartInventory,
    /// Stop inventory scan
    StopInventory,
    /// Read tag data
    ReadTag { epc: String, bank: MemoryBank },
    /// Write tag data
    WriteTag { epc: String, bank: MemoryBank, data: Vec<u8> },
    /// Get reader configuration
    GetConfiguration,
    /// Set reader configuration
    SetConfiguration { power: u8, antenna: u8 },
    /// Get reader status
    GetStatus,
}

/// Memory bank types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryBank {
    Reserved,  // Bank 00
    Epc,       // Bank 01
    Tid,       // Bank 10
    User,      // Bank 11
}

/// Protocol message for reader communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub command: ReaderCommand,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_id: u64,
}

/// Reader protocol trait
pub trait ReaderProtocol: Send + Sync {
    /// Send a command and receive response
    fn send_command(&mut self, command: ReaderCommand) -> Result<ProtocolResponse>;
    
    /// Get protocol name
    fn protocol_name(&self) -> &str;
    
    /// Get protocol version
    fn protocol_version(&self) -> &str;
    
    /// Simulate network delay
    fn simulate_delay(&self) -> Duration;
}

/// Protocol response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolResponse {
    pub success: bool,
    pub data: Option<Vec<u8>>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
}

impl ProtocolResponse {
    pub fn success(data: Vec<u8>, response_time_ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            response_time_ms,
        }
    }
    
    pub fn error(error: String, response_time_ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            response_time_ms,
        }
    }
}

