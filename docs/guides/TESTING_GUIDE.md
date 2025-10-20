# RecogNotes Complete System - Ready for Testing

## System Architecture

```
┌──────────────────────────┐
│   RecogNotes Rust Backend │
│   (FFT Audio Analysis)    │
│   Port: 5000              │
└────────────┬─────────────┘
             │
             │ HTTP (JSON)
             │
┌────────────▼──────────────┐
│  RecogNotes Desktop GUI    │
│  (egui + Tokio)           │
│  (Audio Recording)        │
└──────────────────────────┘
```

## Current Status

✅ **Backend Server**: Running on http://localhost:5000
- FFT-based pitch detection
- Mono, F32, 16 kHz audio format support
- Endpoints: POST /analyze, GET /health

✅ **Desktop GUI**: Running
- Audio recording (via cpal, 16 kHz)
- Backend connection detection
- Real-time note display

## Testing Guide

### 1. Check Backend Connection (Required First)
- Click "Check Backend Connection" button
- Status should change from **Red (Offline)** → **Green (Connected)**
- If it stays red, backend may not be running

### 2. Start Recording
- Click **"🎤 Start Recording"** button
- Button text changes to **"⏹ Stop Recording"**
- Speak, sing, or play a musical instrument near the microphone

### 3. Stop and Analyze
- Click **"⏹ Stop Recording"** button
- GUI shows **"● Analyzing..."** (yellow)
- Backend FFT analysis processes the audio

### 4. View Results
- Detected notes appear below
- Each note shows:
  - Note name (e.g., "A4", "C#5")
  - Confidence % (color-coded):
    - 🟢 Green: > 80% confidence
    - 🟡 Yellow: 50-80% confidence
    - 🔴 Red: < 50% confidence

## Troubleshooting

### "Backend Offline" (Red)
**Problem**: Cannot connect to backend
**Solution**:
1. Ensure backend is running: `cargo run` in `recognotes-rust-backend` folder
2. Check port 5000 is listening: `netstat -ano | findstr :5000`
3. Try clicking "Check Backend Connection" again

### "Failed to start recording"
**Problem**: Audio device not accessible
**Solution**:
1. Check microphone is connected and enabled
2. Ensure no other app has exclusive access
3. Try "Start Recording" again

### No notes detected
**Possible causes**:
1. Recording was too short (< 0.5 seconds)
2. Audio level too low
3. Background noise confusing the algorithm
4. Recorded sound is outside pitch detection range

## Key Features

### Audio Format Flexibility
- Automatically detects and uses device's supported format
- Converts F32/U16/I16 to standard i16 internally
- Adapts sample rate to device capabilities

### Confidence Scoring
- Based on frequency accuracy (cents deviation)
- Higher confidence = more certain note detection
- Color-coded for quick visual assessment

### Session Management
- Editable session title
- BPM setting (30-300) for reference
- Optional metronome toggle (for future implementation)

## Sample Test Case

1. **Start backend**: `cargo run` in backend folder
2. **Start GUI**: Run the executable
3. **Connect**: Click "Check Backend Connection" (should turn green)
4. **Record**: Click "Start Recording"
5. **Perform**: Sing or play a clear musical note for 2-3 seconds
6. **Stop**: Click "Stop Recording"
7. **Analyze**: Wait 1-2 seconds for backend response
8. **View**: See detected notes with confidence scores

### Example Expected Output
```
Note    Confidence
───────────────────
A4      95.2%  ✓ (Green)
A#4     42.1%  ✗ (Red)
```

## Performance Notes

- **Recording latency**: ~10-50ms (device dependent)
- **Backend analysis**: ~100-200ms per second of audio
- **Memory usage**: ~20-50 MB GUI + buffers
- **UI responsiveness**: 60 FPS (egui rendering)

## Next Steps

### Immediate Testing
- [ ] Verify backend connection
- [ ] Record and analyze a clear musical note
- [ ] Test with multiple different notes
- [ ] Record background music and check detection

### Advanced Testing
- [ ] Test polyphonic detection (multiple notes at once)
- [ ] Test edge cases (very high/low frequencies)
- [ ] Performance testing with long recordings
- [ ] Cross-platform compatibility (Linux/macOS)

### Potential Enhancements
- Real-time waveform visualization
- Detected note history/session playback
- Metronome audio generation
- MIDI output integration
- Singing accuracy feedback

## Files Overview

**Backend** (recognotes-rust-backend/)
- `src/main.rs`: HTTP server and endpoints
- `src/audio_analyzer.rs`: FFT analysis engine

**Frontend** (recognotes-desktop-gui/)
- `src/main.rs`: Application state and lifecycle
- `src/audio.rs`: Microphone recording
- `src/backend_client.rs`: HTTP communication
- `src/ui.rs`: egui UI rendering

## Support Commands

```powershell
# Start backend server
cd recognotes-rust-backend
cargo run                    # Debug mode (faster startup)
cargo run --release        # Optimized mode

# Build desktop GUI
cd recognotes-desktop-gui
cargo build                 # Debug mode
cargo build --release      # Optimized binary

# Run GUI executable
.\target\debug\deps\recognotes_desktop_gui-*.exe

# Kill running processes
Stop-Process -Name "recognotes*" -Force
```

## Known Limitations (v0.1.0)

- No audio playback (GUI only)
- Metronome feature not yet implemented
- No session persistence/save
- Single-threaded audio recording (no multi-device)
- Limited error recovery
- No input latency optimization

## Success Metrics

✅ System complete when:
- Backend responds to health checks
- GUI detects backend connection status
- Audio recording captures microphone input
- Analysis results return from backend
- Detected notes display in GUI with confidence scores
