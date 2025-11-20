use crate::hardware::protocol::{ReaderProtocol, ReaderCommand, ProtocolResponse, MemoryBank};
use crate::hardware::simulator::TagSimulator;
use crate::reader::{RFIDReader, ReaderConfig, ReaderCapabilities, ReaderFrequency};
use crate::tag::TagData;
use crate::error::{SampleGuardError, Result};
use std::time::Duration;

/// Zebra FX9600 Reader emulation
/// Implements proprietary Zebra protocol simulation
pub struct ZebraFX9600Reader {
    config: ReaderConfig,
    capabilities: ReaderCapabilities,
    simulator: TagSimulator,
    connected: bool,
    protocol_version: String,
    reader_id: String,
}

impl ZebraFX9600Reader {
    pub fn new() -> Self {
        Self {
            config: ReaderConfig {
                frequency: ReaderFrequency::UltraHighFrequency,
                power_level: 27,
                read_timeout_ms: 1500,
                antenna_gain: 6.5,
            },
            capabilities: ReaderCapabilities {
                supports_encryption: true,
                max_tag_memory: 512,
                read_range_cm: 600,
                write_speed_ms: 80,
                supported_frequencies: vec![
                    ReaderFrequency::UltraHighFrequency,
                ],
            },
            simulator: TagSimulator::new()
                .with_read_delay(Duration::from_millis(12))
                .with_write_delay(Duration::from_millis(90))
                .with_network_delay(Duration::from_millis(6)),
            connected: false,
            protocol_version: "Zebra-2.0".to_string(),
            reader_id: format!("FX9600-{:06X}", rand::random::<u32>()),
        }
    }
    
    pub fn with_simulator(mut self, simulator: TagSimulator) -> Self {
        self.simulator = simulator;
        self
    }
    
    pub fn get_reader_id(&self) -> &str {
        &self.reader_id
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

impl ReaderProtocol for ZebraFX9600Reader {
    fn send_command(&mut self, command: ReaderCommand) -> Result<ProtocolResponse> {
        let start = std::time::Instant::now();
        
        match command {
            ReaderCommand::Initialize => {
                self.connected = true;
                Ok(ProtocolResponse::success(
                    format!("Zebra FX9600 {} initialized", self.reader_id).into_bytes(),
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
                    b"Inventory session started".to_vec(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::StopInventory => {
                Ok(ProtocolResponse::success(
                    b"Inventory session stopped".to_vec(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::ReadTag { epc, bank } => {
                match self.simulator.read_tag(&epc) {
                    Ok(data) => {
                        // Zebra-specific: include memory bank info
                        let mut response_data = data.as_bytes().to_vec();
                        response_data.insert(0, match bank {
                            MemoryBank::Reserved => 0x00,
                            MemoryBank::Epc => 0x01,
                            MemoryBank::Tid => 0x02,
                            MemoryBank::User => 0x03,
                        });
                        Ok(ProtocolResponse::success(
                            response_data,
                            start.elapsed().as_millis() as u64,
                        ))
                    }
                    Err(e) => Ok(ProtocolResponse::error(
                        e.to_string(),
                        start.elapsed().as_millis() as u64,
                    )),
                }
            }
            ReaderCommand::WriteTag { epc, data, bank: _ } => {
                match self.simulator.write_tag(&epc, data) {
                    Ok(_) => Ok(ProtocolResponse::success(
                        b"Tag write completed".to_vec(),
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
                    "reader_id": self.reader_id,
                    "power_level": self.config.power_level,
                    "frequency": format!("{:?}", self.config.frequency),
                    "antenna_gain": self.config.antenna_gain,
                });
                Ok(ProtocolResponse::success(
                    serde_json::to_vec(&config_json).unwrap(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::SetConfiguration { power, antenna } => {
                self.config.power_level = power;
                // Zebra supports antenna selection
                Ok(ProtocolResponse::success(
                    format!("Configuration updated: power={}, antenna={}", power, antenna).into_bytes(),
                    start.elapsed().as_millis() as u64,
                ))
            }
            ReaderCommand::GetStatus => {
                let status_json = serde_json::json!({
                    "connected": self.connected,
                    "reader_id": self.reader_id,
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
        "Zebra"
    }
    
    fn protocol_version(&self) -> &str {
        &self.protocol_version
    }
    
    fn simulate_delay(&self) -> Duration {
        Duration::from_millis(6) // Zebra network delay
    }
}

impl RFIDReader for ZebraFX9600Reader {
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

impl Default for ZebraFX9600Reader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::simulator::SimulatedTag;

    #[test]
    fn test_zebra_creation() {
        let reader = ZebraFX9600Reader::new();
        assert_eq!(reader.protocol_name(), "Zebra");
        assert!(!reader.connected);
    }

    #[test]
    fn test_zebra_initialize() {
        let mut reader = ZebraFX9600Reader::new();
        reader.initialize().unwrap();
        assert!(reader.test_connection().unwrap());
    }

    #[test]
    fn test_zebra_protocol_commands() {
        let mut reader = ZebraFX9600Reader::new();
        reader.initialize().unwrap();
        
        let response = reader.send_command(ReaderCommand::GetStatus).unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_zebra_read_write() {
        let mut reader = ZebraFX9600Reader::new();
        
        let mut simulator = TagSimulator::new();
        let tag = SimulatedTag::new("EPC-ZEBRA-001".to_string(), "TAG-001".to_string(), vec![4, 5, 6]);
        simulator.add_tag(tag);
        *reader.get_simulator_mut() = simulator;
        
        // Initialize after setting up simulator
        reader.initialize().unwrap();
        
        let data = reader.read_tag().unwrap();
        assert_eq!(data.as_bytes(), &[4, 5, 6]);
    }

    #[test]
    fn test_zebra_reader_id() {
        let reader = ZebraFX9600Reader::new();
        assert!(reader.get_reader_id().starts_with("FX9600-"));
    }
}

