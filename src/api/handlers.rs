use crate::api::error::ApiError;
use crate::api::models::*;
use crate::database::Database;
use crate::inventory::InventoryManager;
use crate::temperature::TemperatureMonitor;
use crate::audit::{AuditLogger, AuditEvent};
use crate::sample::{Sample, SampleStatus, SampleMetadata};
use crate::reader::MockRFIDReader;
use crate::SampleGuard;
use actix_web::{web, HttpResponse, Result as ActixResult};
use std::sync::{Arc, Mutex};
use chrono::Utc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Mutex<Database>>,
    pub inventory: Arc<Mutex<InventoryManager>>,
    pub temperature_monitor: Arc<Mutex<TemperatureMonitor>>,
    pub audit_logger: Arc<Mutex<AuditLogger>>,
    pub sample_guard: Arc<Mutex<SampleGuard>>,
}

/// Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
    }))
}

/// Get all samples
pub async fn get_samples(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let samples = db.get_all_samples()?;
    
    let responses: Vec<SampleResponse> = samples.iter().map(SampleResponse::from).collect();
    
    Ok(HttpResponse::Ok().json(responses))
}

/// Get sample by ID
pub async fn get_sample(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let sample_id = path.into_inner();
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let sample = db.get_sample(&sample_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Sample {} not found", sample_id)))?;
    
    Ok(HttpResponse::Ok().json(SampleResponse::from(&sample)))
}

/// Create a new sample
pub async fn create_sample(
    state: web::Data<AppState>,
    req: web::Json<CreateSampleRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = req.into_inner();
    
    let metadata = SampleMetadata {
        batch_number: req.batch_number,
        production_date: req.production_date,
        expiry_date: req.expiry_date,
        temperature_range: req.temperature_range,
        storage_conditions: req.storage_conditions,
        manufacturer: req.manufacturer,
        product_line: req.product_line,
    };
    
    let sample = Sample::new(req.sample_id.clone(), metadata, req.location);
    
    // Store in database
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    db.store_sample(&sample)?;
    
    // Log audit event
    let mut logger = state.audit_logger.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    logger.log_sample_created(&sample, None)?;
    
    Ok(HttpResponse::Created().json(SampleResponse::from(&sample)))
}

/// Update sample status
pub async fn update_sample_status(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: web::Json<UpdateSampleStatusRequest>,
) -> Result<HttpResponse, ApiError> {
    let sample_id = path.into_inner();
    let req = req.into_inner();
    
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let mut sample = db.get_sample(&sample_id)?
        .ok_or_else(|| ApiError::NotFound(format!("Sample {} not found", sample_id)))?;
    
    let old_status = sample.status;
    let new_status = match req.status.as_str() {
        "InProduction" => SampleStatus::InProduction,
        "InTransit" => SampleStatus::InTransit,
        "Stored" => SampleStatus::Stored,
        "InUse" => SampleStatus::InUse,
        "Consumed" => SampleStatus::Consumed,
        "Discarded" => SampleStatus::Discarded,
        "Compromised" => SampleStatus::Compromised,
        _ => return Err(ApiError::Validation(format!("Invalid status: {}", req.status))),
    };
    
    sample.update_status(new_status);
    if let Some(location) = req.location {
        sample.update_location(location);
    }
    
    db.store_sample(&sample)?;
    
    // Log audit event
    let mut logger = state.audit_logger.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    logger.log_status_change(&sample_id, old_status, new_status, None)?;
    
    Ok(HttpResponse::Ok().json(SampleResponse::from(&sample)))
}

/// Delete a sample
pub async fn delete_sample(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let sample_id = path.into_inner();
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let deleted = db.delete_sample(&sample_id)?;
    if !deleted {
        return Err(ApiError::NotFound(format!("Sample {} not found", sample_id)));
    }
    
    Ok(HttpResponse::NoContent().finish())
}

/// Get samples by batch
pub async fn get_samples_by_batch(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let batch_number = path.into_inner();
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let samples = db.get_samples_by_batch(&batch_number)?;
    let responses: Vec<SampleResponse> = samples.iter().map(SampleResponse::from).collect();
    
    Ok(HttpResponse::Ok().json(responses))
}

/// Scan inventory
pub async fn scan_inventory(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let mut inventory = state.inventory.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let mut reader = MockRFIDReader::new();
    
    // In a real implementation, this would use the actual reader from state
    let results = inventory.scan_tags(&mut reader, std::time::Duration::from_millis(100))?;
    
    Ok(HttpResponse::Ok().json(InventoryScanResponse {
        tags: results.clone(),
        count: results.len(),
        timestamp: Utc::now(),
    }))
}

/// Get inventory report
pub async fn get_inventory_report(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let inventory = state.inventory.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let report = inventory.generate_report();
    
    Ok(HttpResponse::Ok().json(report))
}

/// Read temperature
pub async fn read_temperature(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let mut monitor = state.temperature_monitor.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let reading = monitor.read_temperature(None)?;
    let violations = monitor.get_violations();
    
    Ok(HttpResponse::Ok().json(TemperatureResponse {
        reading: reading.clone(),
        within_range: monitor.is_within_range(reading.temperature),
        violations: violations.len(),
    }))
}

/// Get temperature statistics
pub async fn get_temperature_statistics(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let monitor = state.temperature_monitor.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let stats = monitor.get_statistics();
    
    Ok(HttpResponse::Ok().json(stats))
}

/// Get audit events
pub async fn get_audit_events(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let logger = state.audit_logger.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let events = if let Some(sample_id) = query.get("sample_id") {
        logger.get_events_by_sample(sample_id)
    } else if let Some(event_type) = query.get("event_type") {
        use crate::audit::AuditEventType;
        let event_type_enum = match event_type.as_str() {
            "SampleCreated" => AuditEventType::SampleCreated,
            "SampleRead" => AuditEventType::SampleRead,
            "ViolationDetected" => AuditEventType::ViolationDetected,
            _ => return Err(ApiError::Validation(format!("Invalid event type: {}", event_type))),
        };
        logger.get_events_by_type(&event_type_enum)
    } else {
        logger.get_all_events()
    };
    
    let events_vec: Vec<&AuditEvent> = events;
    let events_cloned: Vec<AuditEvent> = events_vec.iter().map(|e| (*e).clone()).collect();
    let total = events_cloned.len();
    
    Ok(HttpResponse::Ok().json(AuditQueryResponse {
        events: events_cloned,
        total,
    }))
}

/// Get audit statistics
pub async fn get_audit_statistics(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let logger = state.audit_logger.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let stats = logger.get_statistics();
    
    Ok(HttpResponse::Ok().json(stats))
}

/// Get system statistics
pub async fn get_statistics(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let db = state.database.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let inventory = state.inventory.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let monitor = state.temperature_monitor.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    let logger = state.audit_logger.lock().map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let db_stats = db.get_statistics()?;
    let temp_stats = monitor.get_statistics();
    let audit_stats = logger.get_statistics();
    
    Ok(HttpResponse::Ok().json(StatisticsResponse {
        samples: db_stats.total_samples,
        inventory_tags: inventory.tag_count(),
        temperature_readings: temp_stats.total_readings,
        audit_events: audit_stats.total_events,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::inventory::InventoryManager;
    use crate::temperature::{TemperatureMonitor, MockTemperatureSensor};
    use crate::audit::AuditLogger;
    use crate::reader::MockRFIDReader;
    use crate::SampleGuard;

    fn create_test_state() -> AppState {
        let database = Database::in_memory().unwrap();
        let inventory = InventoryManager::new();
        let sensor = Box::new(MockTemperatureSensor::new("TEST-SENSOR".to_string(), 5.0));
        let temperature_monitor = TemperatureMonitor::new(sensor, (2.0, 8.0)).unwrap();
        let audit_logger = AuditLogger::new();
        let reader = Box::new(MockRFIDReader::new());
        let sample_guard = SampleGuard::new(reader);
        
        AppState {
            database: Arc::new(Mutex::new(database)),
            inventory: Arc::new(Mutex::new(inventory)),
            temperature_monitor: Arc::new(Mutex::new(temperature_monitor)),
            audit_logger: Arc::new(Mutex::new(audit_logger)),
            sample_guard: Arc::new(Mutex::new(sample_guard)),
        }
    }

    #[test]
    fn test_app_state_creation() {
        let _state = create_test_state();
    }

    #[actix_web::test]
    async fn test_health_check_handler() {
        let resp = health_check().await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_get_samples_empty() {
        let state = web::Data::new(create_test_state());
        let result = get_samples(state).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_get_sample_not_found() {
        let state = web::Data::new(create_test_state());
        let path = web::Path::from("NONEXISTENT".to_string());
        let result = get_sample(state, path).await;
        assert!(result.is_err());
        if let Err(ApiError::NotFound(_)) = result {
            // Expected
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[actix_web::test]
    async fn test_create_sample_handler() {
        let state = web::Data::new(create_test_state());
        let req = web::Json(CreateSampleRequest {
            sample_id: "TEST-001".to_string(),
            batch_number: "BATCH-001".to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
            location: None,
        });
        
        let result = create_sample(state, req).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status(), 201);
    }

    #[actix_web::test]
    async fn test_delete_sample_not_found() {
        let state = web::Data::new(create_test_state());
        let path = web::Path::from("NONEXISTENT".to_string());
        let result = delete_sample(state, path).await;
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_get_inventory_report() {
        let state = web::Data::new(create_test_state());
        let result = get_inventory_report(state).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_read_temperature() {
        let state = web::Data::new(create_test_state());
        let result = read_temperature(state).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_get_temperature_statistics() {
        let state = web::Data::new(create_test_state());
        let result = get_temperature_statistics(state).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_get_audit_statistics() {
        let state = web::Data::new(create_test_state());
        let result = get_audit_statistics(state).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_get_statistics() {
        let state = web::Data::new(create_test_state());
        let result = get_statistics(state).await;
        assert!(result.is_ok());
    }
}
