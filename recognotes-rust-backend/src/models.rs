use serde::{Deserialize, Serialize};
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
