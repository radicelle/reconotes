use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioManager {
    sample_rate: u32,
    stream: Option<cpal::Stream>,
    audio_buffer: Arc<Mutex<Vec<i16>>>,
    recording: bool,
    selected_device: Option<String>,
}

impl AudioManager {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            stream: None,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            recording: false,
            selected_device: None,
        }
    }
    
    /// Set the device to use for recording
    pub fn set_device(&mut self, device_name: Option<String>) {
        self.selected_device = device_name;
    }

    /// Get list of available input devices
    pub fn get_input_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut devices = vec!["Default".to_string()];
        
        if let Ok(device_iter) = host.input_devices() {
            for device in device_iter {
                if let Ok(name) = device.name() {
                    devices.push(name);
                }
            }
        }
        
        devices
    }

    /// Get list of available output devices
    #[allow(dead_code)]
    pub fn get_output_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut devices = vec!["Default".to_string()];
        
        if let Ok(device_iter) = host.output_devices() {
            for device in device_iter {
                if let Ok(name) = device.name() {
                    devices.push(name);
                }
            }
        }
        
        devices
    }

    #[allow(clippy::too_many_lines)]
    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.recording {
            return Err("Already recording".to_string());
        }

        let host = cpal::default_host();
        
        // Get the selected device, or default if none selected
        let device = if let Some(device_name) = &self.selected_device {
            // Find device by name
            host.input_devices()
                .map_err(|e| format!("Failed to get input devices: {e}"))?
                .find(|d| {
                    d.name()
                        .ok()
                        .is_some_and(|name| name == *device_name)
                })
                .ok_or_else(|| format!("Device '{device_name}' not found"))?
        } else {
            // None means use default device
            host.default_input_device()
                .ok_or_else(|| "No input device available".to_string())?
        };

        log::info!("Selected input device: {}", device.name().unwrap_or_default());

        // Get supported configs and find a compatible one
        let supported_configs = device
            .supported_input_configs()
            .map_err(|e| format!("Failed to get supported configs: {e}"))?
            .collect::<Vec<_>>();

        log::info!("Found {} supported configurations", supported_configs.len());
        for (idx, cfg) in supported_configs.iter().enumerate() {
            log::info!(
                "  Config {}: {} channels, sample rate range {:?}-{:?}, format {:?}",
                idx,
                cfg.channels(),
                cfg.min_sample_rate(),
                cfg.max_sample_rate(),
                cfg.sample_format()
            );
        }

        // Try multiple fallback strategies in order of preference
        let config_range = supported_configs
            .iter()
            // Priority 1: Our exact requirements (mono, i16, compatible sample rate)
            .find(|c| {
                c.channels() == 1
                    && c.sample_format() == cpal::SampleFormat::I16
                    && c.min_sample_rate() <= cpal::SampleRate(self.sample_rate)
                    && c.max_sample_rate() >= cpal::SampleRate(self.sample_rate)
            })
            // Priority 2: Any mono + i16 (flexible sample rate)
            .or_else(|| {
                supported_configs
                    .iter()
                    .find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::I16)
            })
            // Priority 3: Mono with any format (flexible format)
            .or_else(|| supported_configs.iter().find(|c| c.channels() == 1))
            // Priority 4: First available config (any channels, any format)
            .or_else(|| supported_configs.first())
            .ok_or_else(|| "No audio configuration available".to_string())?;

        // Use the maximum sample rate the config supports (usually best quality)
        let actual_sample_rate = std::cmp::min(
            self.sample_rate,
            config_range.max_sample_rate().0,
        );
        let actual_sample_rate = std::cmp::max(actual_sample_rate, config_range.min_sample_rate().0);

        log::info!(
            "Using config: {} channels, target {} Hz (actual: {} Hz), format {:?}",
            config_range.channels(),
            self.sample_rate,
            actual_sample_rate,
            config_range.sample_format()
        );

        // Create actual StreamConfig from the range
        let config = cpal::StreamConfig {
            channels: config_range.channels(),
            sample_rate: cpal::SampleRate(actual_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Update our actual sample rate for later use
        self.sample_rate = actual_sample_rate;

        let audio_buffer_i16 = Arc::clone(&self.audio_buffer);

        // Build an I16 stream - try all supported formats
        let stream = match config_range.sample_format() {
            cpal::SampleFormat::I16 => {
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buffer = audio_buffer_i16.lock().unwrap();
                        buffer.extend_from_slice(data);
                    },
                    |err| log::error!("Stream error: {err}"),
                )
            }
            cpal::SampleFormat::U16 => {
                device.build_input_stream(
                    &config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut buffer = audio_buffer_i16.lock().unwrap();
                        for &sample in data {
                            // Convert U16 to I16
                            #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                            let i16_sample = (i32::from(sample) - 32768) as i16;
                            buffer.push(i16_sample);
                        }
                    },
                    |err| log::error!("Stream error: {err}"),
                )
            }
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buffer = audio_buffer_i16.lock().unwrap();
                        for &sample in data {
                            // Convert F32 to I16: [-1.0, 1.0] -> [-32768, 32767]
                            #[allow(clippy::cast_possible_truncation)]
                            let i16_sample = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            buffer.push(i16_sample);
                        }
                    },
                    |err| log::error!("Stream error: {err}"),
                )
            }
        }
        .map_err(|e| format!("Failed to build stream: {e}"))?;

        stream.play().map_err(|e| format!("Failed to play stream: {e}"))?;

        self.stream = Some(stream);
        self.recording = true;

        log::info!("Recording started at {actual_sample_rate} Hz");
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<u8>, String> {
        if !self.recording {
            return Err("Not recording".to_string());
        }

        // Stop the stream
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        self.recording = false;

        // Extract audio data from buffer
        let samples = self.audio_buffer.lock().unwrap().drain(..).collect::<Vec<_>>();

        // Convert i16 samples to bytes
        let mut audio_data = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            audio_data.extend_from_slice(&sample.to_le_bytes());
        }

        log::info!("Recording stopped. Captured {} bytes", audio_data.len());

        Ok(audio_data)
    }

    /// Add samples to sliding window buffer
    /// Used for maintaining a rolling 1-second window of audio data
    pub fn add_to_sliding_buffer(&self, sliding_buffer: &mut Vec<i16>, buffer_size: usize) {
        let mut buffer = self.audio_buffer.lock().unwrap();
        if buffer.is_empty() {
            return;
        }

        // Add all available samples to sliding buffer
        sliding_buffer.extend_from_slice(&buffer);
        buffer.clear();
        drop(buffer);

        // Keep only the most recent buffer_size samples (1 second window)
        if sliding_buffer.len() > buffer_size {
            let drain_count = sliding_buffer.len() - buffer_size;
            sliding_buffer.drain(..drain_count);
        }
    }

    /// Get buffered audio without stopping recording (for continuous analysis)
    /// Returns up to `chunk_size` bytes to keep payloads consistent
    #[allow(dead_code)]
    pub fn get_buffered_audio_chunk(&self, chunk_size: usize) -> Result<Vec<u8>, String> {
        if !self.recording {
            return Err("Not recording".to_string());
        }

        let mut buffer = self.audio_buffer.lock().unwrap();
        if buffer.is_empty() {
            return Ok(Vec::new());
        }

        // Take only up to chunk_size bytes worth of samples
        let max_samples = chunk_size / 2; // 2 bytes per i16 sample
        let take_count = std::cmp::min(buffer.len(), max_samples);
        
        let samples: Vec<i16> = buffer.drain(..take_count).collect();
        drop(buffer);

        // Convert i16 samples to bytes
        let mut audio_data = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            audio_data.extend_from_slice(&sample.to_le_bytes());
        }

        Ok(audio_data)
    }

    #[allow(dead_code)]
    pub const fn is_recording(&self) -> bool {
        self.recording
    }

    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

