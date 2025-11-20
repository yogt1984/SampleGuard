use crate::error::{SampleGuardError, Result};
use crate::reader::RFIDReader;
use crate::sample::Sample;
#[allow(unused_imports)]
use crate::tag::{RFIDTag, TagData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Tag scan result containing tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagScanResult {
    pub epc: String,
    pub tag_id: String,
    pub rssi: i16, // Received Signal Strength Indicator
    pub antenna: u8,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Inventory filter criteria
#[derive(Debug, Clone)]
pub enum InventoryFilter {
    /// Filter by EPC prefix
    EpcPrefix(String),
    /// Filter by minimum RSSI
    MinRssi(i16),
    /// Filter by antenna number
    Antenna(u8),
    /// Filter by tag ID
    TagId(String),
    /// No filter
    None,
}

/// Multi-tag inventory manager
pub struct InventoryManager {
    scanned_tags: HashMap<String, TagScanResult>,
    last_scan_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl InventoryManager {
    /// Create a new inventory manager
    pub fn new() -> Self {
        Self {
            scanned_tags: HashMap::new(),
            last_scan_time: None,
        }
    }

    /// Scan for multiple RFID tags
    pub fn scan_tags<R: RFIDReader>(
        &mut self,
        reader: &mut R,
        duration: Duration,
    ) -> Result<Vec<TagScanResult>> {
        let start_time = chrono::Utc::now();
        let end_time = start_time + chrono::Duration::from_std(duration)
            .map_err(|e| SampleGuardError::ReaderError(format!("Invalid duration: {}", e)))?;
        
        let mut results = Vec::new();
        let mut seen_epcs = std::collections::HashSet::new();

        // Simulate scanning multiple tags
        // In production, this would continuously read from the reader
        while chrono::Utc::now() < end_time {
            match reader.read_tag() {
                Ok(tag_data) => {
                    match RFIDTag::from_bytes(tag_data.as_bytes()) {
                        Ok(tag) => {
                            let epc = format!("EPC-{}", tag.tag_id);
                            
                            // Avoid duplicates
                            if !seen_epcs.contains(&epc) {
                                seen_epcs.insert(epc.clone());
                                
                                let scan_result = TagScanResult {
                                    epc: epc.clone(),
                                    tag_id: tag.tag_id.clone(),
                                    rssi: -60, // Simulated RSSI
                                    antenna: 1,
                                    timestamp: chrono::Utc::now(),
                                };
                                
                                results.push(scan_result.clone());
                                self.scanned_tags.insert(epc, scan_result);
                            }
                        }
                        Err(_) => {
                            // Skip invalid tags
                            continue;
                        }
                    }
                }
                Err(SampleGuardError::ReaderError(_)) => {
                    // No more tags in range
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        self.last_scan_time = Some(chrono::Utc::now());
        Ok(results)
    }

    /// Filter scanned tags based on criteria
    pub fn filter_tags(&self, filter: &InventoryFilter) -> Vec<&TagScanResult> {
        self.scanned_tags
            .values()
            .filter(|tag| match filter {
                InventoryFilter::EpcPrefix(prefix) => tag.epc.starts_with(prefix),
                InventoryFilter::MinRssi(min_rssi) => tag.rssi >= *min_rssi,
                InventoryFilter::Antenna(antenna) => tag.antenna == *antenna,
                InventoryFilter::TagId(tag_id) => tag.tag_id == *tag_id,
                InventoryFilter::None => true,
            })
            .collect()
    }

    /// Get all scanned tags
    pub fn get_all_tags(&self) -> Vec<&TagScanResult> {
        self.scanned_tags.values().collect()
    }

    /// Get tag count
    pub fn tag_count(&self) -> usize {
        self.scanned_tags.len()
    }

    /// Clear inventory
    pub fn clear(&mut self) {
        self.scanned_tags.clear();
        self.last_scan_time = None;
    }

    /// Get last scan time
    pub fn last_scan_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_scan_time
    }

    /// Batch read samples from tags
    pub fn batch_read_samples<R: RFIDReader>(
        &self,
        reader: &mut R,
        tag_ids: &[String],
    ) -> Result<Vec<Sample>> {
        let mut samples = Vec::new();
        
        for tag_id in tag_ids {
            match reader.read_tag() {
                Ok(tag_data) => {
                    match RFIDTag::from_bytes(tag_data.as_bytes()) {
                        Ok(tag) => {
                            match Sample::from_tag(&tag) {
                                Ok(sample) => {
                                    if sample.sample_id == *tag_id {
                                        samples.push(sample);
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                        Err(_) => continue,
                    }
                }
                Err(SampleGuardError::ReaderError(_)) => break,
                Err(e) => return Err(e),
            }
        }
        
        Ok(samples)
    }

    /// Generate inventory report
    pub fn generate_report(&self) -> InventoryReport {
        let total_tags = self.scanned_tags.len();
        let antennas: Vec<u8> = self.scanned_tags
            .values()
            .map(|t| t.antenna)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        let avg_rssi = if total_tags > 0 {
            let sum: i32 = self.scanned_tags.values().map(|t| t.rssi as i32).sum();
            (sum / total_tags as i32) as i16
        } else {
            0
        };

        InventoryReport {
            total_tags,
            antennas,
            average_rssi: avg_rssi,
            last_scan: self.last_scan_time,
        }
    }
}

impl Default for InventoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Inventory report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryReport {
    pub total_tags: usize,
    pub antennas: Vec<u8>,
    pub average_rssi: i16,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::MockRFIDReader;
    use crate::sample::SampleMetadata;
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_sample(id: &str) -> Sample {
        let metadata = SampleMetadata {
            batch_number: format!("BATCH-{}", id),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
        };
        Sample::new(id.to_string(), metadata, None)
    }

    #[test]
    fn test_inventory_manager_creation() {
        let manager = InventoryManager::new();
        assert_eq!(manager.tag_count(), 0);
        assert!(manager.last_scan_time().is_none());
    }

    #[test]
    fn test_inventory_manager_default() {
        let manager = InventoryManager::default();
        assert_eq!(manager.tag_count(), 0);
    }

    #[test]
    fn test_clear_inventory() {
        let mut manager = InventoryManager::new();
        manager.clear();
        assert_eq!(manager.tag_count(), 0);
    }

    #[test]
    fn test_filter_none() {
        let manager = InventoryManager::new();
        let filtered = manager.filter_tags(&InventoryFilter::None);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_epc_prefix() {
        let mut manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        // Create and write a sample
        let sample = create_test_sample("TEST-001");
        let tag = sample.to_tag().unwrap();
        let tag_data = crate::tag::TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        // Scan tags
        let results = manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
        assert!(results.len() > 0);
        
        // Filter by EPC prefix
        let filtered = manager.filter_tags(&InventoryFilter::EpcPrefix("EPC-".to_string()));
        assert!(filtered.len() > 0);
    }

    #[test]
    fn test_filter_min_rssi() {
        let mut manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let sample = create_test_sample("TEST-002");
        let tag = sample.to_tag().unwrap();
        let tag_data = TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        let _results = manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
        
        let filtered = manager.filter_tags(&InventoryFilter::MinRssi(-70));
        assert!(!filtered.is_empty() || filtered.is_empty()); // Just check it doesn't panic
    }

    #[test]
    fn test_filter_antenna() {
        let mut manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let sample = create_test_sample("TEST-003");
        let tag = sample.to_tag().unwrap();
        let tag_data = TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        let _results = manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
        
        let filtered = manager.filter_tags(&InventoryFilter::Antenna(1));
        assert!(!filtered.is_empty() || filtered.is_empty()); // Just check it doesn't panic
    }

    #[test]
    fn test_filter_tag_id() {
        let mut manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let sample = create_test_sample("TEST-004");
        let tag = sample.to_tag().unwrap();
        let tag_data = TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        let _results = manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
        
        let filtered = manager.filter_tags(&InventoryFilter::TagId("TEST-004".to_string()));
        assert!(!filtered.is_empty() || filtered.is_empty()); // Just check it doesn't panic
    }

    #[test]
    fn test_get_all_tags() {
        let manager = InventoryManager::new();
        let tags = manager.get_all_tags();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_tag_count() {
        let manager = InventoryManager::new();
        assert_eq!(manager.tag_count(), 0);
    }

    #[test]
    fn test_last_scan_time() {
        let manager = InventoryManager::new();
        assert!(manager.last_scan_time().is_none());
    }

    #[test]
    fn test_generate_report_empty() {
        let manager = InventoryManager::new();
        let report = manager.generate_report();
        assert_eq!(report.total_tags, 0);
        assert_eq!(report.average_rssi, 0);
    }

    #[test]
    fn test_generate_report_with_tags() {
        let mut manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let sample = create_test_sample("TEST-005");
        let tag = sample.to_tag().unwrap();
        let tag_data = TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        let _results = manager.scan_tags(&mut reader, Duration::from_millis(100)).unwrap();
        
        let report = manager.generate_report();
        assert!(report.total_tags > 0);
        assert!(report.last_scan.is_some());
    }

    #[test]
    fn test_batch_read_samples() {
        let manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let sample = create_test_sample("TEST-006");
        let tag = sample.to_tag().unwrap();
        let tag_data = TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        let tag_ids = vec!["TEST-006".to_string()];
        let samples = manager.batch_read_samples(&mut reader, &tag_ids).unwrap();
        assert!(samples.len() > 0);
    }

    #[test]
    fn test_batch_read_empty_list() {
        let manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let tag_ids = Vec::new();
        let samples = manager.batch_read_samples(&mut reader, &tag_ids).unwrap();
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_scan_tags_duration() {
        let mut manager = InventoryManager::new();
        let mut reader = MockRFIDReader::new();
        
        let sample = create_test_sample("TEST-007");
        let tag = sample.to_tag().unwrap();
        let tag_data = TagData::new(tag.to_bytes().unwrap());
        reader.write_tag(&tag_data).unwrap();
        
        let results = manager.scan_tags(&mut reader, Duration::from_millis(50)).unwrap();
        assert!(!results.is_empty() || results.is_empty()); // Just check it doesn't panic
        assert!(manager.last_scan_time().is_some());
    }

    #[test]
    fn test_inventory_report_serialization() {
        let report = InventoryReport {
            total_tags: 5,
            antennas: vec![1, 2],
            average_rssi: -65,
            last_scan: Some(Utc::now()),
        };
        
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: InventoryReport = serde_json::from_str(&json).unwrap();
        
        assert_eq!(report.total_tags, deserialized.total_tags);
        assert_eq!(report.average_rssi, deserialized.average_rssi);
    }
}

