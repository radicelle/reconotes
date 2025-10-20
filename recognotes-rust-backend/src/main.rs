mod audio_analyzer;
mod endpoints;
mod models;
mod utils;

use actix_web::{web, App, HttpServer, HttpResponse, error};
use std::sync::Mutex;
use audio_analyzer::AudioAnalyzer;

// Export for use in endpoints module
pub use models::{AnalysisResult, AudioData, DetectedNote};

// Global audio analyzer (lazy-initialized to avoid expensive setup)
pub static ANALYZER: std::sync::LazyLock<AudioAnalyzer> = std::sync::LazyLock::new(AudioAnalyzer::new);

// In-memory storage for analysis results
pub struct AppState {
    pub last_result: Mutex<Option<AnalysisResult>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app_state = web::Data::new(AppState {
        last_result: Mutex::new(None),
    });

    log::info!("Starting RecogNotes Rust Backend on http://127.0.0.1:5000");
    log::info!("Audio analysis with FFT-based pitch detection enabled");
    log::info!("Max payload size: 16MB, Workers: 8, No request timeout");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Increase JSON payload limit to 16MB to handle larger audio chunks
            .app_data(web::JsonConfig::default()
                .limit(16 * 1024 * 1024) // 16MB limit
                .error_handler(|err, _req| {
                    let err_msg = format!("{err}");
                    log::error!("JSON parsing error: {err_msg}");
                    error::InternalError::from_response(
                        err,
                        HttpResponse::BadRequest().json(
                            serde_json::json!({"error": format!("JSON parse error: {}", err_msg)})
                        )
                    ).into()
                })
            )
            // DISABLED: Logger middleware was causing 2-second delay!
            // .wrap(middleware::Logger::default())
            .route("/health", web::get().to(endpoints::health))
            .route("/analyze", web::post().to(endpoints::analyze_audio))
            .route("/last-result", web::get().to(endpoints::get_last_result))
    })
    .workers(8)  // Increase worker threads for parallel processing
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
