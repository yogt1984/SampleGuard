use crate::api::handlers::*;
use actix_web::web;

/// Configure all API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/statistics", web::get().to(get_statistics))
            .service(
                web::scope("/samples")
                    .route("", web::get().to(get_samples))
                    .route("", web::post().to(create_sample))
                    .route("/{sample_id}", web::get().to(get_sample))
                    .route("/{sample_id}/status", web::put().to(update_sample_status))
                    .route("/{sample_id}", web::delete().to(delete_sample))
                    .route("/batch/{batch_number}", web::get().to(get_samples_by_batch)),
            )
            .service(
                web::scope("/inventory")
                    .route("/scan", web::post().to(scan_inventory))
                    .route("/report", web::get().to(get_inventory_report)),
            )
            .service(
                web::scope("/temperature")
                    .route("/read", web::post().to(read_temperature))
                    .route("/statistics", web::get().to(get_temperature_statistics)),
            )
            .service(
                web::scope("/audit")
                    .route("/events", web::get().to(get_audit_events))
                    .route("/statistics", web::get().to(get_audit_statistics)),
            ),
    );
}

