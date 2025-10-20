use actix_web::HttpResponse;
use serde_json::json;
use crate::AppState;
use actix_web::web;

/// Get last analysis result
pub async fn get_last_result(state: web::Data<AppState>) -> HttpResponse {
    state.last_result.lock().map_or_else(
        |_| HttpResponse::InternalServerError().json(
            json!({"error": "Failed to access stored result"})
        ),
        |last_result| {
            last_result.as_ref().map_or_else(
                || HttpResponse::NoContent().finish(),
                |result| HttpResponse::Ok().json(result.clone())
            )
        }
    )
}
