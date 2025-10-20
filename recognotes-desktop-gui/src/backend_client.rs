use crate::DetectedNote;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    /// Base64-encoded audio data (faster than Vec<u8> JSON encoding)
    pub audio_data: String,
    pub sample_rate: u32,
    /// Optional voice profile for filtering notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub notes: Vec<DetectedNote>,
    pub sample_rate: u32,
    pub samples_analyzed: usize,
    pub timestamp: f64,
}

/// Send audio data to the backend for analysis with timeout
/// Uses base64 encoding for optimal performance (~1-5ms instead of slow JSON arrays)
pub async fn analyze_audio(
    backend_url: &str,
    audio_data: Vec<u8>,
    sample_rate: u32,
    profile: Option<String>,
) -> Result<Vec<DetectedNote>, String> {
    let url = format!("{backend_url}/analyze");
    let start = Instant::now();
    let data_size = audio_data.len();
    let profile_str = profile.as_deref().unwrap_or("no_profile").to_string();
    
    // Encode audio as base64 (much faster than JSON array encoding)
    let audio_b64 = STANDARD.encode(&audio_data);
    
    let request = AnalyzeRequest {
        audio_data: audio_b64.clone(),
        sample_rate,
        profile,
    };

    // Create new client for each request (reqwest handles connection pooling internally)
    let client = reqwest::Client::new();
    
    log::debug!(
        "Sending to backend: {} bytes audio (base64), {} Hz sample rate, profile: {}, payload size: {}B",
        data_size,
        sample_rate,
        profile_str,
        audio_b64.len()
    );
    
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),  // 5 second timeout
        client
            .post(&url)
            .json(&request)
            .send()
    )
    .await
    .map_err(|_| "Backend request timeout (5s)".to_string())?
    .map_err(|e| format!("Failed to send request: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Backend returned status: {}", response.status()));
    }

    let analyze_response: AnalyzeResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let elapsed = start.elapsed().as_millis();
    log::debug!(
        "Backend analysis: {} notes, {} samples in {:.0}ms ({}KB sent, base64 encoded)",
        analyze_response.notes.len(),
        analyze_response.samples_analyzed,
        elapsed,
        data_size / 1024
    );

    Ok(analyze_response.notes)
}

/// Check if backend is healthy
/// Uses fast timeout to fail quickly if backend is down
pub async fn check_health(backend_url: &str) -> Result<(), String> {
    let url = format!("{backend_url}/health");
    
    let client = reqwest::Client::new();
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(1),  // Quick timeout for health checks
        client.get(&url).send()
    )
    .await
    .map_err(|_| "Backend health check timeout".to_string())?
    .map_err(|e| format!("Failed to connect to backend: {e}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Backend health check failed: {}", response.status()))
    }

}
