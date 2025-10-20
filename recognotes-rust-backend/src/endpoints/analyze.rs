use actix_web::{web, HttpResponse};
use serde_json::json;
use std::time::SystemTime;

use crate::{
    models::{AnalysisResult, AudioData, DetectedNote},
    utils::{confidence_weight, low_frequency_bonus, note_to_frequency},
    AppState, ANALYZER,
};

/// Analyze audio endpoint - processes raw audio and returns detected notes
pub async fn analyze_audio(
    _state: web::Data<AppState>,
    audio: web::Json<AudioData>,
) -> HttpResponse {
    // Measure from START of function (JSON already deserialized by framework)
    let request_start = std::time::Instant::now();

    log::debug!("Received request: sample_rate={}", audio.sample_rate);

    if audio.sample_rate == 0 {
        log::error!("Invalid sample_rate: 0");
        return HttpResponse::BadRequest().json(
            json!({"error": "sample_rate must be greater than 0"})
        );
    }

    // Decode audio data (base64 string)
    let audio_bytes = match audio.to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::BadRequest().json(
                json!({"error": format!("Audio decode error: {}", e)})
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
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    } else {
        let pre_analysis = std::time::Instant::now();

        // Analyze the audio (FFT processing is internally optimized)
        let notes_raw = ANALYZER.analyze_raw_bytes(&audio_bytes, audio.sample_rate);

        analysis_ms = pre_analysis.elapsed().as_millis();

        // Convert to result format with confidence filter (>= 10%)
        // Keep top 3 notes with smart scoring: prefer lower frequencies (bass voices)
        let pre_convert = std::time::Instant::now();
        let notes: Vec<DetectedNote> = notes_raw
            .into_iter()
            .filter(|(_, confidence, _)| *confidence >= 0.10)
            .map(|(note, confidence, intensity)| DetectedNote { note, confidence, intensity })
            .collect();

        // OPTIMIZED: Pre-compute scores with frequency lookup cache
        // This avoids redundant note_to_frequency() and bonus calculations
        let mut notes_with_scores: Vec<(DetectedNote, f32)> = notes
            .into_iter()
            .map(|note| {
                let freq = note_to_frequency(&note.note);
                let score = (low_frequency_bonus(freq) * 0.7)
                    + (confidence_weight(note.confidence) * 0.2)
                    + (note.intensity * 0.1);
                (note, score)
            })
            .collect();

        // Sort once by pre-computed scores
        notes_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Extract top 3 notes
        let notes: Vec<DetectedNote> = notes_with_scores
            .into_iter()
            .take(3)
            .map(|(note, _)| note)
            .collect();

        convert_us = pre_convert.elapsed().as_micros();

        AnalysisResult {
            notes,
            sample_rate: audio.sample_rate,
            samples_analyzed: audio_bytes.len() / 2, // 16-bit samples = 2 bytes each
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
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
        let notes_str = result
            .notes
            .iter()
            .map(|n| format!("{}({}%, {:.0})", n.note, (n.confidence * 100.0) as u32, n.intensity * 100.0))
            .collect::<Vec<_>>()
            .join(", ");
        log::info!(
            "REQUEST: bytes={}, analysis={}ms, convert={}us, serialize={}ms, TOTAL={}ms, NOTES: [{}]",
            audio_len, analysis_ms, convert_us, serialize_ms, total_ms, notes_str
        );
    } else {
        log::info!(
            "REQUEST: bytes={}, analysis={}ms, convert={}us, serialize={}ms, TOTAL={}ms, NOTES: (none)",
            audio_len, analysis_ms, convert_us, serialize_ms, total_ms
        );
    }

    response
}
