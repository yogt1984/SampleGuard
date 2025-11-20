use sample_guard::*;
use sample_guard::hardware::{HardwareDriver};
use sample_guard::database::Database;
use sample_guard::inventory::InventoryManager;
use sample_guard::temperature::{TemperatureMonitor, MockTemperatureSensor};
use sample_guard::audit::AuditLogger;
use sample_guard::reader::MockRFIDReader;
use chrono::Utc;
use std::time::Duration;

fn print_header(title: &str) {
    println!("\n{}", "=".repeat(80));
    println!("{}", title);
    println!("{}", "=".repeat(80));
}

fn print_section(title: &str) {
    println!("\n{}", "-".repeat(80));
    println!("{}", title);
    println!("{}", "-".repeat(80));
}

fn print_transaction(step: u32, operation: &str, status: &str, details: &str) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] STEP {:03} | {} | {} | {}", timestamp, step, operation, status, details);
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    print_header("SampleGuard System Demonstration - Complete Transaction Log");
    println!("This demonstration shows a comprehensive sequence of operations");
    println!("demonstrating all system capabilities and proving functional operation.\n");
    
    let mut step_counter = 1u32;
    
    // ============================================================================
    // PHASE 1: SYSTEM INITIALIZATION
    // ============================================================================
    print_section("PHASE 1: SYSTEM INITIALIZATION");
    
    print_transaction(step_counter, "INIT", "START", "Initializing SampleGuard system components");
    step_counter += 1;
    
    // Initialize database
    print_transaction(step_counter, "DB_INIT", "IN_PROGRESS", "Creating in-memory SQLite database");
    let db = Database::in_memory()?;
    print_transaction(step_counter, "DB_INIT", "SUCCESS", "Database initialized successfully");
    step_counter += 1;
    
    // Initialize inventory manager
    print_transaction(step_counter, "INV_INIT", "IN_PROGRESS", "Creating inventory manager");
    let mut inventory = InventoryManager::new();
    print_transaction(step_counter, "INV_INIT", "SUCCESS", "Inventory manager ready");
    step_counter += 1;
    
    // Initialize temperature monitor
    print_transaction(step_counter, "TEMP_INIT", "IN_PROGRESS", "Creating temperature monitoring system");
    let sensor = Box::new(MockTemperatureSensor::new("MAIN-SENSOR-001".to_string(), 5.0));
    let mut temp_monitor = TemperatureMonitor::new(sensor, (2.0, 8.0))?;
    print_transaction(step_counter, "TEMP_INIT", "SUCCESS", "Temperature monitor initialized (range: 2.0-8.0°C)");
    step_counter += 1;
    
    // Initialize audit logger
    print_transaction(step_counter, "AUDIT_INIT", "IN_PROGRESS", "Creating audit logging system");
    let mut audit_logger = AuditLogger::new();
    print_transaction(step_counter, "AUDIT_INIT", "SUCCESS", "Audit logger ready");
    step_counter += 1;
    
    // Initialize hardware drivers
    print_transaction(step_counter, "HW_INIT", "IN_PROGRESS", "Initializing hardware emulation drivers");
    let mut hardware_driver = HardwareDriver::new();
    hardware_driver.initialize_all().map_err(|e| format!("Hardware initialization failed: {}", e))?;
    print_transaction(step_counter, "HW_INIT", "SUCCESS", "Hardware drivers initialized (Impinj + Zebra)");
    step_counter += 1;
    
    // Setup demo tags
    print_transaction(step_counter, "TAG_SETUP", "IN_PROGRESS", "Configuring simulated RFID tags");
    hardware_driver.setup_demo_tags();
    print_transaction(step_counter, "TAG_SETUP", "SUCCESS", "5 demo tags configured");
    step_counter += 1;
    
    print_transaction(step_counter, "INIT", "COMPLETE", "All system components initialized");
    step_counter += 1;
    
    // ============================================================================
    // PHASE 2: SAMPLE CREATION AND STORAGE
    // ============================================================================
    print_section("PHASE 2: SAMPLE CREATION AND STORAGE");
    
    let mut samples = Vec::new();
    
    for i in 1..=5 {
        let sample_id = format!("SAMPLE-{:03}", i);
        print_transaction(step_counter, "SAMPLE_CREATE", "IN_PROGRESS", &format!("Creating sample {}", sample_id));
        
        let metadata = SampleMetadata {
            batch_number: format!("BATCH-{:03}", (i + 1) / 2),
            production_date: Utc::now() - chrono::Duration::days(i as i64 * 10),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365 - (i as i64 * 10))),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "PharmaCorp Inc.".to_string(),
            product_line: if i % 2 == 0 { "Vaccines" } else { "Medications" }.to_string(),
        };
        
        let sample = Sample::new(sample_id.clone(), metadata, Some(format!("Warehouse-{}", (i % 3) + 1)));
        samples.push(sample.clone());
        
        // Store in database
        db.store_sample(&sample)?;
        print_transaction(step_counter, "SAMPLE_STORE", "SUCCESS", &format!("Sample {} stored in database", sample_id));
        step_counter += 1;
        
        // Log audit event
        audit_logger.log_sample_created(&sample, Some("system_demo".to_string()))?;
        print_transaction(step_counter, "AUDIT_LOG", "SUCCESS", &format!("Sample creation logged for {}", sample_id));
        step_counter += 1;
    }
    
    print_transaction(step_counter, "SAMPLE_CREATE", "COMPLETE", "5 samples created and stored");
    step_counter += 1;
    
    // ============================================================================
    // PHASE 3: RFID INVENTORY SCANNING
    // ============================================================================
    print_section("PHASE 3: RFID INVENTORY SCANNING");
    
    print_transaction(step_counter, "INVENTORY_SCAN", "IN_PROGRESS", "Starting RFID inventory scan with Impinj reader");
    step_counter += 1;
    
    let mut reader = MockRFIDReader::new();
    let scan_results = inventory.scan_tags(&mut reader, Duration::from_millis(500))?;
    print_transaction(step_counter, "INVENTORY_SCAN", "SUCCESS", &format!("Found {} tags in inventory", scan_results.len()));
    step_counter += 1;
    
    for (idx, result) in scan_results.iter().enumerate() {
        print_transaction(step_counter, "TAG_DETECTED", "INFO", 
            &format!("Tag {}: EPC={}, RSSI={}dBm, Antenna={}", 
                idx + 1, result.epc, result.rssi, result.antenna));
        step_counter += 1;
    }
    
    // Hardware driver inventory
    print_transaction(step_counter, "HW_INVENTORY", "IN_PROGRESS", "Performing hardware emulation inventory scan");
    let hw_tags = hardware_driver.perform_inventory_scan().map_err(|e| format!("Inventory scan failed: {}", e))?;
    print_transaction(step_counter, "HW_INVENTORY", "SUCCESS", &format!("Hardware scan found {} tags", hw_tags.len()));
    step_counter += 1;
    
    // Generate inventory report
    print_transaction(step_counter, "INVENTORY_REPORT", "IN_PROGRESS", "Generating inventory report");
    let report = inventory.generate_report();
    print_transaction(step_counter, "INVENTORY_REPORT", "SUCCESS", 
        &format!("Report: {} total tags", report.total_tags));
    step_counter += 1;
    
    // ============================================================================
    // PHASE 4: TEMPERATURE MONITORING
    // ============================================================================
    print_section("PHASE 4: TEMPERATURE MONITORING");
    
    for i in 1..=10 {
        print_transaction(step_counter, "TEMP_READ", "IN_PROGRESS", &format!("Reading temperature (reading #{})", i));
        
        // Simulate temperature variations
        let temp_value = 5.0 + (i as f32 * 0.3) - 1.5;
        let sensor = Box::new(MockTemperatureSensor::new("TEMP-SENSOR".to_string(), temp_value));
        let mut monitor = TemperatureMonitor::new(sensor, (2.0, 8.0))?;
        
        let reading = monitor.read_temperature(None)?;
        print_transaction(step_counter, "TEMP_READ", "SUCCESS", 
            &format!("Temperature: {:.2}°C, Timestamp: {}", 
                reading.temperature, reading.timestamp.format("%H:%M:%S")));
        step_counter += 1;
        
        // Check for violations
        let violations = monitor.get_violations();
        if !violations.is_empty() {
            print_transaction(step_counter, "TEMP_VIOLATION", "WARNING", 
                &format!("Temperature violation detected: {} violations", violations.len()));
            for violation in &violations {
                    audit_logger.log_temperature_violation(
                        None,
                        violation.reading.temperature,
                        violation.expected_range,
                        Some("system_demo".to_string())
                    )?;
            }
            step_counter += 1;
        }
        
        // Update main monitor
        temp_monitor.read_temperature(None)?;
    }
    
    // Get temperature statistics
    print_transaction(step_counter, "TEMP_STATS", "IN_PROGRESS", "Calculating temperature statistics");
    let temp_stats = temp_monitor.get_statistics();
    let violations_count = temp_monitor.get_violations().len();
    print_transaction(step_counter, "TEMP_STATS", "SUCCESS", 
        &format!("Stats: {} readings, Avg: {:.2}°C, Min: {:.2}°C, Max: {:.2}°C, Violations: {}", 
            temp_stats.total_readings, 
            temp_stats.average_temperature.unwrap_or(0.0), 
            temp_stats.min_temperature.unwrap_or(0.0), 
            temp_stats.max_temperature.unwrap_or(0.0), 
            violations_count));
    step_counter += 1;
    
    // ============================================================================
    // PHASE 5: SAMPLE STATUS TRANSITIONS
    // ============================================================================
    print_section("PHASE 5: SAMPLE STATUS TRANSITIONS");
    
    let statuses = vec![
        SampleStatus::InProduction,
        SampleStatus::InTransit,
        SampleStatus::Stored,
        SampleStatus::InUse,
    ];
    
    for (idx, sample) in samples.iter().enumerate() {
        if idx < statuses.len() {
            let old_status = sample.status;
            let new_status = statuses[idx];
            
            print_transaction(step_counter, "STATUS_UPDATE", "IN_PROGRESS", 
                &format!("Updating {}: {:?} -> {:?}", sample.sample_id, old_status, new_status));
            
            let mut updated_sample = sample.clone();
            updated_sample.update_status(new_status);
            db.store_sample(&updated_sample)?;
            
            audit_logger.log_status_change(
                &sample.sample_id,
                old_status,
                new_status,
                Some("system_demo".to_string())
            )?;
            
            print_transaction(step_counter, "STATUS_UPDATE", "SUCCESS", 
                &format!("Status updated and logged for {}", sample.sample_id));
            step_counter += 1;
        }
    }
    
    // ============================================================================
    // PHASE 6: DATABASE OPERATIONS
    // ============================================================================
    print_section("PHASE 6: DATABASE OPERATIONS");
    
    // Query samples by batch
    print_transaction(step_counter, "DB_QUERY", "IN_PROGRESS", "Querying samples by batch BATCH-001");
    let batch_samples = db.get_samples_by_batch("BATCH-001")?;
    print_transaction(step_counter, "DB_QUERY", "SUCCESS", &format!("Found {} samples in batch", batch_samples.len()));
    step_counter += 1;
    
    // Query by status
    print_transaction(step_counter, "DB_QUERY", "IN_PROGRESS", "Querying samples with status InTransit");
    let transit_samples = db.get_samples_by_status(SampleStatus::InTransit)?;
    print_transaction(step_counter, "DB_QUERY", "SUCCESS", &format!("Found {} samples in transit", transit_samples.len()));
    step_counter += 1;
    
    // Get sample history
    if !samples.is_empty() {
        print_transaction(step_counter, "DB_HISTORY", "IN_PROGRESS", &format!("Retrieving history for {}", samples[0].sample_id));
        let history = db.get_sample_history(&samples[0].sample_id)?;
        print_transaction(step_counter, "DB_HISTORY", "SUCCESS", &format!("Found {} history entries", history.len()));
        step_counter += 1;
    }
    
    // Get database statistics
    print_transaction(step_counter, "DB_STATS", "IN_PROGRESS", "Calculating database statistics");
    let db_stats = db.get_statistics()?;
    print_transaction(step_counter, "DB_STATS", "SUCCESS", 
        &format!("Total samples: {}", db_stats.total_samples));
    step_counter += 1;
    
    // ============================================================================
    // PHASE 7: HARDWARE OPERATIONS
    // ============================================================================
    print_section("PHASE 7: HARDWARE OPERATIONS");
    
    // Read tags from hardware
    if !hw_tags.is_empty() {
        for (_idx, epc) in hw_tags.iter().take(3).enumerate() {
            print_transaction(step_counter, "HW_READ", "IN_PROGRESS", &format!("Reading tag {} from Impinj reader", epc));
            match hardware_driver.read_tag_impinj(epc) {
                Ok(data) => {
                    print_transaction(step_counter, "HW_READ", "SUCCESS", 
                        &format!("Tag {} read successfully, {} bytes", epc, data.len()));
                    step_counter += 1;
                }
                Err(e) => {
                    print_transaction(step_counter, "HW_READ", "ERROR", 
                        &format!("Failed to read tag {}: {}", epc, e));
                    step_counter += 1;
                }
            }
        }
    }
    
    // Get reader configurations
    print_transaction(step_counter, "HW_CONFIG", "IN_PROGRESS", "Retrieving Impinj reader configuration");
    let impinj_config = hardware_driver.get_reader_config("impinj")?;
    print_transaction(step_counter, "HW_CONFIG", "SUCCESS", &format!("Impinj config: {}", impinj_config));
    step_counter += 1;
    
    print_transaction(step_counter, "HW_CONFIG", "IN_PROGRESS", "Retrieving Zebra reader configuration");
    let zebra_config = hardware_driver.get_reader_config("zebra")?;
    print_transaction(step_counter, "HW_CONFIG", "SUCCESS", &format!("Zebra config: {}", zebra_config));
    step_counter += 1;
    
    // ============================================================================
    // PHASE 8: AUDIT LOGGING AND QUERIES
    // ============================================================================
    print_section("PHASE 8: AUDIT LOGGING AND QUERIES");
    
    // Log various events
    print_transaction(step_counter, "AUDIT_LOG", "IN_PROGRESS", "Logging sample read event");
    audit_logger.log_sample_read(&samples[0], Some("system_demo".to_string()))?;
    print_transaction(step_counter, "AUDIT_LOG", "SUCCESS", "Sample read event logged");
    step_counter += 1;
    
    print_transaction(step_counter, "AUDIT_LOG", "IN_PROGRESS", "Logging integrity violation");
    audit_logger.log_integrity_violation(
        &samples[0].sample_id,
        vec!["Test violation for demonstration".to_string()],
        Some("system_demo".to_string())
    )?;
    print_transaction(step_counter, "AUDIT_LOG", "SUCCESS", "Integrity violation logged");
    step_counter += 1;
    
    // Query audit events
    print_transaction(step_counter, "AUDIT_QUERY", "IN_PROGRESS", "Querying all audit events");
    let all_events = audit_logger.get_all_events();
    print_transaction(step_counter, "AUDIT_QUERY", "SUCCESS", &format!("Found {} total audit events", all_events.len()));
    step_counter += 1;
    
    // Query by sample
    if !samples.is_empty() {
        print_transaction(step_counter, "AUDIT_QUERY", "IN_PROGRESS", &format!("Querying events for {}", samples[0].sample_id));
        let sample_events = audit_logger.get_events_by_sample(&samples[0].sample_id);
        print_transaction(step_counter, "AUDIT_QUERY", "SUCCESS", &format!("Found {} events for sample", sample_events.len()));
        step_counter += 1;
    }
    
    // Get audit statistics
    print_transaction(step_counter, "AUDIT_STATS", "IN_PROGRESS", "Calculating audit statistics");
    let audit_stats = audit_logger.get_statistics();
    print_transaction(step_counter, "AUDIT_STATS", "SUCCESS", 
        &format!("Total events: {}, By type: {:?}, By severity: {:?}", 
            audit_stats.total_events, audit_stats.type_counts, audit_stats.severity_counts));
    step_counter += 1;
    
    // ============================================================================
    // PHASE 9: INTEGRITY VALIDATION
    // ============================================================================
    print_section("PHASE 9: INTEGRITY VALIDATION");
    
    let validator = IntegrityValidator::new();
    
    for sample in &samples {
        print_transaction(step_counter, "INTEGRITY_CHECK", "IN_PROGRESS", &format!("Validating integrity of {}", sample.sample_id));
        match validator.validate(sample) {
            Ok(validation) => {
                if validation.is_valid() {
                    print_transaction(step_counter, "INTEGRITY_CHECK", "SUCCESS", 
                        &format!("Sample {} passed integrity validation", sample.sample_id));
                } else {
                    print_transaction(step_counter, "INTEGRITY_CHECK", "WARNING", 
                        &format!("Sample {} failed validation", sample.sample_id));
                }
            }
            Err(e) => {
                print_transaction(step_counter, "INTEGRITY_CHECK", "ERROR", 
                    &format!("Validation error for {}: {}", sample.sample_id, e));
            }
        }
        step_counter += 1;
    }
    
    // ============================================================================
    // PHASE 10: SYSTEM STATISTICS AND SUMMARY
    // ============================================================================
    print_section("PHASE 10: SYSTEM STATISTICS AND SUMMARY");
    
    print_transaction(step_counter, "SYS_STATS", "IN_PROGRESS", "Compiling system-wide statistics");
    step_counter += 1;
    
    let db_stats = db.get_statistics()?;
    let temp_stats = temp_monitor.get_statistics();
    let audit_stats = audit_logger.get_statistics();
    let inv_report = inventory.generate_report();
    
    print_transaction(step_counter, "SYS_STATS", "SUCCESS", "System statistics compiled");
    step_counter += 1;
    
    println!("\n╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    SYSTEM STATISTICS SUMMARY                                ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Database:                                                                 ║");
    println!("║   • Total Samples: {:>50} ║", db_stats.total_samples);
    println!("║   • Status Distribution: {:>45} ║", format!("{:?}", db_stats.status_counts));
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Temperature Monitoring:                                                    ║");
    println!("║   • Total Readings: {:>50} ║", temp_stats.total_readings);
    println!("║   • Average Temperature: {:>45.2}°C ║", temp_stats.average_temperature.unwrap_or(0.0));
    let violations_count = temp_monitor.get_violations().len();
    println!("║   • Violations: {:>54} ║", violations_count);
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Audit Logging:                                                             ║");
    println!("║   • Total Events: {:>52} ║", audit_stats.total_events);
    println!("║   • Events by Type: {:>48} ║", format!("{:?}", audit_stats.type_counts));
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Inventory:                                                                 ║");
    println!("║   • Total Tags: {:>54} ║", inv_report.total_tags);
    println!("║   • Antennas Used: {:>48} ║", inv_report.antennas.len());
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Transaction Log:                                                           ║");
    println!("║   • Total Operations: {:>48} ║", step_counter - 1);
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    
    // Print hardware driver events
    print_section("HARDWARE DRIVER EVENTS");
    hardware_driver.print_events();
    
    print_transaction(step_counter, "DEMO", "COMPLETE", "System demonstration completed successfully");
    
    println!("\n{}", "=".repeat(80));
    println!("DEMONSTRATION COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nThis transaction log demonstrates:");
    println!("  ✓ Complete system initialization");
    println!("  ✓ Sample creation and management");
    println!("  ✓ RFID inventory scanning");
    println!("  ✓ Temperature monitoring with violation detection");
    println!("  ✓ Status transitions and tracking");
    println!("  ✓ Database operations and queries");
    println!("  ✓ Hardware emulation (Impinj & Zebra)");
    println!("  ✓ Comprehensive audit logging");
    println!("  ✓ Integrity validation");
    println!("  ✓ System-wide statistics");
    println!("\nTotal operations logged: {}", step_counter);
    println!("All systems functional and operational.\n");
    
    Ok(())
}

