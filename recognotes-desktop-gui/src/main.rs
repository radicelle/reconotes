mod audio;
mod backend_client;
mod ui;
mod visualization;

use eframe::egui;
use image::GenericImageView;
use parking_lot::RwLock;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .default_filter_or("warn")
            .write_style("always"),
    )
    .filter_module("recognotes_desktop_gui", log::LevelFilter::Info)
    .filter_module("recognotes_desktop_gui::audio", log::LevelFilter::Warn)
    .filter_module(
        "recognotes_desktop_gui::backend_client",
        log::LevelFilter::Warn,
    )
    .filter_module("reqwest", log::LevelFilter::Error)
    .filter_module("hyper", log::LevelFilter::Error)
    .filter_module("tokio", log::LevelFilter::Error)
    .init();

    // Create Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().expect("Failed to initialize Tokio runtime");
    let guard = rt.enter();

    let icon_data = load_icon().unwrap_or_else(|| {
        log::warn!("Failed to load icon from assets");
        create_default_icon()
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_icon(std::sync::Arc::new(icon_data)),
        ..Default::default()
    };

    let result = eframe::run_native(
        "RecogNotes Desktop",
        options,
        Box::new(|cc| Ok(Box::new(RecogNotesApp::new(cc)))),
    );

    drop(guard);
    drop(rt);

    result
}

fn load_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let rgba = image.to_rgba8();
    let (w, h) = image.dimensions();

    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width: w,
        height: h,
    })
}

fn create_default_icon() -> egui::IconData {
    // Create a simple 64x64 default icon (music note blue square)
    let size = 64;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    // Fill with light blue background
    for i in (0..rgba.len()).step_by(4) {
        rgba[i] = 100; // R
        rgba[i + 1] = 150; // G
        rgba[i + 2] = 200; // B
        rgba[i + 3] = 255; // A
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

/// Main application state
#[allow(clippy::struct_excessive_bools)]
pub struct RecogNotesApp {
    // Device selection
    selected_input_device: Option<String>,

    // UI state
    recording: bool,
    backend_connected: bool,
    backend_checked: bool, // Track if we've already checked health

    // Audio
    #[allow(clippy::arc_with_non_send_sync)]
    audio_manager: Arc<RwLock<audio::AudioManager>>,

    // Results
    detected_notes: Vec<DetectedNote>,
    detected_notes_history: Vec<(DetectedNote, f64)>, // (note, timestamp)
    last_error: Option<String>,

    // Backend URL
    backend_url: String,

    // Voice profile for filtering notes
    selected_profile: String, // "no_profile", "soprano", "mezzo", "alto", "tenor", "baritone", "bass"

    // Channel for receiving notes from async tasks
    notes_receiver: std::sync::mpsc::Receiver<Vec<DetectedNote>>,
    notes_sender: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Sender<Vec<DetectedNote>>>>,

    // Channel for backend health status
    health_receiver: std::sync::mpsc::Receiver<bool>,
    health_sender: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Sender<bool>>>,

    // Rolling history of detected notes with timestamps (last ~1 second)
    notes_with_timestamps: Vec<(DetectedNote, std::time::Instant)>,

    // Track when we last had ANY notes (for display timing)
    last_notes_received_time: std::time::Instant,

    // How long to keep displaying notes after they were last detected (1 second)
    note_display_duration: std::time::Duration,

    // Sliding window for 1-second audio analysis
    sliding_window_buffer: Vec<i16>,
    // Size of sliding window in samples (1 second at sample_rate)
    sliding_window_size: usize,
    // How often to slide the window and analyze (20ms)
    sliding_window_interval: std::time::Duration,
    // Last time we performed sliding window analysis
    last_sliding_window_analysis: std::time::Instant,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DetectedNote {
    pub note: String,
    pub confidence: f32,
    /// Power/intensity of the note (0.0-1.0)
    #[serde(default)]
    pub intensity: f32,
}

impl Default for RecogNotesApp {
    fn default() -> Self {
        Self::new_with_config(
            "http://localhost:5000".to_string(),
            48000, // 48kHz is more commonly supported on Windows
        )
    }
}

impl RecogNotesApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_config(
            "http://localhost:5000".to_string(),
            48000, // 48kHz is more commonly supported on Windows
        )
    }

    fn new_with_config(backend_url: String, sample_rate: u32) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (health_tx, health_rx) = std::sync::mpsc::channel();

        // Sliding window: 2 seconds of audio for better low-frequency resolution
        // At 48kHz: 48000 * 2 = 96000 samples
        let sliding_window_size = sample_rate as usize * 2;

        Self {
            recording: false,
            backend_connected: false,
            backend_checked: false,
            #[allow(clippy::arc_with_non_send_sync)]
            audio_manager: Arc::new(RwLock::new(audio::AudioManager::new(sample_rate))),
            detected_notes: Vec::new(),
            detected_notes_history: Vec::new(),
            last_error: None,
            backend_url,
            selected_profile: "no_profile".to_string(),
            notes_receiver: rx,
            notes_sender: Arc::new(std::sync::Mutex::new(tx)),
            health_receiver: health_rx,
            health_sender: Arc::new(std::sync::Mutex::new(health_tx)),
            notes_with_timestamps: Vec::new(),
            last_notes_received_time: std::time::Instant::now(),
            note_display_duration: std::time::Duration::from_secs(1),
            sliding_window_buffer: Vec::with_capacity(sliding_window_size),
            sliding_window_size,
            sliding_window_interval: std::time::Duration::from_millis(20),
            last_sliding_window_analysis: std::time::Instant::now(),
            selected_input_device: None,
        }
    }

    fn start_recording(&mut self) {
        self.recording = true;
        self.last_error = None;

        // Pre-fill the sliding window buffer with silence (2 seconds worth)
        self.sliding_window_buffer.clear();
        self.sliding_window_buffer
            .extend(std::iter::repeat_n(0i16, self.sliding_window_size));
        log::debug!(
            "Initialized sliding window buffer with {} silent samples",
            self.sliding_window_size
        );

        // Set the device on the audio manager before starting
        let mut manager = self.audio_manager.write();
        manager.set_device(self.selected_input_device.clone());

        if let Err(e) = manager.start_recording() {
            self.last_error = Some(format!("Failed to start recording: {e}"));
            self.recording = false;
        }
    }

    fn stop_recording(&mut self) {
        self.recording = false;

        let mut manager = self.audio_manager.write();
        if let Err(e) = manager.stop_recording() {
            self.last_error = Some(format!("Failed to stop recording: {e}"));
        }
    }

    fn continuous_analysis(&mut self) {
        // Check if it's time to analyze (every 20ms for sliding window)
        if self.last_sliding_window_analysis.elapsed() < self.sliding_window_interval {
            return;
        }

        self.last_sliding_window_analysis = std::time::Instant::now();

        if !self.recording {
            return;
        }

        // Add new audio to sliding window (replaces oldest samples with newest)
        let manager = self.audio_manager.write();
        manager.add_to_sliding_buffer(&mut self.sliding_window_buffer, self.sliding_window_size);
        drop(manager);

        // Get the actual sample rate from the audio manager after it has been configured.
        let sample_rate = self.audio_manager.read().sample_rate();

        // Buffer is always pre-filled with silence, so we always have 2 seconds ready
        if self.sliding_window_buffer.len() < self.sliding_window_size {
            log::debug!(
                "Waiting for sliding buffer to fill: {}/{} samples",
                self.sliding_window_buffer.len(),
                self.sliding_window_size
            );
            return;
        }

        // Convert sliding window buffer to bytes and send immediately
        let mut audio_data = Vec::with_capacity(self.sliding_window_buffer.len() * 2);
        for &sample in &self.sliding_window_buffer {
            audio_data.extend_from_slice(&sample.to_le_bytes());
        }

        let backend_url = self.backend_url.clone();
        let sender = Arc::clone(&self.notes_sender);
        let data_len = audio_data.len();
        let profile = if self.selected_profile == "no_profile" {
            None
        } else {
            Some(self.selected_profile.clone())
        };
        let profile_display = profile.as_deref().unwrap_or("no_profile").to_string();

        // Spawn async task to send to backend
        tokio::spawn(async move {
            let client_start = std::time::Instant::now();
            match backend_client::analyze_audio(&backend_url, audio_data, sample_rate, profile)
                .await
            {
                Ok(notes) => {
                    let total_client_ms = client_start.elapsed().as_millis();
                    log::info!(
                        "Backend response [{}]: {} notes from {}B audio in {}ms",
                        profile_display,
                        notes.len(),
                        data_len,
                        total_client_ms
                    );
                    let _ = sender.lock().unwrap().send(notes);
                }
                Err(e) => {
                    let total_client_ms = client_start.elapsed().as_millis();
                    log::error!("Backend error after {total_client_ms}ms: {e}");
                }
            }
        });

        // Receive any notes from completed async tasks
        let now = std::time::Instant::now();
        if let Ok(notes) = self.notes_receiver.try_recv() {
            if !notes.is_empty() {
                log::info!("🎵 Received {} notes from backend", notes.len());
                for note in &notes {
                    log::info!(
                        "   - {} ({:.0}% confidence)",
                        note.note,
                        note.confidence * 100.0
                    );

                    // Add each note to rolling history with timestamp
                    self.notes_with_timestamps.push((note.clone(), now));
                }
                self.last_notes_received_time = now;
            }

            // Clean up old notes (older than display duration)
            let cutoff_time = now.checked_sub(self.note_display_duration).unwrap();
            self.notes_with_timestamps
                .retain(|(_, timestamp)| *timestamp > cutoff_time);

            // Build current detected_notes from the recent history (for UI display)
            let mut unique_notes = std::collections::HashMap::new();
            for (note, _timestamp) in &self.notes_with_timestamps {
                unique_notes
                    .entry(note.note.clone())
                    .and_modify(|existing: &mut DetectedNote| {
                        if note.confidence > existing.confidence {
                            *existing = note.clone();
                        }
                    })
                    .or_insert_with(|| note.clone());
            }
            self.detected_notes = unique_notes.into_values().collect();

            // Sort by note name for consistent display
            self.detected_notes.sort_by(|a, b| a.note.cmp(&b.note));
        } else {
            // If no new notes received, clean up old ones based on display duration
            let cutoff_time = now.checked_sub(self.note_display_duration).unwrap();
            self.notes_with_timestamps
                .retain(|(_, timestamp)| *timestamp > cutoff_time);

            // If all notes have expired, clear display
            if self.notes_with_timestamps.is_empty() {
                self.detected_notes.clear();
            }
        }
    }
}

impl eframe::App for RecogNotesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check backend connection only once on startup
        if !self.backend_checked {
            self.backend_checked = true;
            let backend_url = self.backend_url.clone();
            let sender = Arc::clone(&self.health_sender);
            tokio::spawn(async move {
                let is_healthy = backend_client::check_health(&backend_url).await.is_ok();
                if is_healthy {
                    log::debug!("✓ Backend health check passed on startup");
                }
                let _ = sender.lock().unwrap().send(is_healthy);
            });
        }

        // Check if backend health result came back
        if let Ok(is_healthy) = self.health_receiver.try_recv() {
            self.backend_connected = is_healthy;
        }

        // Continuous analysis if recording
        self.continuous_analysis();

        // Request repaint to keep analysis running at the sound format frequency
        // This ensures the update loop runs continuously even without mouse movement
        // Also needed for smooth fade animation
        if self.recording {
            ctx.request_repaint();
        } else if !self.notes_with_timestamps.is_empty() {
            // Keep repainting while notes are fading out (for 2 seconds)
            ctx.request_repaint();
        }

        ui::draw_ui(self, ctx);
    }
}
