use actix_web::HttpResponse;
use serde_json::json;

/// Health check endpoint
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "ok", "version": "0.2.0-fft"}))
}
