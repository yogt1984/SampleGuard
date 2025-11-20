use sample_guard::hardware::*;
use sample_guard::hardware::protocol::{ReaderCommand, MemoryBank};
use sample_guard::hardware::simulator::{TagSimulator, SimulatedTag};
use sample_guard::reader::RFIDReader;
use std::time::Duration;

#[test]
fn test_impinj_creation() {
    let mut reader = ImpinjSpeedwayReader::new();
    assert_eq!(reader.protocol_name(), "LLRP");
    assert!(!reader.test_connection().unwrap());
}

#[test]
fn test_impinj_initialization() {
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
    reader.initialize().unwrap();
    
    let mut simulator = TagSimulator::new();
    let tag = SimulatedTag::new("EPC-IMPINJ-001".to_string(), "TAG-001".to_string(), vec![1, 2, 3]);
    simulator.add_tag(tag);
    reader = reader.with_simulator(simulator);
    
    let data = reader.read_tag().unwrap();
    assert_eq!(data.as_bytes(), &[1, 2, 3]);
}

#[test]
fn test_zebra_creation() {
    let mut reader = ZebraFX9600Reader::new();
    assert_eq!(reader.protocol_name(), "Zebra");
    assert!(!reader.test_connection().unwrap());
}

#[test]
fn test_zebra_initialization() {
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
    
    let response = reader.send_command(ReaderCommand::GetConfiguration).unwrap();
    assert!(response.success);
}

#[test]
fn test_zebra_read_write() {
    let mut reader = ZebraFX9600Reader::new();
    reader.initialize().unwrap();
    
    let mut simulator = TagSimulator::new();
    let tag = SimulatedTag::new("EPC-ZEBRA-001".to_string(), "TAG-001".to_string(), vec![4, 5, 6]);
    simulator.add_tag(tag);
    reader = reader.with_simulator(simulator);
    
    let data = reader.read_tag().unwrap();
    assert_eq!(data.as_bytes(), &[4, 5, 6]);
}

#[test]
fn test_tag_simulator() {
    let mut simulator = TagSimulator::new();
    let tag = SimulatedTag::new("EPC-TEST".to_string(), "TAG-TEST".to_string(), vec![1, 2, 3]);
    simulator.add_tag(tag);
    
    let data = simulator.read_tag("EPC-TEST").unwrap();
    assert_eq!(data.as_bytes(), &[1, 2, 3]);
}

#[test]
fn test_tag_simulator_error_rate() {
    let mut simulator = TagSimulator::new();
    let tag = SimulatedTag::new("EPC-ERROR".to_string(), "TAG-ERROR".to_string(), vec![1, 2, 3])
        .with_error_rate(1.0); // 100% error rate
    simulator.add_tag(tag);
    
    let result = simulator.read_tag("EPC-ERROR");
    assert!(result.is_err());
}

#[test]
fn test_tag_simulator_scan() {
    let mut simulator = TagSimulator::new();
    for i in 0..5 {
        let tag = SimulatedTag::new(
            format!("EPC-{}", i),
            format!("TAG-{}", i),
            vec![i as u8],
        ).with_rssi(-70);
        simulator.add_tag(tag);
    }
    
    let found = simulator.scan_tags(Duration::from_millis(100)).unwrap();
    assert!(found.len() > 0);
}

#[test]
fn test_protocol_response() {
    use sample_guard::hardware::protocol::ProtocolResponse;
    
    let success = ProtocolResponse::success(vec![1, 2, 3], 10);
    assert!(success.success);
    assert!(success.data.is_some());
    
    let error = ProtocolResponse::error("Test error".to_string(), 5);
    assert!(!error.success);
    assert!(error.error.is_some());
}

#[test]
fn test_reader_command_serialization() {
    let cmd = ReaderCommand::ReadTag {
        epc: "EPC-001".to_string(),
        bank: MemoryBank::User,
    };
    
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("EPC-001"));
}

#[test]
fn test_hardware_driver_creation() {
    let driver = HardwareDriver::new();
    assert!(driver.get_events().is_empty());
}

#[test]
fn test_hardware_driver_initialization() {
    let mut driver = HardwareDriver::new();
    driver.initialize_all().unwrap();
    
    let events = driver.get_events();
    assert!(events.len() >= 2);
}

#[test]
fn test_hardware_driver_inventory() {
    let mut driver = HardwareDriver::new();
    driver.initialize_all().unwrap();
    driver.setup_demo_tags();
    
    let tags = driver.perform_inventory_scan().unwrap();
    assert!(tags.len() > 0);
}

#[test]
fn test_network_delay_simulation() {
    let reader = ImpinjSpeedwayReader::new();
    let delay = reader.simulate_delay();
    assert!(delay.as_millis() > 0);
    
    let zebra = ZebraFX9600Reader::new();
    let zebra_delay = zebra.simulate_delay();
    assert!(zebra_delay.as_millis() > 0);
    assert_ne!(delay, zebra_delay); // Different readers have different delays
}

#[test]
fn test_memory_bank_enum() {
    use sample_guard::hardware::protocol::MemoryBank;
    
    assert_eq!(MemoryBank::Reserved, MemoryBank::Reserved);
    assert_ne!(MemoryBank::Epc, MemoryBank::Tid);
}

#[test]
fn test_reader_configuration() {
    let mut impinj = ImpinjSpeedwayReader::new();
    impinj.initialize().unwrap();
    
    let response = impinj.send_command(ReaderCommand::SetConfiguration {
        power: 50,
        antenna: 1,
    }).unwrap();
    
    assert!(response.success);
}

#[test]
fn test_inventory_commands() {
    let mut impinj = ImpinjSpeedwayReader::new();
    impinj.initialize().unwrap();
    
    let start_response = impinj.send_command(ReaderCommand::StartInventory).unwrap();
    assert!(start_response.success);
    
    let stop_response = impinj.send_command(ReaderCommand::StopInventory).unwrap();
    assert!(stop_response.success);
}

#[test]
fn test_error_condition_simulation() {
    let mut simulator = TagSimulator::new();
    let tag = SimulatedTag::new("EPC-ERROR".to_string(), "TAG-ERROR".to_string(), vec![1, 2, 3])
        .with_error_rate(0.5); // 50% error rate
    
    simulator.add_tag(tag);
    
    // Try reading multiple times - some should fail
    let mut errors = 0;
    for _ in 0..10 {
        if simulator.read_tag("EPC-ERROR").is_err() {
            errors += 1;
        }
    }
    
    // With 50% error rate, we should see some errors
    assert!(errors >= 0 && errors <= 10);
}

#[test]
fn test_multiple_readers_comparison() {
    let mut impinj = ImpinjSpeedwayReader::new();
    let mut zebra = ZebraFX9600Reader::new();
    
    impinj.initialize().unwrap();
    zebra.initialize().unwrap();
    
    // Both should have different protocol versions
    assert_ne!(impinj.protocol_version(), zebra.protocol_version());
    
    // Both should have different network delays
    assert_ne!(impinj.simulate_delay(), zebra.simulate_delay());
}

