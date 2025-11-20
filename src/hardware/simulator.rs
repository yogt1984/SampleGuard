use crate::tag::TagData;
use crate::error::{SampleGuardError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simulated RFID tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedTag {
    pub epc: String,
    pub tag_id: String,
    pub data: Vec<u8>,
    pub rssi: i16,
    pub antenna: u8,
    pub read_count: u64,
    pub last_read: Option<chrono::DateTime<chrono::Utc>>,
    pub error_rate: f32, // 0.0 to 1.0, probability of read error
}

impl SimulatedTag {
    pub fn new(epc: String, tag_id: String, data: Vec<u8>) -> Self {
        Self {
            epc,
            tag_id,
            data,
            rssi: -60,
            antenna: 1,
            read_count: 0,
            last_read: None,
            error_rate: 0.0,
        }
    }
    
    pub fn with_error_rate(mut self, rate: f32) -> Self {
        self.error_rate = rate.max(0.0).min(1.0);
        self
    }
    
    pub fn with_rssi(mut self, rssi: i16) -> Self {
        self.rssi = rssi;
        self
    }
    
    pub fn with_antenna(mut self, antenna: u8) -> Self {
        self.antenna = antenna;
        self
    }
    
    pub fn should_error(&self) -> bool {
        use rand::Rng;
        rand::thread_rng().gen::<f32>() < self.error_rate
    }
}

/// Tag simulator for realistic RFID behavior
pub struct TagSimulator {
    tags: HashMap<String, SimulatedTag>,
    read_delay: Duration,
    write_delay: Duration,
    network_delay: Duration,
}

impl TagSimulator {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            read_delay: Duration::from_millis(10),
            write_delay: Duration::from_millis(50),
            network_delay: Duration::from_millis(5),
        }
    }
    
    pub fn with_read_delay(mut self, delay: Duration) -> Self {
        self.read_delay = delay;
        self
    }
    
    pub fn with_write_delay(mut self, delay: Duration) -> Self {
        self.write_delay = delay;
        self
    }
    
    pub fn with_network_delay(mut self, delay: Duration) -> Self {
        self.network_delay = delay;
        self
    }
    
    /// Add a simulated tag
    pub fn add_tag(&mut self, tag: SimulatedTag) {
        self.tags.insert(tag.epc.clone(), tag);
    }
    
    /// Remove a tag
    pub fn remove_tag(&mut self, epc: &str) {
        self.tags.remove(epc);
    }
    
    /// Get all tags
    pub fn get_tags(&self) -> Vec<&SimulatedTag> {
        self.tags.values().collect()
    }
    
    /// Simulate reading a tag
    pub fn read_tag(&mut self, epc: &str) -> Result<TagData> {
        // Simulate network delay
        std::thread::sleep(self.network_delay);
        
        let tag = self.tags.get_mut(epc)
            .ok_or_else(|| SampleGuardError::ReaderError(format!("Tag {} not found", epc)))?;
        
        // Check for read error
        if tag.should_error() {
            return Err(SampleGuardError::ReaderError("Tag read error (simulated)".to_string()));
        }
        
        // Simulate read delay
        std::thread::sleep(self.read_delay);
        
        tag.read_count += 1;
        tag.last_read = Some(chrono::Utc::now());
        
        Ok(TagData::new(tag.data.clone()))
    }
    
    /// Simulate writing to a tag
    pub fn write_tag(&mut self, epc: &str, data: Vec<u8>) -> Result<()> {
        // Simulate network delay
        std::thread::sleep(self.network_delay);
        
        let tag = self.tags.get_mut(epc)
            .ok_or_else(|| SampleGuardError::ReaderError(format!("Tag {} not found", epc)))?;
        
        // Check for write error
        if tag.should_error() {
            return Err(SampleGuardError::ReaderError("Tag write error (simulated)".to_string()));
        }
        
        // Simulate write delay
        std::thread::sleep(self.write_delay);
        
        tag.data = data;
        tag.read_count += 1;
        tag.last_read = Some(chrono::Utc::now());
        
        Ok(())
    }
    
    /// Simulate scanning for tags in range
    pub fn scan_tags(&mut self, duration: Duration) -> Result<Vec<SimulatedTag>> {
        // Simulate network delay
        std::thread::sleep(self.network_delay);
        
        let start = Instant::now();
        let mut found_tags = Vec::new();
        
        while start.elapsed() < duration && found_tags.len() < self.tags.len() {
            for tag in self.tags.values() {
                // Simulate tags appearing/disappearing based on RSSI
                if tag.rssi > -80 && !found_tags.iter().any(|t: &SimulatedTag| t.epc == tag.epc) {
                    if !tag.should_error() {
                        found_tags.push(tag.clone());
                    }
                }
            }
            
            // Small delay between scan cycles
            std::thread::sleep(Duration::from_millis(10));
        }
        
        Ok(found_tags)
    }
    
    /// Get read delay
    pub fn read_delay(&self) -> Duration {
        self.read_delay
    }
    
    /// Get write delay
    pub fn write_delay(&self) -> Duration {
        self.write_delay
    }
}

impl Default for TagSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_simulator_creation() {
        let simulator = TagSimulator::new();
        assert_eq!(simulator.get_tags().len(), 0);
    }

    #[test]
    fn test_add_tag() {
        let mut simulator = TagSimulator::new();
        let tag = SimulatedTag::new("EPC-001".to_string(), "TAG-001".to_string(), vec![1, 2, 3]);
        simulator.add_tag(tag);
        assert_eq!(simulator.get_tags().len(), 1);
    }

    #[test]
    fn test_read_tag() {
        let mut simulator = TagSimulator::new();
        let tag = SimulatedTag::new("EPC-002".to_string(), "TAG-002".to_string(), vec![4, 5, 6]);
        simulator.add_tag(tag);
        
        let data = simulator.read_tag("EPC-002").unwrap();
        assert_eq!(data.as_bytes(), &[4, 5, 6]);
    }

    #[test]
    fn test_read_nonexistent_tag() {
        let mut simulator = TagSimulator::new();
        let result = simulator.read_tag("NONEXISTENT");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_tag() {
        let mut simulator = TagSimulator::new();
        let tag = SimulatedTag::new("EPC-003".to_string(), "TAG-003".to_string(), vec![1, 2, 3]);
        simulator.add_tag(tag);
        
        simulator.write_tag("EPC-003", vec![7, 8, 9]).unwrap();
        let data = simulator.read_tag("EPC-003").unwrap();
        assert_eq!(data.as_bytes(), &[7, 8, 9]);
    }

    #[test]
    fn test_tag_error_rate() {
        let tag = SimulatedTag::new("EPC-004".to_string(), "TAG-004".to_string(), vec![])
            .with_error_rate(1.0); // 100% error rate
        
        // Should always error
        for _ in 0..10 {
            assert!(tag.should_error());
        }
    }

    #[test]
    fn test_scan_tags() {
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
}

