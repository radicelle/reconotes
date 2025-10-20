/// Convert note name to approximate frequency (for scoring)
pub fn note_to_frequency(note_name: &str) -> f32 {
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
pub fn low_frequency_bonus(freq: f32) -> f32 {
    // Logarithmic inverse: 50 Hz -> 1.0, 100 Hz -> 0.85, 200 Hz -> 0.65, 400 Hz -> 0.40, 800 Hz -> 0.0
    // Formula: 1.0 / (1.0 + (freq / 100.0).log2())
    let bonus = 1.0 / (1.0 + (freq / 50.0).log2());
    bonus.clamp(0.0, 1.0)
}

/// Weight confidence scores
pub const fn confidence_weight(confidence: f32) -> f32 {
    // Higher confidence = better score
    confidence.clamp(0.0, 1.0)
}
