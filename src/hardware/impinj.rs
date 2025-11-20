use crate::hardware::protocol::{ReaderProtocol, ReaderCommand, ProtocolResponse};
use crate::hardware::simulator::TagSimulator;
use crate::reader::{RFIDReader, ReaderConfig, ReaderCapabilities, ReaderFrequency};
use crate::tag::TagData;
use crate::error::{SampleGuardError, Result};
use std::time::Duration;

/// Impinj Speedway Reader emulation
/// Implements LLRP (Low Level Reader Protocol) simulation
pub struct ImpinjSpeedwayReader {
    config: ReaderConfig,
    capabilities: ReaderCapabilities,
    simulator: TagSimulator,
    connected: bool,
    protocol_version: String,
}

impl ImpinjSpeedwayReader {
    pub fn new() -> Self {
        Self {
            config: ReaderConfig {
                frequency: ReaderFrequency::UltraHighFrequency,
                power_level: 30,
                read_timeout_ms: 2000,
                antenna_gain: 6.0,
            },
            capabilities: ReaderCapabilities {
                supports_encryption: true,
                max_tag_memory: 2048,
                read_range_cm: 900,
                write_speed_ms: 100,
                supported_frequencies: vec![
                    ReaderFrequency::UltraHighFrequency,
                ],
            },
            simulator: TagSimulator::new()
                .with_read_delay(Duration::from_millis(15))
                .with_write_delay(Duration::from_millis(120))
                .with_network_delay(Duration::from_millis(8)),
            connected: false,
            protocol_version: "LLRP-1.0.1".to_string(),
        }
    }
    
    pub fn with_simulator(mut self, simulator: TagSimulator) -> Self {
        self.simulator = simulator;
        self
    }
    
    pub fn get_protocol_version(&self) -> &str {
        &self.protocol_version
    }
    
    pub fn get_simulator(&self) -> &TagSimulator {
        &self.simulator
    }
    
    pub fn get_simulator_mut(&mut self) -> &mut TagSimulator {
        &mut self.simulator
    }
}

impl ReaderProtocol for ImpinjSpeedwayReader {
    fn send_command(&mut self, command: ReaderCommand) -> Result<ProtocolResponse> {
        let start = std::time::Instant::now();
        
        match command {
            ReaderCommand::Initialize => {
                self.connected = true;
                Ok(ProtocolResponse::success(
                    b"Impinj Speedway Reader initialized".to_vec(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            _ if !self.connected => {
                return Ok(ProtocolResponse::error(
                    "Reader not connected".to_string(),
                    start.elapsed().as_millis() as u64,
                ));
            }
            ReaderCommand::StartInventory => {
                Ok(ProtocolResponse::success(
                    b"Inventory started".to_vec(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::StopInventory => {
                Ok(ProtocolResponse::success(
                    b"Inventory stopped".to_vec(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::ReadTag { epc, .. } => {
                match self.simulator.read_tag(&epc) {
                    Ok(data) => Ok(ProtocolResponse::success(
                        data.as_bytes().to_vec(),
                        start.elapsed().as_millis() as u64,
                    )),
                    Err(e) => Ok(ProtocolResponse::error(
                        e.to_string(),
                        start.elapsed().as_millis() as u64,
                    )),
                }
            }
            ReaderCommand::WriteTag { epc, data, .. } => {
                match self.simulator.write_tag(&epc, data) {
                    Ok(_) => Ok(ProtocolResponse::success(
                        b"Write successful".to_vec(),
                        start.elapsed().as_millis() as u64,
                    )),
                    Err(e) => Ok(ProtocolResponse::error(
                        e.to_string(),
                        start.elapsed().as_millis() as u64,
                    )),
                }
            }
            ReaderCommand::GetConfiguration => {
                let config_json = serde_json::json!({
                    "power_level": self.config.power_level,
                    "frequency": format!("{:?}", self.config.frequency),
                    "antenna_gain": self.config.antenna_gain,
                });
                Ok(ProtocolResponse::success(
                    serde_json::to_vec(&config_json).unwrap(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::SetConfiguration { power, antenna: _ } => {
                self.config.power_level = power;
                Ok(ProtocolResponse::success(
                    b"Configuration updated".to_vec(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::GetStatus => {
                let status_json = serde_json::json!({
                    "connected": self.connected,
                    "protocol": self.protocol_version,
                    "tags_in_range": self.simulator.get_tags().len(),
                });
                Ok(ProtocolResponse::success(
                    serde_json::to_vec(&status_json).unwrap(),
                    start.elapsed().as_millis() as u64,
                ))
            }
        }
    }
    
    fn protocol_name(&self) -> &str {
        "LLRP"
    }
    
    fn protocol_version(&self) -> &str {
        &self.protocol_version
    }
    
    fn simulate_delay(&self) -> Duration {
        Duration::from_millis(8) // Impinj network delay
    }
}

impl RFIDReader for ImpinjSpeedwayReader {
    fn initialize(&mut self) -> Result<()> {
        let response = self.send_command(ReaderCommand::Initialize)?;
        if response.success {
            self.connected = true;
            Ok(())
        } else {
            Err(SampleGuardError::ReaderError(
                response.error.unwrap_or_else(|| "Initialization failed".to_string())
            ))
        }
    }
    
    fn read_tag(&mut self) -> Result<TagData> {
        if !self.connected {
            return Err(SampleGuardError::ReaderError("Reader not connected".to_string()));
        }
        
        // Get first available tag
        let tags = self.simulator.get_tags();
        if tags.is_empty() {
            return Err(SampleGuardError::ReaderError("No tags in range".to_string()));
        }
        
        let epc = tags[0].epc.clone();
        self.simulator.read_tag(&epc)
    }
    
    fn write_tag(&mut self, data: &TagData) -> Result<()> {
        if !self.connected {
            return Err(SampleGuardError::ReaderError("Reader not connected".to_string()));
        }
        
        // Get first available tag
        let tags = self.simulator.get_tags();
        if tags.is_empty() {
            return Err(SampleGuardError::ReaderError("No tags in range".to_string()));
        }
        
        let epc = tags[0].epc.clone();
        self.simulator.write_tag(&epc, data.as_bytes().to_vec())
    }
    
    fn get_config(&self) -> &ReaderConfig {
        &self.config
    }
    
    fn get_capabilities(&self) -> &ReaderCapabilities {
        &self.capabilities
    }
    
    fn test_connection(&mut self) -> Result<bool> {
        Ok(self.connected)
    }
}

impl Default for ImpinjSpeedwayReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::simulator::SimulatedTag;

    #[test]
    fn test_impinj_creation() {
        let reader = ImpinjSpeedwayReader::new();
        assert_eq!(reader.protocol_name(), "LLRP");
        assert!(!reader.connected);
    }

    #[test]
    fn test_impinj_initialize() {
        let mut reader = ImpinjSpeedwayReader::new();
        reader.initialize().unwrap();
        assert!(reader.test_connection().unwrap());
    }

    #[test]
    fn test_impinj_protocol_commands() {
        let mut reader = ImpinjSpeedwayReader::new();
        reader.initialize().unwrap();
        
        let response = reader.send_command(ReaderCommand::GetStatus).unwrap();
        assert!(response.success);
        
        let response = reader.send_command(ReaderCommand::GetConfiguration).unwrap();
        assert!(response.success);
    }

    #[test]
    fn test_impinj_read_write() {
        let mut reader = ImpinjSpeedwayReader::new();
        
        let mut simulator = TagSimulator::new();
        let tag = SimulatedTag::new("EPC-IMPINJ-001".to_string(), "TAG-001".to_string(), vec![1, 2, 3]);
        simulator.add_tag(tag);
        *reader.get_simulator_mut() = simulator;
        
        // Initialize after setting up simulator
        reader.initialize().unwrap();
        
        let data = reader.read_tag().unwrap();
        assert_eq!(data.as_bytes(), &[1, 2, 3]);
    }
}

