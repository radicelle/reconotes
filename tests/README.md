# RecogNotes Tests

This directory contains all project tests organized by category.

## 📝 Structure

### `/backend`

Backend API and audio processing tests:

- **test-api.ps1** - REST API endpoint tests
- **test_440hz.ps1** - 440Hz frequency detection test
- **test_note_detection.ps1** - Note detection validation
- **demo-fft.ps1** - FFT demonstration script
- **timing_test.py** - Performance timing tests
- **stress_test_backend_fast.py** - Stress testing script
- **analyze_confidence.py** - Confidence level analysis
- **voice_recorder_fft.py** - Voice recording FFT analysis

### `/frontend`

Frontend GUI and UI tests:

- **test_frontend_performance.rs** - Frontend performance testing

### `/integration`

Integration tests and test audio files:

- **test_all_notes.py** - Test detection of all musical notes
- **test_scoring.py** - Scoring system tests
- **O grave.wav** - Test audio file (WAV format)
- **O grave.m4a** - Test audio file (M4A format)

## 🚀 Running Tests

### Backend Tests

```bash
# Test API endpoints
powershell tests/backend/test-api.ps1

# Test 440Hz detection
powershell tests/backend/test_440hz.ps1

# Run Python tests
python tests/backend/test_scoring.py
python tests/backend/stress_test_backend_fast.py
```

### Frontend Tests

```bash
# Compile and run frontend performance test
rustc tests/frontend/test_frontend_performance.rs
```

### Integration Tests

```bash
# Test all notes detection
python tests/integration/test_all_notes.py

# Test scoring system
python tests/integration/test_scoring.py
```

## 📁 Test Data

Audio test files are located in `/integration`:

- `O grave.wav` - Main test audio file
- `O grave.m4a` - Alternative format test file
