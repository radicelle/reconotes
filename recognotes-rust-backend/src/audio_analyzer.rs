use std::f32::consts::PI;
use rustfft::FftPlanner;
use num_complex::Complex;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Constants for note-to-frequency mapping
const KNOWN_NOTE_FREQUENCY: f32 = 440.0; // A4 = 440 Hz
// Natural notes only (no sharps/flats) - focuses on standard musical notes
const NOTE_NAMES: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];
// IMPROVED: Extended range to include low bass notes
// Covers: Bass (C1-E2), Baritone (A1-G3), Tenor (C3-C5), Countertenor/Alto (F3-F5), Soprano (C4-C6)
const MIN_OCTAVE: i32 = 1;  // C1 = 32.7 Hz (very low bass)
const MAX_OCTAVE: i32 = 7;  // C7 = 2093 Hz (high soprano)

/// Global FFT planner - reused across all requests
/// Creating a new FftPlanner is very expensive, so we share one globally
static FFT_PLANNER: Lazy<Mutex<FftPlanner<f32>>> = Lazy::new(|| Mutex::new(FftPlanner::new()));

/// Pre-computed lookup table for frequency-to-note conversion
/// This avoids expensive log calculations on every call
pub struct FrequencyToNoteLookup {
    table: Vec<(String, f32)>, // (note_name, base_frequency)
}

impl FrequencyToNoteLookup {
    /// Create a lookup table for note-to-frequency mapping
    /// Limited to realistic human vocal range: C2 (65.4 Hz) to C6 (1046.5 Hz)
    /// Uses only natural notes (no sharps/flats) to focus on standard musical notes
    /// Covers all professional voice types: Bass, Baritone, Tenor, Countertenor, Contralto, Mezzo-Soprano, Soprano
    pub fn new() -> Self {
        let mut table = Vec::new();
        
        // Generate natural notes (C, D, E, F, G, A, B) from octave 2 to 6
        for octave in MIN_OCTAVE..=MAX_OCTAVE {
            for note_name in NOTE_NAMES.iter() {
                // Map natural notes to MIDI semitone positions
                // C=0, D=2, E=4, F=5, G=7, A=9, B=11 (skips sharps/flats)
                let note_semitones = match *note_name {
                    "C" => 0,
                    "D" => 2,
                    "E" => 4,
                    "F" => 5,
                    "G" => 7,
                    "A" => 9,
                    "B" => 11,
                    _ => continue,
                };
                
                // Calculate MIDI note number
                let note_num = (octave * 12) + note_semitones + 12; // C0 is MIDI 12
                let semitones_from_a4 = note_num - 69; // A4 is MIDI 69
                let frequency = KNOWN_NOTE_FREQUENCY * 2.0_f32.powf(semitones_from_a4 as f32 / 12.0);
                
                let note_full_name = format!("{}{}", note_name, octave);
                table.push((note_full_name, frequency));
            }
        }
        
        // Sort by frequency for binary search
        table.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        Self { table }
    }
    
    /// Find the closest note for a given frequency
    /// Returns (note_name, confidence)
    pub fn find_closest_note(&self, frequency: f32) -> Option<(String, f32)> {
        if frequency <= 0.0 || frequency > 20000.0 {
            return None;
        }
        
        // Binary search to find closest frequency
        let mut left = 0;
        let mut right = self.table.len() - 1;
        
        while left < right {
            let mid = (left + right) / 2;
            if self.table[mid].1 < frequency {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        
        // Check both neighbors to find closest
        let mut closest_idx = left;
        if left > 0 && (frequency - self.table[left - 1].1).abs() < (frequency - self.table[left].1).abs() {
            closest_idx = left - 1;
        }
        
        let (note_name, base_freq) = &self.table[closest_idx];
        
        // Calculate confidence (how close the frequency matches)
        // Within 50 cents (0.5 semitone) is considered a good match
        let cents_diff = 1200.0 * (frequency / base_freq).log2().abs();
        let confidence = (1.0 - (cents_diff / 100.0).min(1.0)).max(0.0);
        
        Some((note_name.clone(), confidence))
    }
}

/// Analyze audio buffer and detect dominant frequency
pub struct AudioAnalyzer {
    lookup: FrequencyToNoteLookup,
}

impl AudioAnalyzer {
    pub fn new() -> Self {
        Self {
            lookup: FrequencyToNoteLookup::new(),
        }
    }
    
    /// Compute FFT and return Power Spectral Density
    /// Uses global FFT planner to avoid expensive re-planning on every call
    fn compute_fft(&self, signal: &[f32], _sample_rate: u32) -> Vec<f32> {
        let signal_len = signal.len();
        let lock_start = std::time::Instant::now();
        
        // Get the global FFT planner (created once, reused for all requests)
        let mut planner = FFT_PLANNER.lock().unwrap();
        let lock_time = lock_start.elapsed().as_micros();
        
        let plan_start = std::time::Instant::now();
        let fft = planner.plan_fft_forward(signal_len);
        let plan_time = plan_start.elapsed().as_micros();
        
        // Convert input to complex numbers
        let convert_start = std::time::Instant::now();
        let mut buffer: Vec<Complex<f32>> = signal
            .iter()
            .map(|&s| Complex { re: s, im: 0.0 })
            .collect();
        let convert_time = convert_start.elapsed().as_micros();
        
        // Compute FFT
        let process_start = std::time::Instant::now();
        fft.process(&mut buffer);
        let process_time = process_start.elapsed().as_micros();
        
        // Compute Power Spectral Density
        let psd_start = std::time::Instant::now();
        let psd: Vec<f32> = buffer
            .iter()
            .map(|c| (c.norm().powi(2) / signal_len as f32).sqrt())
            .collect();
        let psd_time = psd_start.elapsed().as_micros();
        
        log::debug!("compute_fft({}): lock={}us, plan={}us, convert={}us, process={}us, psd={}us", 
            signal_len, lock_time, plan_time, convert_time, process_time, psd_time);
        
        psd
    }
    
    /// Find all significant peaks in the FFT spectrum, with harmonic suppression to find the fundamental.
    fn find_all_peaks(&self, psd: &[f32], sample_rate: u32, signal_len: usize) -> Vec<(f32, f32)> {
        if psd.is_empty() {
            return Vec::new();
        }

        let mut peaks = Vec::new();
        let mut mutable_psd = psd.to_vec(); // Make a mutable copy of the power spectrum

        // Find global maximum for threshold calculation
        let max_power = psd[1..psd.len() / 2].iter().cloned().fold(0.0_f32, f32::max);
        
        // IMPROVED: Lower threshold (10% instead of 20%) to catch even weaker fundamental frequencies
        // Notes: lower frequencies often have less energy than their harmonics
        let threshold = (max_power * 0.10).max(0.05);

        // --- Iterative Harmonic Suppression ---
        // This loop finds the strongest peak, assumes it's a fundamental, removes its harmonics,
        // and then repeats. This helps to isolate true fundamental frequencies from their overtones.
        for _ in 0..10 { // Limit to finding the 10 strongest fundamental peaks
            let spectrum = &mutable_psd[1..mutable_psd.len() / 2]; // Use the mutable spectrum
            
            let max_idx_opt = spectrum.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i + 1);

            if let Some(max_idx) = max_idx_opt {
                let power = spectrum[max_idx - 1];
                if power < threshold {
                    break; // Stop if the strongest remaining peak is below the noise threshold
                }

                let frequency = (max_idx as f32) * (sample_rate as f32) / (signal_len as f32);
                
                // Add the found fundamental peak to our list
                peaks.push((frequency, power.min(1.0)));

                // --- Suppress the found peak and its harmonics ---
                let freq_resolution = sample_rate as f32 / signal_len as f32;
                let peak_width_bins = (frequency * 0.03 / freq_resolution).ceil() as usize; // Suppress in a small window around the peak

                // Suppress the fundamental peak itself to prevent re-detection
                let start_bin = (max_idx - peak_width_bins).max(1);
                let end_bin = (max_idx + peak_width_bins).min(mutable_psd.len() / 2);
                for i in start_bin..=end_bin {
                    mutable_psd[i] = 0.0;
                }

                // Suppress harmonics (2x, 3x, 4x, 5x, 6x)
                for n in 2..=6 {
                    let harmonic_freq = frequency * n as f32;
                    let harmonic_idx = (harmonic_freq / freq_resolution) as usize;
                    
                    if harmonic_idx < mutable_psd.len() / 2 {
                        let h_start_bin = (harmonic_idx - peak_width_bins).max(1);
                        let h_end_bin = (harmonic_idx + peak_width_bins).min(mutable_psd.len() / 2);
                        for i in h_start_bin..=h_end_bin {
                            mutable_psd[i] = 0.0; // Zero out the power of the harmonic
                        }
                    }
                }
            } else {
                break; // No more peaks to find
            }
        }
        
        // Sort by power (descending) as the primary result
        peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Debug logging for detected peaks
        log::debug!("FFT Peaks (Harmonic Suppression): max_power={:.3}, threshold={:.3}, fundamentals_found={}", 
            max_power, threshold, peaks.len());
        for (i, (freq, power)) in peaks.iter().take(10).enumerate() {
            log::debug!("  Fundamental Peak {}: {:.2} Hz @ power={:.3}", i + 1, freq, power);
        }
        
        peaks
    }
    
    /// Clean FFT output to find primary frequency (returns strongest peak only)
    fn find_primary_frequency(&self, psd: &[f32], sample_rate: u32, signal_len: usize) -> Option<(f32, f32)> {
        if psd.is_empty() {
            return None;
        }
        
        // Find the index of maximum power (excluding DC component at index 0)
        let max_idx = psd[1..psd.len() / 2]  // Only look at positive frequencies
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i + 1)?;
        
        let max_power = psd[max_idx];
        
        // Filter out very weak signals (noise floor)
        if max_power < 0.1 {
            return None;
        }
        
        // Convert index to frequency
        let frequency = (max_idx as f32) * (sample_rate as f32) / (signal_len as f32);
        
        // Return (frequency, power_as_confidence)
        Some((frequency, max_power.min(1.0)))
    }
    
    /// Analyze audio chunk and return detected notes with confidence and intensity
    /// Returns multiple notes if multiple strong peaks are detected
    pub fn analyze_chunk_multi(&self, audio_data: &[f32], sample_rate: u32) -> Vec<(String, f32, f32)> {
        if audio_data.is_empty() {
            return Vec::new();
        }
        
        // Apply Hann window to reduce spectral leakage
        let windowed = self.apply_hann_window(audio_data);
        
        // Compute FFT
        let psd = self.compute_fft(&windowed, sample_rate);
        
        // Find all peaks in the spectrum
        let peaks = self.find_all_peaks(&psd, sample_rate, audio_data.len());
        
        // Convert each peak to a note (limit to top 10 peaks to avoid noise)
        let mut notes = Vec::new();
        for (frequency, power) in peaks.iter().take(10) {
            if let Some((note_name, note_confidence)) = self.lookup.find_closest_note(*frequency) {
                // Use frequency-match confidence directly (ignore power-based confidence)
                // Power-based confidence can be artificially low due to FFT bin resolution
                // The frequency match confidence is more reliable for determining if a peak is a real note
                // power is already normalized to [0.0, 1.0] from find_all_peaks
                notes.push((note_name, note_confidence, *power));
            }
        }
        
        notes
    }
    
    /// Analyze audio chunk and return detected notes with confidence
    pub fn analyze_chunk(&self, audio_data: &[f32], sample_rate: u32) -> Option<(String, f32)> {
        if audio_data.is_empty() {
            return None;
        }
        
        let start = std::time::Instant::now();
        
        // Apply Hann window to reduce spectral leakage
        let window_start = std::time::Instant::now();
        let windowed = self.apply_hann_window(audio_data);
        let window_time = window_start.elapsed().as_millis();
        
        // Compute FFT
        let fft_start = std::time::Instant::now();
        let psd = self.compute_fft(&windowed, sample_rate);
        let fft_time = fft_start.elapsed().as_millis();
        
        // Find primary frequency
        let find_start = std::time::Instant::now();
        let (frequency, _power_confidence) = self.find_primary_frequency(&psd, sample_rate, audio_data.len())?;
        let find_time = find_start.elapsed().as_millis();
        
        // Convert frequency to note
        let lookup_start = std::time::Instant::now();
        let (note_name, note_confidence) = self.lookup.find_closest_note(frequency)?;
        let lookup_time = lookup_start.elapsed().as_millis();
        
        // Use frequency-match confidence directly (ignore power-based confidence)
        // Power-based confidence can be artificially low due to FFT bin resolution
        let final_confidence = note_confidence;
        
        let total_time = start.elapsed().as_millis();
        log::debug!("analyze_chunk: total={}ms, window={}ms, fft={}ms, find={}ms, lookup={}ms", 
            total_time, window_time, fft_time, find_time, lookup_time);
        
        Some((note_name, final_confidence))
    }
    
    /// Apply Hann window to reduce spectral leakage
    fn apply_hann_window(&self, signal: &[f32]) -> Vec<f32> {
        let n = signal.len() as f32;
        signal
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let window = 0.5 * (1.0 - (2.0 * PI * i as f32 / (n - 1.0)).cos());
                sample * window
            })
            .collect()
    }
    
    /// Analyze raw audio buffer (simpler version for HTTP requests)
    /// Takes raw bytes and interprets them as 16-bit PCM audio
    /// Returns multiple detected notes per chunk
    /// Only returns notes with confidence > 0.5 to filter out noise
    pub fn analyze_raw_bytes(&self, audio_data: &[u8], sample_rate: u32) -> Vec<(String, f32, f32)> {
        if audio_data.len() < 2 {
            return Vec::new();
        }
        
        let start = std::time::Instant::now();
        
        // Convert bytes to 16-bit samples
        let convert_start = std::time::Instant::now();
        let samples: Vec<f32> = audio_data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32;
                // Normalize to [-1.0, 1.0]
                sample / 32768.0
            })
            .collect();
        let convert_time = convert_start.elapsed().as_millis();
        
        // If we have enough samples, analyze as a single large chunk for better frequency resolution
        // Otherwise split into smaller chunks
        let analysis_start = std::time::Instant::now();
        let mut notes = if samples.len() >= 2048 {
            // Use multi-peak detection for better harmonic detection
            self.analyze_chunk_multi(&samples, sample_rate)
        } else {
            // Fallback to single note detection if not enough samples
            if let Some((note, confidence)) = self.analyze_chunk(&samples, sample_rate) {
                // Default intensity to 0.5 if not enough samples for power calculation
                vec![(note, confidence, 0.5)]
            } else {
                Vec::new()
            }
        };
        let analysis_time = analysis_start.elapsed().as_millis();
        
        // Filter out low-confidence noise (only keep notes with > 30% confidence)
        // IMPROVED: Lowered from 50% to 30% to allow weak bass fundamentals
        // Bass notes naturally have weaker fundamental amplitudes than their harmonics
        let filter_start = std::time::Instant::now();
        notes.retain(|(_, confidence, _)| *confidence > 0.30);
        let filter_time = filter_start.elapsed().as_millis();
        
        let total_time = start.elapsed().as_millis();
        log::debug!("analyze_raw_bytes: total={}ms, convert={}ms, analysis={}ms, filter={}ms", 
            total_time, convert_time, analysis_time, filter_time);
        
        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lookup_table_creation() {
        let lookup = FrequencyToNoteLookup::new();
        // Should have 7 natural notes × 5 octaves (C2 to C6) = 35 entries
        assert!(lookup.table.len() >= 35);
    }
    
    #[test]
    fn test_find_a4_frequency() {
        let lookup = FrequencyToNoteLookup::new();
        // A4 (440 Hz) should be found
        let (note_name, confidence) = lookup.find_closest_note(440.0).unwrap();
        assert_eq!(note_name, "A4");
        assert!(confidence > 0.99);
    }
    
    #[test]
    fn test_frequency_to_note_nearby() {
        let lookup = FrequencyToNoteLookup::new();
        // Frequency slightly off from A4 should still return A4 with lower confidence
        let (note_name, confidence) = lookup.find_closest_note(445.0).unwrap();
        assert_eq!(note_name, "A4");
        assert!(confidence < 0.99 && confidence > 0.9);
    }
}
