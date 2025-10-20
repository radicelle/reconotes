use serde::{Deserialize, Serialize};
use base64::{Engine, engine::general_purpose::STANDARD};

/// Voice profile for filtering notes by typical vocal range
#[allow(clippy::trivially_copy_pass_by_ref, clippy::doc_markdown)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Returns (`min_freq_hz`, `max_freq_hz`)
    pub const fn freq_range(self) -> Option<(f32, f32)> {
        match self {
            Self::NoProfile => None,
            Self::Soprano => Some((261.63, 1046.50)),   // C4-C6
            Self::Mezzo => Some((220.00, 880.00)),      // A3-A5
            Self::Alto => Some((174.61, 698.46)),       // F3-F5
            Self::Tenor => Some((130.81, 523.25)),      // C3-C5
            Self::Baritone => Some((110.00, 440.00)),   // A2-A4
            Self::Bass => Some((65.41, 261.63)),        // C2-C4
        }
    }

    /// Get all available profiles as strings for UI selection
    pub const fn all_profiles() -> &'static [&'static str] {
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

    /// Parse string to `VoiceProfile`
    pub fn from_str(s: &str) -> Self {
        match s {
            "soprano" => Self::Soprano,
            "mezzo" => Self::Mezzo,
            "alto" => Self::Alto,
            "tenor" => Self::Tenor,
            "baritone" => Self::Baritone,
            "bass" => Self::Bass,
            _ => Self::NoProfile,
        }
    }

    /// Get string representation
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoProfile => "no_profile",
            Self::Soprano => "soprano",
            Self::Mezzo => "mezzo",
            Self::Alto => "alto",
            Self::Tenor => "tenor",
            Self::Baritone => "baritone",
            Self::Bass => "bass",
        }
    }
}

impl Default for VoiceProfile {
    fn default() -> Self {
        Self::NoProfile
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
    /// Decode base64-encoded audio data to bytes
    /// 
    /// # Errors
    /// Returns an error if the base64 decoding fails
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        STANDARD
            .decode(&self.audio_data)
            .map_err(|e| format!("Base64 decode error: {e}"))
    }

    /// Get the voice profile from the optional profile string
    #[must_use] pub fn get_profile(&self) -> VoiceProfile {
        self.profile.as_ref().map_or(VoiceProfile::NoProfile, |profile_str| VoiceProfile::from_str(profile_str))
    }
}
