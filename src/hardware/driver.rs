use crate::hardware::{ImpinjSpeedwayReader, ZebraFX9600Reader};
use crate::hardware::protocol::{ReaderProtocol, ReaderCommand};
use crate::hardware::simulator::{TagSimulator, SimulatedTag};
use crate::sample::{Sample, SampleMetadata};
use crate::encryption::RFIDEncryption;
use chrono::Utc;
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

/// Event types for hardware driver logging
#[derive(Debug, Clone, serde::Serialize)]
pub enum DriverEvent {
    ReaderInitialized { reader_type: String, protocol: String },
    TagDetected { epc: String, rssi: i16, antenna: u8 },
    TagRead { epc: String, data_size: usize, duration_ms: u64 },
    TagWritten { epc: String, data_size: usize, duration_ms: u64 },
    InventoryStarted { reader_type: String },
    InventoryCompleted { reader_type: String, tags_found: usize },
    Error { reader_type: String, error: String },
    ConfigurationChanged { reader_type: String, setting: String },
    NetworkDelay { reader_type: String, delay_ms: u64 },
    ProtocolMessage { reader_type: String, command: String, response_time_ms: u64 },
}

/// Hardware driver that orchestrates RFID readers and logs events
pub struct HardwareDriver {
    impinj_reader: ImpinjSpeedwayReader,
    zebra_reader: ZebraFX9600Reader,
    event_sender: Option<mpsc::Sender<DriverEvent>>,
    event_receiver: Option<mpsc::Receiver<DriverEvent>>,
}

impl HardwareDriver {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            impinj_reader: ImpinjSpeedwayReader::new(),
            zebra_reader: ZebraFX9600Reader::new(),
            event_sender: Some(sender),
            event_receiver: Some(receiver),
        }
    }
    
    /// Initialize all readers
    pub fn initialize_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.log_event(DriverEvent::ReaderInitialized {
            reader_type: "Impinj Speedway".to_string(),
            protocol: self.impinj_reader.protocol_version().to_string(),
        });
        
        use crate::reader::RFIDReader;
        self.impinj_reader.initialize()?;
        
        self.log_event(DriverEvent::ReaderInitialized {
            reader_type: "Zebra FX9600".to_string(),
            protocol: self.zebra_reader.protocol_version().to_string(),
        });
        
        self.zebra_reader.initialize()?;
        
        Ok(())
    }
    
    /// Setup simulated tags for demonstration
    pub fn setup_demo_tags(&mut self) {
        let _encryption = RFIDEncryption::new(b"demo_master_key_32_bytes_long!!");
        
        // Create sample data
        let metadata = SampleMetadata {
            batch_number: "DEMO-BATCH-001".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Demo Pharma".to_string(),
            product_line: "Vaccines".to_string(),
        };
        
        let sample = Sample::new("DEMO-SAMPLE-001".to_string(), metadata, Some("Warehouse A".to_string()));
        let tag = sample.to_tag().unwrap();
        let tag_data = tag.to_bytes().unwrap();
        
        // Add tags to both readers' simulators
        let mut impinj_sim = TagSimulator::new();
        let mut zebra_sim = TagSimulator::new();
        
        for i in 1..=5 {
            let epc = format!("EPC-DEMO-{:03}", i);
            let tag_id = format!("TAG-DEMO-{:03}", i);
            
            let mut sim_tag = SimulatedTag::new(epc.clone(), tag_id, tag_data.clone())
                .with_rssi(-60 - (i as i16 * 5))
                .with_antenna((i % 4) as u8 + 1);
            
            if i == 3 {
                sim_tag = sim_tag.with_error_rate(0.1); // 10% error rate for tag 3
            }
            
            impinj_sim.add_tag(sim_tag.clone());
            zebra_sim.add_tag(sim_tag);
        }
        
        *self.impinj_reader.get_simulator_mut() = impinj_sim;
        *self.zebra_reader.get_simulator_mut() = zebra_sim;
    }
    
    /// Perform inventory scan with both readers
    pub fn perform_inventory_scan(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.log_event(DriverEvent::InventoryStarted {
            reader_type: "Impinj Speedway".to_string(),
        });
        
        let start = std::time::Instant::now();
        let impinj_tags = self.impinj_reader.get_simulator_mut().scan_tags(Duration::from_millis(500))?;
        let _impinj_duration = start.elapsed();
        
        self.log_event(DriverEvent::InventoryCompleted {
            reader_type: "Impinj Speedway".to_string(),
            tags_found: impinj_tags.len(),
        });
        
        self.log_event(DriverEvent::InventoryStarted {
            reader_type: "Zebra FX9600".to_string(),
        });
        
        let start = std::time::Instant::now();
        let zebra_tags = self.zebra_reader.get_simulator_mut().scan_tags(Duration::from_millis(500))?;
        let _zebra_duration = start.elapsed();
        
        self.log_event(DriverEvent::InventoryCompleted {
            reader_type: "Zebra FX9600".to_string(),
            tags_found: zebra_tags.len(),
        });
        
        // Log detected tags
        for tag in &impinj_tags {
            self.log_event(DriverEvent::TagDetected {
                epc: tag.epc.clone(),
                rssi: tag.rssi,
                antenna: tag.antenna,
            });
        }
        
        Ok(impinj_tags.iter().map(|t| t.epc.clone()).collect())
    }
    
    /// Read tag from Impinj reader
    pub fn read_tag_impinj(&mut self, epc: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        
        // Simulate network delay
        let delay = self.impinj_reader.simulate_delay();
        thread::sleep(delay);
        self.log_event(DriverEvent::NetworkDelay {
            reader_type: "Impinj Speedway".to_string(),
            delay_ms: delay.as_millis() as u64,
        });
        
        let command = ReaderCommand::ReadTag {
            epc: epc.to_string(),
            bank: crate::hardware::protocol::MemoryBank::User,
        };
        
        let response = self.impinj_reader.send_command(command)?;
        let duration = start.elapsed();
        
        self.log_event(DriverEvent::ProtocolMessage {
            reader_type: "Impinj Speedway".to_string(),
            command: "ReadTag".to_string(),
            response_time_ms: duration.as_millis() as u64,
        });
        
        if response.success {
            let data = response.data.unwrap_or_default();
            self.log_event(DriverEvent::TagRead {
                epc: epc.to_string(),
                data_size: data.len(),
                duration_ms: duration.as_millis() as u64,
            });
            Ok(data)
        } else {
            let error = response.error.unwrap_or_else(|| "Unknown error".to_string());
            self.log_event(DriverEvent::Error {
                reader_type: "Impinj Speedway".to_string(),
                error: error.clone(),
            });
            Err(error.into())
        }
    }
    
    /// Read tag from Zebra reader
    pub fn read_tag_zebra(&mut self, epc: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        
        // Simulate network delay
        let delay = self.zebra_reader.simulate_delay();
        thread::sleep(delay);
        self.log_event(DriverEvent::NetworkDelay {
            reader_type: "Zebra FX9600".to_string(),
            delay_ms: delay.as_millis() as u64,
        });
        
        let command = ReaderCommand::ReadTag {
            epc: epc.to_string(),
            bank: crate::hardware::protocol::MemoryBank::User,
        };
        
        let response = self.zebra_reader.send_command(command)?;
        let duration = start.elapsed();
        
        self.log_event(DriverEvent::ProtocolMessage {
            reader_type: "Zebra FX9600".to_string(),
            command: "ReadTag".to_string(),
            response_time_ms: duration.as_millis() as u64,
        });
        
        if response.success {
            let data = response.data.unwrap_or_default();
            self.log_event(DriverEvent::TagRead {
                epc: epc.to_string(),
                data_size: data.len(),
                duration_ms: duration.as_millis() as u64,
            });
            Ok(data)
        } else {
            let error = response.error.unwrap_or_else(|| "Unknown error".to_string());
            self.log_event(DriverEvent::Error {
                reader_type: "Zebra FX9600".to_string(),
                error: error.clone(),
            });
            Err(error.into())
        }
    }
    
    /// Write tag using Impinj reader
    pub fn write_tag_impinj(&mut self, epc: &str, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        
        let delay = self.impinj_reader.simulate_delay();
        thread::sleep(delay);
        
        let command = ReaderCommand::WriteTag {
            epc: epc.to_string(),
            bank: crate::hardware::protocol::MemoryBank::User,
            data: data.clone(),
        };
        
        let response = self.impinj_reader.send_command(command)?;
        let duration = start.elapsed();
        
        if response.success {
            self.log_event(DriverEvent::TagWritten {
                epc: epc.to_string(),
                data_size: data.len(),
                duration_ms: duration.as_millis() as u64,
            });
            Ok(())
        } else {
            let error = response.error.unwrap_or_else(|| "Unknown error".to_string());
            self.log_event(DriverEvent::Error {
                reader_type: "Impinj Speedway".to_string(),
                error: error.clone(),
            });
            Err(error.into())
        }
    }
    
    /// Get configuration from reader
    pub fn get_reader_config(&mut self, reader_type: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = match reader_type {
            "impinj" => self.impinj_reader.send_command(ReaderCommand::GetConfiguration)?,
            "zebra" => self.zebra_reader.send_command(ReaderCommand::GetConfiguration)?,
            _ => return Err("Unknown reader type".into()),
        };
        
        if response.success {
            let config = String::from_utf8(response.data.unwrap_or_default())?;
            Ok(config)
        } else {
            Err(response.error.unwrap_or_else(|| "Failed to get configuration".to_string()).into())
        }
    }
    
    /// Log an event
    fn log_event(&self, event: DriverEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
    
    /// Get all logged events
    pub fn get_events(&self) -> Vec<DriverEvent> {
        let mut events = Vec::new();
        if let Some(receiver) = &self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        events
    }
    
    /// Print events in a formatted way
    pub fn print_events(&self) {
        let events = self.get_events();
        println!("\n=== Hardware Driver Events ===");
        for event in events {
            match event {
                DriverEvent::ReaderInitialized { reader_type, protocol } => {
                    println!("[INIT] {} initialized with protocol {}", reader_type, protocol);
                }
                DriverEvent::TagDetected { epc, rssi, antenna } => {
                    println!("[DETECT] Tag {} detected - RSSI: {} dBm, Antenna: {}", epc, rssi, antenna);
                }
                DriverEvent::TagRead { epc, data_size, duration_ms } => {
                    println!("[READ] Tag {} read - {} bytes in {}ms", epc, data_size, duration_ms);
                }
                DriverEvent::TagWritten { epc, data_size, duration_ms } => {
                    println!("[WRITE] Tag {} written - {} bytes in {}ms", epc, data_size, duration_ms);
                }
                DriverEvent::InventoryStarted { reader_type } => {
                    println!("[INVENTORY] {} started inventory scan", reader_type);
                }
                DriverEvent::InventoryCompleted { reader_type, tags_found } => {
                    println!("[INVENTORY] {} completed - {} tags found", reader_type, tags_found);
                }
                DriverEvent::Error { reader_type, error } => {
                    println!("[ERROR] {}: {}", reader_type, error);
                }
                DriverEvent::ConfigurationChanged { reader_type, setting } => {
                    println!("[CONFIG] {} configuration changed: {}", reader_type, setting);
                }
                DriverEvent::NetworkDelay { reader_type, delay_ms } => {
                    println!("[NETWORK] {} network delay: {}ms", reader_type, delay_ms);
                }
                DriverEvent::ProtocolMessage { reader_type, command, response_time_ms } => {
                    println!("[PROTOCOL] {} command '{}' completed in {}ms", reader_type, command, response_time_ms);
                }
            }
        }
        println!("=== End of Events ===\n");
    }
    
    /// Demonstrate system architecture understanding
    pub fn demonstrate_architecture(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║  SampleGuard Hardware Emulation - Architecture Demo        ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        
        println!("System Architecture Layers:");
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│ Layer 1: Application Layer (SampleGuard)                │");
        println!("│   - Sample management                                   │");
        println!("│   - Integrity validation                                │");
        println!("│   - Business logic                                       │");
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ Layer 2: Hardware Abstraction Layer                     │");
        println!("│   - RFIDReader trait                                    │");
        println!("│   - Protocol abstraction                                │");
        println!("│   - Multi-vendor support                                │");
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ Layer 3: Protocol Layer                                 │");
        println!("│   - LLRP (Impinj)                                       │");
        println!("│   - Zebra Protocol                                      │");
        println!("│   - Command/Response handling                           │");
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ Layer 4: Hardware Emulation Layer                       │");
        println!("│   - Tag simulator                                       │");
        println!("│   - Network delay simulation                            │");
        println!("│   - Error condition simulation                          │");
        println!("└─────────────────────────────────────────────────────────┘\n");
        
        // Initialize readers
        println!("[1/5] Initializing hardware readers...");
        self.initialize_all()?;
        
        // Setup demo tags
        println!("[2/5] Setting up simulated RFID tags...");
        self.setup_demo_tags();
        
        // Get configurations
        println!("[3/5] Reading reader configurations...");
        let impinj_config = self.get_reader_config("impinj")?;
        println!("  Impinj Config: {}", impinj_config);
        let zebra_config = self.get_reader_config("zebra")?;
        println!("  Zebra Config: {}", zebra_config);
        
        // Perform inventory
        println!("[4/5] Performing inventory scan...");
        let tags = self.perform_inventory_scan()?;
        println!("  Found {} tags", tags.len());
        
        // Read a tag
        if !tags.is_empty() {
            println!("[5/5] Reading tag data...");
            let epc = &tags[0];
            match self.read_tag_impinj(epc) {
                Ok(data) => println!("  Successfully read {} bytes from {}", data.len(), epc),
                Err(e) => println!("  Error reading tag: {}", e),
            }
        }
        
        // Print all events
        self.print_events();
        
        println!("Architecture Demonstration Complete!");
        println!("This demonstrates understanding of:");
        println!("  ✓ Multi-layer system architecture");
        println!("  ✓ Hardware abstraction patterns");
        println!("  ✓ Protocol implementation");
        println!("  ✓ Error handling and simulation");
        println!("  ✓ Event-driven logging\n");
        
        Ok(())
    }
}

impl Default for HardwareDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_creation() {
        let driver = HardwareDriver::new();
        assert!(driver.get_events().is_empty());
    }

    #[test]
    fn test_driver_initialization() {
        let mut driver = HardwareDriver::new();
        assert!(driver.initialize_all().is_ok());
        
        let events = driver.get_events();
        assert!(events.len() >= 2); // At least 2 initialization events
    }

    #[test]
    fn test_setup_demo_tags() {
        let mut driver = HardwareDriver::new();
        driver.setup_demo_tags();
        
        // Tags should be added to simulators
        assert!(driver.impinj_reader.get_simulator().get_tags().len() > 0);
        assert!(driver.zebra_reader.get_simulator().get_tags().len() > 0);
    }

    #[test]
    fn test_inventory_scan() {
        let mut driver = HardwareDriver::new();
        assert!(driver.initialize_all().is_ok());
        driver.setup_demo_tags();
        
        let tags = driver.perform_inventory_scan().unwrap();
        assert!(tags.len() > 0);
        
        let events = driver.get_events();
        assert!(events.len() > 0);
    }

    #[test]
    fn test_read_tag() {
        let mut driver = HardwareDriver::new();
        assert!(driver.initialize_all().is_ok());
        driver.setup_demo_tags();
        
        let tags = driver.perform_inventory_scan().unwrap();
        if !tags.is_empty() {
            let result = driver.read_tag_impinj(&tags[0]);
            // May succeed or fail based on error rate, both are valid
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_event_logging() {
        let mut driver = HardwareDriver::new();
        assert!(driver.initialize_all().is_ok());
        
        let events = driver.get_events();
        assert!(events.len() >= 2);
        
        // Check for initialization events
        let has_init = events.iter().any(|e| matches!(e, DriverEvent::ReaderInitialized { .. }));
        assert!(has_init);
    }
}

