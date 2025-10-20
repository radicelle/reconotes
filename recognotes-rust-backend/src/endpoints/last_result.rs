use actix_web::HttpResponse;
use serde_json::json;
use crate::AppState;
use actix_web::web;

/// Get last analysis result
pub async fn get_last_result(state: web::Data<AppState>) -> HttpResponse {
    if let Ok(last_result) = state.last_result.lock() {
        match &*last_result {
            Some(result) => HttpResponse::Ok().json(result.clone()),
            None => HttpResponse::NoContent().finish(),
        }
    } else {
        HttpResponse::InternalServerError().json(
            json!({"error": "Failed to access stored result"})
        )
    }
}
