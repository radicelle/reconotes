use serde::{Deserialize, Serialize};
use base64::{Engine, engine::general_purpose::STANDARD};

/// Voice profile for filtering notes by typical vocal range
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VoiceProfile {
    #[serde(rename = "no_profile")]
    NoProfile,
    #[serde(rename = "soprano")]
    Soprano,     // C4-C6 (261-1047 Hz) - highest female voice
    #[serde(rename = "mezzo")]
    Mezzo,       // A3-A5 (220-880 Hz) - middle female voice
    #[serde(rename = "alto")]
    Alto,        // F3-F5 (174-698 Hz) - lower female/countertenor voice
    #[serde(rename = "tenor")]
    Tenor,       // C3-C5 (131-523 Hz) - highest male voice
    #[serde(rename = "baritone")]
    Baritone,    // A2-A4 (110-440 Hz) - middle male voice
    #[serde(rename = "bass")]
    Bass,        // C2-C4 (65-261 Hz) - lowest male voice
}

impl VoiceProfile {
    /// Get the frequency range for this voice profile
    /// Returns (min_freq_hz, max_freq_hz)
    pub fn freq_range(&self) -> Option<(f32, f32)> {
        match self {
            VoiceProfile::NoProfile => None,
            VoiceProfile::Soprano => Some((261.63, 1046.50)),   // C4-C6
            VoiceProfile::Mezzo => Some((220.00, 880.00)),      // A3-A5
            VoiceProfile::Alto => Some((174.61, 698.46)),       // F3-F5
            VoiceProfile::Tenor => Some((130.81, 523.25)),      // C3-C5
            VoiceProfile::Baritone => Some((110.00, 440.00)),   // A2-A4
            VoiceProfile::Bass => Some((65.41, 261.63)),        // C2-C4
        }
    }

    /// Get all available profiles as strings for UI selection
    pub fn all_profiles() -> &'static [&'static str] {
        &[
            "no_profile",
            "soprano",
            "mezzo",
            "alto",
            "tenor",
            "baritone",
            "bass",
        ]
    }

    /// Parse string to VoiceProfile
    pub fn from_str(s: &str) -> Self {
        match s {
            "soprano" => VoiceProfile::Soprano,
            "mezzo" => VoiceProfile::Mezzo,
            "alto" => VoiceProfile::Alto,
            "tenor" => VoiceProfile::Tenor,
            "baritone" => VoiceProfile::Baritone,
            "bass" => VoiceProfile::Bass,
            _ => VoiceProfile::NoProfile,
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            VoiceProfile::NoProfile => "no_profile",
            VoiceProfile::Soprano => "soprano",
            VoiceProfile::Mezzo => "mezzo",
            VoiceProfile::Alto => "alto",
            VoiceProfile::Tenor => "tenor",
            VoiceProfile::Baritone => "baritone",
            VoiceProfile::Bass => "bass",
        }
    }
}

impl Default for VoiceProfile {
    fn default() -> Self {
        VoiceProfile::NoProfile
    }
}

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
    #[serde(default)]
    pub profile: Option<String>,  // Voice profile for filtering notes
}

impl AudioData {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        STANDARD
            .decode(&self.audio_data)
            .map_err(|e| format!("Base64 decode error: {}", e))
    }

    /// Get the voice profile from the optional profile string
    pub fn get_profile(&self) -> VoiceProfile {
        match &self.profile {
            Some(profile_str) => VoiceProfile::from_str(profile_str),
            None => VoiceProfile::NoProfile,
        }
    }
}
