use crate::error::{SampleGuardError, Result};
use crate::sample::{Sample, SampleMetadata, SampleStatus};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Database manager for SampleGuard
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open a database at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database connection failed: {}", e)
            )))?;
        
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("In-memory database failed: {}", e)
            )))?;
        
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS samples (
                id TEXT PRIMARY KEY,
                sample_id TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL,
                batch_number TEXT NOT NULL,
                production_date TEXT NOT NULL,
                expiry_date TEXT,
                temperature_min REAL,
                temperature_max REAL,
                storage_conditions TEXT NOT NULL,
                manufacturer TEXT NOT NULL,
                product_line TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_updated TEXT NOT NULL,
                read_count INTEGER NOT NULL,
                location TEXT,
                integrity_checksum TEXT NOT NULL
            )",
            [],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Schema initialization failed: {}", e)
        )))?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sample_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sample_id TEXT NOT NULL,
                status TEXT NOT NULL,
                location TEXT,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (sample_id) REFERENCES samples(sample_id)
            )",
            [],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("History table creation failed: {}", e)
        )))?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sample_id ON samples(sample_id)",
            [],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Index creation failed: {}", e)
        )))?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_batch_number ON samples(batch_number)",
            [],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Index creation failed: {}", e)
        )))?;

        Ok(())
    }

    /// Store a sample in the database
    pub fn store_sample(&self, sample: &Sample) -> Result<()> {
        let checksum_hex = hex::encode(sample.integrity_checksum);
        
        self.conn.execute(
            "INSERT OR REPLACE INTO samples (
                id, sample_id, status, batch_number, production_date, expiry_date,
                temperature_min, temperature_max, storage_conditions, manufacturer,
                product_line, created_at, last_updated, read_count, location, integrity_checksum
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                sample.id.to_string(),
                sample.sample_id,
                format!("{:?}", sample.status),
                sample.metadata.batch_number,
                sample.metadata.production_date.to_rfc3339(),
                sample.metadata.expiry_date.map(|d| d.to_rfc3339()),
                sample.metadata.temperature_range.map(|r| r.0),
                sample.metadata.temperature_range.map(|r| r.1),
                sample.metadata.storage_conditions,
                sample.metadata.manufacturer,
                sample.metadata.product_line,
                sample.created_at.to_rfc3339(),
                sample.last_updated.to_rfc3339(),
                sample.read_count,
                sample.location,
                checksum_hex,
            ],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to store sample: {}", e)
        )))?;

        // Store history entry
        self.add_history_entry(&sample.sample_id, &sample.status, sample.location.as_deref())?;

        Ok(())
    }

    /// Retrieve a sample by ID
    pub fn get_sample(&self, sample_id: &str) -> Result<Option<Sample>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sample_id, status, batch_number, production_date, expiry_date,
             temperature_min, temperature_max, storage_conditions, manufacturer,
             product_line, created_at, last_updated, read_count, location, integrity_checksum
             FROM samples WHERE sample_id = ?1"
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to prepare query: {}", e)
        )))?;

        let mut rows = stmt.query_map(params![sample_id], |row| {
            Self::row_to_sample(row)
        }).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to execute query: {}", e)
        )))?;

        match rows.next() {
            Some(Ok(sample)) => Ok(Some(sample)),
            Some(Err(e)) => Err(SampleGuardError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to parse row: {}", e)
            ))),
            None => Ok(None),
        }
    }

    /// Get all samples
    pub fn get_all_samples(&self) -> Result<Vec<Sample>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sample_id, status, batch_number, production_date, expiry_date,
             temperature_min, temperature_max, storage_conditions, manufacturer,
             product_line, created_at, last_updated, read_count, location, integrity_checksum
             FROM samples ORDER BY created_at DESC"
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to prepare query: {}", e)
        )))?;

        let samples = stmt.query_map([], |row| {
            Self::row_to_sample(row)
        }).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to execute query: {}", e)
        )))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse rows: {}", e)
        )))?;

        Ok(samples)
    }

    /// Get samples by batch number
    pub fn get_samples_by_batch(&self, batch_number: &str) -> Result<Vec<Sample>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sample_id, status, batch_number, production_date, expiry_date,
             temperature_min, temperature_max, storage_conditions, manufacturer,
             product_line, created_at, last_updated, read_count, location, integrity_checksum
             FROM samples WHERE batch_number = ?1 ORDER BY created_at DESC"
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to prepare query: {}", e)
        )))?;

        let samples = stmt.query_map(params![batch_number], |row| {
            Self::row_to_sample(row)
        }).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to execute query: {}", e)
        )))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse rows: {}", e)
        )))?;

        Ok(samples)
    }

    /// Get samples by status
    pub fn get_samples_by_status(&self, status: SampleStatus) -> Result<Vec<Sample>> {
        let status_str = format!("{:?}", status);
        let mut stmt = self.conn.prepare(
            "SELECT id, sample_id, status, batch_number, production_date, expiry_date,
             temperature_min, temperature_max, storage_conditions, manufacturer,
             product_line, created_at, last_updated, read_count, location, integrity_checksum
             FROM samples WHERE status = ?1 ORDER BY created_at DESC"
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to prepare query: {}", e)
        )))?;

        let samples = stmt.query_map(params![status_str], |row| {
            Self::row_to_sample(row)
        }).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to execute query: {}", e)
        )))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse rows: {}", e)
        )))?;

        Ok(samples)
    }

    /// Delete a sample
    pub fn delete_sample(&self, sample_id: &str) -> Result<bool> {
        // Delete history entries first (due to foreign key constraint)
        self.conn.execute(
            "DELETE FROM sample_history WHERE sample_id = ?1",
            params![sample_id],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to delete history: {}", e)
        )))?;

        let rows_affected = self.conn.execute(
            "DELETE FROM samples WHERE sample_id = ?1",
            params![sample_id],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to delete sample: {}", e)
        )))?;

        Ok(rows_affected > 0)
    }

    /// Add history entry
    pub fn add_history_entry(
        &self,
        sample_id: &str,
        status: &SampleStatus,
        location: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sample_history (sample_id, status, location, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![
                sample_id,
                format!("{:?}", status),
                location,
                Utc::now().to_rfc3339(),
            ],
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to add history entry: {}", e)
        )))?;

        Ok(())
    }

    /// Get sample history
    pub fn get_sample_history(&self, sample_id: &str) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT sample_id, status, location, timestamp FROM sample_history 
             WHERE sample_id = ?1 ORDER BY timestamp DESC"
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to prepare query: {}", e)
        )))?;

        let entries = stmt.query_map(params![sample_id], |row| {
            let status_str: String = row.get(1)?;
            let status = match status_str.as_str() {
                "InProduction" => SampleStatus::InProduction,
                "InTransit" => SampleStatus::InTransit,
                "Stored" => SampleStatus::Stored,
                "InUse" => SampleStatus::InUse,
                "Consumed" => SampleStatus::Consumed,
                "Discarded" => SampleStatus::Discarded,
                "Compromised" => SampleStatus::Compromised,
                _ => SampleStatus::InProduction,
            };
            Ok(HistoryEntry {
                sample_id: row.get(0)?,
                status,
                location: row.get(2)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        }).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to execute query: {}", e)
        )))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse rows: {}", e)
        )))?;

        Ok(entries)
    }

    /// Convert database row to Sample
    fn row_to_sample(row: &Row) -> rusqlite::Result<Sample> {
        let id_str: String = row.get(0)?;
        let id = uuid::Uuid::parse_str(&id_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, id_str, rusqlite::types::Type::Text))?;
        
        let sample_id: String = row.get(1)?;
        let status_str: String = row.get(2)?;
        let status = match status_str.as_str() {
            "InProduction" => SampleStatus::InProduction,
            "InTransit" => SampleStatus::InTransit,
            "Stored" => SampleStatus::Stored,
            "InUse" => SampleStatus::InUse,
            "Consumed" => SampleStatus::Consumed,
            "Discarded" => SampleStatus::Discarded,
            "Compromised" => SampleStatus::Compromised,
            _ => SampleStatus::InProduction,
        };
        
        let batch_number: String = row.get(3)?;
        let production_date_str: String = row.get(4)?;
        let production_date = DateTime::parse_from_rfc3339(&production_date_str)
            .unwrap()
            .with_timezone(&Utc);
        
        let expiry_date: Option<String> = row.get(5)?;
        let expiry_date_parsed = expiry_date.and_then(|s| {
            DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
        });
        
        let temp_min: Option<f32> = row.get(6)?;
        let temp_max: Option<f32> = row.get(7)?;
        let temperature_range = temp_min.zip(temp_max).map(|(min, max)| (min, max));
        
        let storage_conditions: String = row.get(8)?;
        let manufacturer: String = row.get(9)?;
        let product_line: String = row.get(10)?;
        
        let created_at_str: String = row.get(11)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .unwrap()
            .with_timezone(&Utc);
        
        let last_updated_str: String = row.get(12)?;
        let last_updated = DateTime::parse_from_rfc3339(&last_updated_str)
            .unwrap()
            .with_timezone(&Utc);
        
        let read_count: u64 = row.get(13)?;
        let location: Option<String> = row.get(14)?;
        
        let checksum_hex: String = row.get(15)?;
        let checksum_bytes = hex::decode(&checksum_hex)
            .map_err(|_| rusqlite::Error::InvalidColumnType(15, checksum_hex, rusqlite::types::Type::Text))?;
        let mut checksum = [0u8; 32];
        checksum.copy_from_slice(&checksum_bytes[..32]);

        let metadata = SampleMetadata {
            batch_number,
            production_date,
            expiry_date: expiry_date_parsed,
            temperature_range,
            storage_conditions,
            manufacturer,
            product_line,
        };

        let sample = Sample {
            id,
            sample_id,
            status,
            metadata,
            created_at,
            last_updated,
            read_count,
            location,
            integrity_checksum: checksum,
        };

        Ok(sample)
    }

    /// Get database statistics
    pub fn get_statistics(&self) -> Result<DatabaseStatistics> {
        let total_samples: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM samples",
            [],
            |row| row.get(0),
        ).map_err(|e| SampleGuardError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get statistics: {}", e)
        )))?;

        let status_counts: Vec<(String, i64)> = self.conn
            .prepare("SELECT status, COUNT(*) FROM samples GROUP BY status")
            .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to prepare query: {}", e)
            )))?
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to execute query: {}", e)
            )))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| SampleGuardError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to parse rows: {}", e)
            )))?;

        let status_map: std::collections::HashMap<String, usize> = status_counts
            .into_iter()
            .map(|(k, v)| (k, v as usize))
            .collect();

        Ok(DatabaseStatistics {
            total_samples: total_samples as usize,
            status_counts: status_map,
        })
    }
}

/// History entry for sample tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub sample_id: String,
    pub status: SampleStatus,
    pub location: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatistics {
    pub total_samples: usize,
    pub status_counts: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::SampleMetadata;
    use chrono::Utc;

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
        Sample::new(id.to_string(), metadata, Some("Test Location".to_string()))
    }

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().unwrap();
        assert!(db.get_statistics().is_ok());
    }

    #[test]
    fn test_store_sample() {
        let db = Database::in_memory().unwrap();
        let sample = create_test_sample("TEST-001");
        assert!(db.store_sample(&sample).is_ok());
    }

    #[test]
    fn test_get_sample() {
        let db = Database::in_memory().unwrap();
        let sample = create_test_sample("TEST-002");
        db.store_sample(&sample).unwrap();
        
        let retrieved = db.get_sample("TEST-002").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().sample_id, "TEST-002");
    }

    #[test]
    fn test_get_nonexistent_sample() {
        let db = Database::in_memory().unwrap();
        let retrieved = db.get_sample("NONEXISTENT").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_all_samples() {
        let db = Database::in_memory().unwrap();
        let sample1 = create_test_sample("TEST-003");
        let sample2 = create_test_sample("TEST-004");
        
        db.store_sample(&sample1).unwrap();
        db.store_sample(&sample2).unwrap();
        
        let all = db.get_all_samples().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_samples_by_batch() {
        let db = Database::in_memory().unwrap();
        let sample1 = create_test_sample("TEST-005");
        db.store_sample(&sample1).unwrap();
        
        let batch_samples = db.get_samples_by_batch(&sample1.metadata.batch_number).unwrap();
        assert!(batch_samples.len() > 0);
    }

    #[test]
    fn test_get_samples_by_status() {
        let db = Database::in_memory().unwrap();
        let mut sample = create_test_sample("TEST-006");
        sample.update_status(SampleStatus::InTransit);
        db.store_sample(&sample).unwrap();
        
        let transit_samples = db.get_samples_by_status(SampleStatus::InTransit).unwrap();
        assert!(transit_samples.len() > 0);
    }

    #[test]
    fn test_delete_sample() {
        let db = Database::in_memory().unwrap();
        let sample = create_test_sample("TEST-007");
        db.store_sample(&sample).unwrap();
        
        let deleted = db.delete_sample("TEST-007").unwrap();
        assert!(deleted);
        
        let retrieved = db.get_sample("TEST-007").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_delete_nonexistent_sample() {
        let db = Database::in_memory().unwrap();
        let deleted = db.delete_sample("NONEXISTENT").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_add_history_entry() {
        let db = Database::in_memory().unwrap();
        // First create a sample so the foreign key constraint is satisfied
        let sample = create_test_sample("TEST-008");
        db.store_sample(&sample).unwrap();
        // Now we can add history entry
        assert!(db.add_history_entry("TEST-008", &SampleStatus::InProduction, None).is_ok());
    }

    #[test]
    fn test_get_sample_history() {
        let db = Database::in_memory().unwrap();
        let sample = create_test_sample("TEST-009");
        db.store_sample(&sample).unwrap();
        
        let mut sample2 = sample.clone();
        sample2.update_status(SampleStatus::InTransit);
        db.store_sample(&sample2).unwrap();
        
        let history = db.get_sample_history("TEST-009").unwrap();
        assert!(history.len() >= 2);
    }

    #[test]
    fn test_get_statistics() {
        let db = Database::in_memory().unwrap();
        let sample1 = create_test_sample("TEST-010");
        let sample2 = create_test_sample("TEST-011");
        
        db.store_sample(&sample1).unwrap();
        db.store_sample(&sample2).unwrap();
        
        let stats = db.get_statistics().unwrap();
        assert_eq!(stats.total_samples, 2);
    }

    #[test]
    fn test_store_duplicate_sample() {
        let db = Database::in_memory().unwrap();
        let sample = create_test_sample("TEST-012");
        db.store_sample(&sample).unwrap();
        
        // Store again - should replace
        let mut sample2 = sample.clone();
        sample2.update_status(SampleStatus::InTransit);
        db.store_sample(&sample2).unwrap();
        
        let retrieved = db.get_sample("TEST-012").unwrap().unwrap();
        assert_eq!(retrieved.status, SampleStatus::InTransit);
    }

    #[test]
    fn test_sample_with_no_expiry() {
        let metadata = SampleMetadata {
            batch_number: "BATCH-013".to_string(),
            production_date: Utc::now(),
            expiry_date: None,
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
        };
        let sample = Sample::new("TEST-013".to_string(), metadata, None);
        
        let db = Database::in_memory().unwrap();
        db.store_sample(&sample).unwrap();
        
        let retrieved = db.get_sample("TEST-013").unwrap().unwrap();
        assert!(retrieved.metadata.expiry_date.is_none());
    }

    #[test]
    fn test_sample_with_no_temperature() {
        let metadata = SampleMetadata {
            batch_number: "BATCH-014".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: None,
            storage_conditions: "Room Temperature".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
        };
        let sample = Sample::new("TEST-014".to_string(), metadata, None);
        
        let db = Database::in_memory().unwrap();
        db.store_sample(&sample).unwrap();
        
        let retrieved = db.get_sample("TEST-014").unwrap().unwrap();
        assert!(retrieved.metadata.temperature_range.is_none());
    }

    #[test]
    fn test_multiple_batches() {
        let db = Database::in_memory().unwrap();
        
        let metadata1 = SampleMetadata {
            batch_number: "BATCH-A".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
        };
        let sample1 = Sample::new("TEST-015".to_string(), metadata1, None);
        
        let metadata2 = SampleMetadata {
            batch_number: "BATCH-B".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
        };
        let sample2 = Sample::new("TEST-016".to_string(), metadata2, None);
        
        db.store_sample(&sample1).unwrap();
        db.store_sample(&sample2).unwrap();
        
        let batch_a = db.get_samples_by_batch("BATCH-A").unwrap();
        assert_eq!(batch_a.len(), 1);
        
        let batch_b = db.get_samples_by_batch("BATCH-B").unwrap();
        assert_eq!(batch_b.len(), 1);
    }

    #[test]
    fn test_history_tracking() {
        let db = Database::in_memory().unwrap();
        let mut sample = create_test_sample("TEST-017");
        
        db.store_sample(&sample).unwrap();
        sample.update_status(SampleStatus::InTransit);
        db.store_sample(&sample).unwrap();
        sample.update_status(SampleStatus::Stored);
        db.store_sample(&sample).unwrap();
        
        let history = db.get_sample_history("TEST-017").unwrap();
        assert!(history.len() >= 3);
    }

    #[test]
    fn test_empty_statistics() {
        let db = Database::in_memory().unwrap();
        let stats = db.get_statistics().unwrap();
        assert_eq!(stats.total_samples, 0);
    }
}

