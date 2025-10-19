mod audio_analyzer;

use actix_web::{web, App, HttpServer, HttpResponse, error};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use audio_analyzer::AudioAnalyzer;
use once_cell::sync::Lazy;
use base64::{Engine, engine::general_purpose::STANDARD};

/// Single note detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedNote {
    pub note: String,
    pub confidence: f32,
    /// Power/intensity of the note (0.0-1.0, where 1.0 is maximum loudness)
    pub intensity: f32,
}

/// Complete analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub notes: Vec<DetectedNote>,
    pub sample_rate: u32,
    pub samples_analyzed: usize,
    pub timestamp: f64,
}

#[derive(Debug, Deserialize)]
pub struct AudioData {
    pub audio_data: String,  // Direct String for base64
    pub sample_rate: u32,
}

impl AudioData {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        STANDARD
            .decode(&self.audio_data)
            .map_err(|e| format!("Base64 decode error: {}", e))
    }
}

// Global audio analyzer (lazy-initialized to avoid expensive setup)
static ANALYZER: Lazy<AudioAnalyzer> = Lazy::new(AudioAnalyzer::new);

// In-memory storage for analysis results
pub struct AppState {
    last_result: Mutex<Option<AnalysisResult>>,
}

/// Convert note name to approximate frequency (for scoring)
fn note_to_frequency(note_name: &str) -> f32 {
    // Simple mapping: extract note and octave
    // E.g., "A4" -> 440 Hz, "C4" -> 261.63 Hz
    match note_name {
        n if n.starts_with("C1") => 32.7,
        n if n.starts_with("C2") => 65.4,
        n if n.starts_with("C3") => 130.8,
        n if n.starts_with("D1") => 36.7,
        n if n.starts_with("D2") => 73.4,
        n if n.starts_with("D3") => 146.8,
        n if n.starts_with("E1") => 41.2,
        n if n.starts_with("E2") => 82.4,
        n if n.starts_with("E3") => 164.8,
        n if n.starts_with("F1") => 43.7,
        n if n.starts_with("F2") => 87.3,
        n if n.starts_with("F3") => 174.6,
        n if n.starts_with("G1") => 49.0,
        n if n.starts_with("G2") => 98.0,
        n if n.starts_with("G3") => 196.0,
        n if n.starts_with("A1") => 55.0,
        n if n.starts_with("A2") => 110.0,
        n if n.starts_with("A3") => 220.0,
        n if n.starts_with("B1") => 61.7,
        n if n.starts_with("B2") => 123.5,
        n if n.starts_with("B3") => 247.0,
        _ => 440.0,  // Default to A4
    }
}

/// Bonus for low frequencies (bass notes)
/// Lower frequencies get MUCH higher bonus to compensate for lower natural amplitude
/// Uses inverse log scale: lower freq = exponentially higher score
fn low_frequency_bonus(freq: f32) -> f32 {
    // Logarithmic inverse: 50 Hz -> 1.0, 100 Hz -> 0.85, 200 Hz -> 0.65, 400 Hz -> 0.40, 800 Hz -> 0.0
    // Formula: 1.0 / (1.0 + (freq / 100.0).log2())
    let bonus = 1.0 / (1.0 + (freq / 50.0).log2());
    bonus.max(0.0).min(1.0)
}

/// Weight confidence scores
fn confidence_weight(confidence: f32) -> f32 {
    // Higher confidence = better score
    confidence.clamp(0.0, 1.0)
}

/// Analyze audio endpoint - processes raw audio and returns detected notes
async fn analyze_audio(
    _state: web::Data<AppState>,
    audio: web::Json<AudioData>,
) -> HttpResponse {
    // Measure from START of function (JSON already deserialized by framework)
    let request_start = std::time::Instant::now();
    
    log::debug!("Received request: sample_rate={}", audio.sample_rate);
    
    if audio.sample_rate == 0 {
        log::error!("Invalid sample_rate: 0");
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "sample_rate must be greater than 0"})
        );
    }

    // Decode audio data (base64 string)
    let audio_bytes = match audio.to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": format!("Audio decode error: {}", e)})
            );
        }
    };

    // Track timing for analysis
    let mut analysis_ms = 0u128;
    let mut convert_us = 0u128;
    let audio_len = audio_bytes.len();

    // Allow empty audio_data - just return empty notes (for UI updates)
    let result = if audio_bytes.is_empty() {
        AnalysisResult {
            notes: Vec::new(),
            sample_rate: audio.sample_rate,
            samples_analyzed: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    } else {
        let pre_analysis = std::time::Instant::now();
        
        // Analyze the audio
        let notes_raw = ANALYZER.analyze_raw_bytes(&audio_bytes, audio.sample_rate);
        
        analysis_ms = pre_analysis.elapsed().as_millis();
        
        // Convert to result format with confidence filter (>= 10%)
        // Keep top 3 notes with smart scoring: prefer lower frequencies (bass voices)
        let pre_convert = std::time::Instant::now();
        let mut notes: Vec<DetectedNote> = notes_raw
            .into_iter()
            .filter(|(_, confidence, _)| *confidence >= 0.10)  // Only include notes with confidence >= 10%
            .map(|(note, confidence, intensity)| DetectedNote { note, confidence, intensity })
            .collect();
        
        // IMPROVED: Sort with bias towards lower frequencies (bass notes)
        // Bass singers naturally have weaker spectral power, so we must not rely on intensity alone
        notes.sort_by(|a, b| {
            // Extract frequency from note name (crude but effective)
            // C1=32.7Hz, C2=65.4Hz, ..., C3=130.8Hz, C4=261.6Hz, C5=523.3Hz, C6=1046.5Hz
            let freq_a = note_to_frequency(&a.note);
            let freq_b = note_to_frequency(&b.note);
            
            // Scoring: HEAVILY favor lower frequencies (bass priority!)
            // Weights: bass_bonus(0.5) > confidence(0.4) > intensity(0.1)
            let score_a = (low_frequency_bonus(freq_a) * 0.7) + 
                         (confidence_weight(a.confidence) * 0.2) + 
                         (a.intensity * 0.1);
            let score_b = (low_frequency_bonus(freq_b) * 0.7) + 
                         (confidence_weight(b.confidence) * 0.2) + 
                         (b.intensity * 0.1);
            
            score_b.partial_cmp(&score_a).unwrap()
        });
        notes.truncate(3);
        
        convert_us = pre_convert.elapsed().as_micros();

        AnalysisResult {
            notes,
            sample_rate: audio.sample_rate,
            samples_analyzed: audio_bytes.len() / 2, // 16-bit samples = 2 bytes each
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    };

    let pre_serialize = std::time::Instant::now();
    let response = HttpResponse::Ok().json(&result);
    let serialize_ms = pre_serialize.elapsed().as_millis();
    
    let total_ms = request_start.elapsed().as_millis();
    
    // Log notes with confidence
    if !result.notes.is_empty() {
        let notes_str = result.notes
            .iter()
            .map(|n| format!("{}({}%, {:.0})", n.note, (n.confidence * 100.0) as u32, n.intensity * 100.0))
            .collect::<Vec<_>>()
            .join(", ");
        log::info!("REQUEST: bytes={}, analysis={}ms, convert={}us, serialize={}ms, TOTAL={}ms, NOTES: [{}]", 
            audio_len, 
            analysis_ms,
            convert_us,
            serialize_ms,
            total_ms,
            notes_str
        );
    } else {
        log::info!("REQUEST: bytes={}, analysis={}ms, convert={}us, serialize={}ms, TOTAL={}ms, NOTES: (none)", 
            audio_len, 
            analysis_ms,
            convert_us,
            serialize_ms,
            total_ms
        );
    }

    response
}

/// Get last analysis result
async fn get_last_result(state: web::Data<AppState>) -> HttpResponse {
    if let Ok(last_result) = state.last_result.lock() {
        match &*last_result {
            Some(result) => HttpResponse::Ok().json(result.clone()),
            None => HttpResponse::NoContent().finish(),
        }
    } else {
        HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to access stored result"})
        )
    }
}

/// Health check endpoint
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "version": "0.2.0-fft"}))
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
                    let err_msg = format!("{}", err);
                    log::error!("JSON parsing error: {}", err_msg);
                    actix_web::error::InternalError::from_response(
                        err,
                        HttpResponse::BadRequest().json(
                            serde_json::json!({"error": format!("JSON parse error: {}", err_msg)})
                        )
                    ).into()
                })
            )
            // DISABLED: Logger middleware was causing 2-second delay!
            // .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health))
            .route("/analyze", web::post().to(analyze_audio))
            .route("/last-result", web::get().to(get_last_result))
    })
    .workers(8)  // Increase worker threads for parallel processing
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
