use crate::api::handlers::AppState;
use crate::api::routes::configure_routes;
use crate::database::Database;
use crate::inventory::InventoryManager;
use crate::temperature::{TemperatureMonitor, MockTemperatureSensor};
use crate::audit::AuditLogger;
use crate::reader::MockRFIDReader;
use crate::SampleGuard;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

/// Create application state
pub fn create_app_state() -> AppState {
    // Create in-memory database for testing/demo
    let database = Database::in_memory()
        .expect("Failed to create database");
    
    let inventory = InventoryManager::new();
    let sensor = Box::new(MockTemperatureSensor::new("API-SENSOR".to_string(), 5.0));
    let temperature_monitor = TemperatureMonitor::new(sensor, (2.0, 8.0))
        .expect("Failed to create temperature monitor");
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

/// Start the HTTP server
pub async fn start_server(host: &str, port: u16) -> std::io::Result<()> {
    let app_state = create_app_state();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}

