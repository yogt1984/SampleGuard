use sample_guard::*;

fn main() -> Result<()> {
    env_logger::init();
    
    println!("SampleGuard - RFID Sample Integrity Tracking System");
    println!("===================================================\n");
    
    // Demonstrate the system with a mock reader
    #[cfg(test)]
    {
        let reader = Box::new(reader::MockRFIDReader::new());
        let mut guard = SampleGuard::new(reader);
        
        // Create a sample
        let metadata = sample::SampleMetadata {
            batch_number: "BATCH2024-001".to_string(),
            production_date: chrono::Utc::now(),
            expiry_date: Some(chrono::Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated 2-8°C".to_string(),
            manufacturer: "PharmaCorp".to_string(),
            product_line: "Vaccines".to_string(),
        };
        
        let sample = sample::Sample::new(
            "SAMPLE-2024-001".to_string(),
            metadata,
            Some("Warehouse A, Shelf 3".to_string()),
        );
        
        println!("Created sample: {}", sample.sample_id);
        println!("Status: {:?}", sample.status);
        println!("Batch: {}", sample.metadata.batch_number);
        println!();
        
        // Write sample to tag
        println!("Writing sample to RFID tag...");
        guard.write_sample(&sample)?;
        println!("✓ Sample written successfully\n");
        
        // Read sample from tag
        println!("Reading sample from RFID tag...");
        let read_sample = guard.read_sample()?;
        println!("✓ Sample read successfully");
        println!("Sample ID: {}", read_sample.sample_id);
        println!("Read count: {}", read_sample.read_count);
        println!();
        
        // Check integrity
        println!("Validating sample integrity...");
        let validation = guard.check_integrity(&read_sample)?;
        if validation.is_valid() {
            println!("✓ Sample integrity validated");
        } else {
            println!("✗ Integrity violations detected:");
            for violation in &validation.violations {
                println!("  - {:?}", violation);
            }
        }
        
        if validation.has_warnings() {
            println!("\n⚠ Warnings:");
            for warning in &validation.warnings {
                println!("  - {:?}", warning);
            }
        }
    }
    
    #[cfg(not(test))]
    {
        println!("Run with 'cargo test' to see the system in action");
        println!("Or implement hardware reader integration for production use");
    }
    
    Ok(())
}

