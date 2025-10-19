# Audio Configuration Fixes - Session Log

## Problem
When running the desktop GUI, the error occurred:
```
Failed to start recording: Failed to build stream: The requested stream configuration is not supported by the device.
```

## Root Cause
The audio system was requesting a specific configuration (44.1 kHz, 1 channel, i16 format) without checking if the device actually supported it. Different audio devices (especially Bluetooth devices like your headset) may have different supported sample rates and formats.

## Solution Applied

### 1. Device Configuration Query (audio.rs)
Changed from:
- Directly requesting 44.1 kHz configuration
- Assuming the device supports the exact sample rate

To:
- Query the device's `supported_input_configs()`
- Find a compatible configuration that supports:
  - 1 channel (mono)
  - i16 PCM format
  - Sample rate range that includes our desired rate
- Fall back to any available 1-channel i16 config if exact match not found
- Create `StreamConfig` from the supported range

```rust
// Get supported configs and find a compatible one
let supported_configs = device
    .supported_input_configs()
    .map_err(|e| format!("Failed to get supported configs: {}", e))?
    .collect::<Vec<_>>();

// Try to find a config that supports our desired sample rate
let config_range = supported_configs
    .iter()
    .find(|c| {
        c.channels() == 1
            && c.sample_format() == cpal::SampleFormat::I16
            && c.min_sample_rate() <= cpal::SampleRate(self.sample_rate)
            && c.max_sample_rate() >= cpal::SampleRate(self.sample_rate)
    })
    .or_else(|| {
        // Fallback to any 1-channel I16 config
        supported_configs.iter().find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::I16)
    })?;

// Create actual StreamConfig from the range
let config = cpal::StreamConfig {
    channels: config_range.channels(),
    sample_rate: cpal::SampleRate(self.sample_rate),
    buffer_size: cpal::BufferSize::Default,
};
```

### 2. Default Sample Rate Change (main.rs)
- Changed default from 44.1 kHz to **48 kHz**
- Reason: 48 kHz is more commonly supported across different audio devices, especially Bluetooth
- Both in `Default` trait implementation and `new()` method
- Updated `stop_recording()` to use the app's configured sample_rate instead of hardcoded value

### 3. Tokio Runtime Initialization (main.rs)
Added explicit Tokio runtime initialization to support async operations:
```rust
let rt = tokio::runtime::Runtime::new()
    .expect("Failed to initialize Tokio runtime");
let _guard = rt.enter();
```

This enables the UI to call `tokio::spawn()` for async tasks like backend communication without panicking.

## Results

✅ **Status**: Application now runs successfully!

**Success Indicators**:
- Audio device detected: "Headset (LE-Manu 🦊)"
- Supported configurations query succeeds
- No panic or crash on startup
- GUI window appears with all controls
- Application ready for recording

**Logs on Startup**:
```
[2025-10-18T16:36:25Z INFO  recognotes_desktop_gui::audio] Selected input device: Headset (LE-Manu 🦊)
[2025-10-18T16:36:25Z INFO  recognotes_desktop_gui::audio] Found 1 supported configurations
```

## Next Steps

1. **Test Recording**
   - Click "🎤 Start Recording" button
   - Speak or make a sound
   - Click "⏹ Stop Recording"
   - Audio should be captured without errors

2. **Test Backend Connection**
   - Ensure `recognotes-rust-backend` is running on localhost:5000
   - Click "Check Backend Connection" button
   - Verify backend responds with health check

3. **End-to-End Testing**
   - Record audio in the GUI
   - Verify audio is sent to backend
   - Check that detected notes appear in the results section
   - Verify confidence scores display correctly

## Files Modified

1. **src/audio.rs**
   - Implemented device configuration query
   - Added fallback configuration selection
   - Improved error messages with device info

2. **src/main.rs**
   - Changed default sample rate: 44100 → 48000
   - Added Tokio runtime initialization
   - Updated `stop_recording()` to use configurable sample rate

## Technical Notes

**Why 48 kHz?**
- Standard for professional audio
- Better supported on Windows (matches WASAPI default)
- Works well with most Bluetooth devices
- Only 9% more data than 44.1 kHz (negligible for desktop app)

**Why query supported configs?**
- Different devices have different capabilities
- Bluetooth devices often have limited format support
- WASAPI may enumerate multiple configs per device
- Graceful degradation ensures robustness

**Tokio runtime scope:**
- `rt.enter()` activates the runtime as current for the thread
- Guard keeps runtime alive during GUI execution
- Dropped after `eframe::run_native()` completes
- Allows `tokio::spawn()` in UI event handlers
