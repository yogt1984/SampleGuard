use actix_web::{test, web, App};
use sample_guard::api::{configure_routes, create_app_state};
use sample_guard::api::models::*;
use chrono::Utc;

#[actix_web::test]
async fn test_health_check() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: HealthResponse = test::read_body_json(resp).await;
    assert_eq!(body.status, "ok");
}

#[actix_web::test]
async fn test_create_sample() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-001".to_string(),
        batch_number: "BATCH-API-001".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: Some("Test Location".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    
    let body: SampleResponse = test::read_body_json(resp).await;
    assert_eq!(body.sample_id, "API-TEST-001");
}

#[actix_web::test]
async fn test_get_sample() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a sample first
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-002".to_string(),
        batch_number: "BATCH-API-002".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    test::call_service(&app, req).await;
    
    // Get the sample
    let req = test::TestRequest::get()
        .uri("/api/v1/samples/API-TEST-002")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: SampleResponse = test::read_body_json(resp).await;
    assert_eq!(body.sample_id, "API-TEST-002");
}

#[actix_web::test]
async fn test_get_nonexistent_sample() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/samples/NONEXISTENT")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_get_all_samples() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a few samples
    for i in 1..=3 {
        let create_req = CreateSampleRequest {
            sample_id: format!("API-TEST-{}", i),
            batch_number: format!("BATCH-API-{}", i),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
            location: None,
        };
        
        let req = test::TestRequest::post()
            .uri("/api/v1/samples")
            .set_json(&create_req)
            .to_request();
        test::call_service(&app, req).await;
    }
    
    let req = test::TestRequest::get()
        .uri("/api/v1/samples")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Vec<SampleResponse> = test::read_body_json(resp).await;
    assert!(body.len() >= 3);
}

#[actix_web::test]
async fn test_update_sample_status() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a sample
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-003".to_string(),
        batch_number: "BATCH-API-003".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    test::call_service(&app, req).await;
    
    // Update status
    let update_req = UpdateSampleStatusRequest {
        status: "InTransit".to_string(),
        location: Some("New Location".to_string()),
    };
    
    let req = test::TestRequest::put()
        .uri("/api/v1/samples/API-TEST-003/status")
        .set_json(&update_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: SampleResponse = test::read_body_json(resp).await;
    assert_eq!(body.status, "InTransit");
    assert_eq!(body.location, Some("New Location".to_string()));
}

#[actix_web::test]
async fn test_delete_sample() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a sample
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-004".to_string(),
        batch_number: "BATCH-API-004".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    test::call_service(&app, req).await;
    
    // Delete the sample
    let req = test::TestRequest::delete()
        .uri("/api/v1/samples/API-TEST-004")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);
    
    // Verify it's deleted
    let req = test::TestRequest::get()
        .uri("/api/v1/samples/API-TEST-004")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_get_samples_by_batch() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create samples with same batch
    let batch = "BATCH-SHARED";
    for i in 1..=2 {
        let create_req = CreateSampleRequest {
            sample_id: format!("API-TEST-BATCH-{}", i),
            batch_number: batch.to_string(),
            production_date: Utc::now(),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            temperature_range: Some((2.0, 8.0)),
            storage_conditions: "Refrigerated".to_string(),
            manufacturer: "Test".to_string(),
            product_line: "Test".to_string(),
            location: None,
        };
        
        let req = test::TestRequest::post()
            .uri("/api/v1/samples")
            .set_json(&create_req)
            .to_request();
        test::call_service(&app, req).await;
    }
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/samples/batch/{}", batch))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Vec<SampleResponse> = test::read_body_json(resp).await;
    assert!(body.len() >= 2);
}

#[actix_web::test]
async fn test_scan_inventory() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::post()
        .uri("/api/v1/inventory/scan")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let _body: InventoryScanResponse = test::read_body_json(resp).await;
    // count is always non-negative (usize)
}

#[actix_web::test]
async fn test_get_inventory_report() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/inventory/report")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_read_temperature() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::post()
        .uri("/api/v1/temperature/read")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let _body: TemperatureResponse = test::read_body_json(resp).await;
    // violations is always non-negative (usize)
}

#[actix_web::test]
async fn test_get_temperature_statistics() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/temperature/statistics")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_audit_events() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a sample to generate audit events
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-AUDIT".to_string(),
        batch_number: "BATCH-AUDIT".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    test::call_service(&app, req).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/audit/events")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: AuditQueryResponse = test::read_body_json(resp).await;
    assert!(body.total > 0);
}

#[actix_web::test]
async fn test_get_audit_statistics() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/audit/statistics")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_statistics() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/v1/statistics")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let _body: StatisticsResponse = test::read_body_json(resp).await;
    // samples is always non-negative (usize)
}

#[actix_web::test]
async fn test_invalid_status_update() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a sample
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-INVALID".to_string(),
        batch_number: "BATCH-INVALID".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    test::call_service(&app, req).await;
    
    // Try to update with invalid status
    let update_req = UpdateSampleStatusRequest {
        status: "InvalidStatus".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::put()
        .uri("/api/v1/samples/API-TEST-INVALID/status")
        .set_json(&update_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_audit_events_by_sample() {
    let app_state = create_app_state();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create a sample
    let create_req = CreateSampleRequest {
        sample_id: "API-TEST-AUDIT-SAMPLE".to_string(),
        batch_number: "BATCH-AUDIT".to_string(),
        production_date: Utc::now(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
        temperature_range: Some((2.0, 8.0)),
        storage_conditions: "Refrigerated".to_string(),
        manufacturer: "Test".to_string(),
        product_line: "Test".to_string(),
        location: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/samples")
        .set_json(&create_req)
        .to_request();
    test::call_service(&app, req).await;
    
    // Query audit events by sample
    let req = test::TestRequest::get()
        .uri("/api/v1/audit/events?sample_id=API-TEST-AUDIT-SAMPLE")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: AuditQueryResponse = test::read_body_json(resp).await;
    assert!(body.total > 0);
}

