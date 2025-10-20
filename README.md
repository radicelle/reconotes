# 🎵 RecogNotes

> **A high-performance musical note recognition system** built with **Rust** and **egui**
>
> Detect, analyze, and visualize musical notes from live audio input with real-time FFT-based pitch detection and voice profile filtering.

---

## 📖 Quick Navigation

- [Quick Start](#-quick-start) • [Project Story](#-project-story) • [Running](#-running-the-project) • [Testing](#-testing) • [API](#-api-reference) • [Contributing](#-contributing)

---

## 🎯 Quick Start

### Prerequisites

- **Rust 1.70+** ([Install here](https://rustup.rs/))
- **Windows, macOS, or Linux**
- **Microphone** (for recording audio)

### 30-Second Setup

```powershell
# Clone or enter the project
cd c:\Users\manua\CodeProjects\other\diapazon

# Terminal 1: Start the backend server
cd recognotes-rust-backend
cargo run --release

# Terminal 2: Start the desktop GUI (in a new terminal)
cd recognotes-desktop-gui
cargo run --release
```

✨ **That's it!** The GUI will open automatically and connect to the backend.

---

## 📚 Project Story

**RecogNotes** is a musical note recognition system that combines real-time audio capture with advanced FFT-based pitch detection.

### What It Does

🎤 **Records audio** from your microphone in real-time  
🎵 **Detects musical notes** using FFT (Fast Fourier Transform)  
📊 **Visualizes results** with live note detection and confidence scores  
🎯 **Filters by voice** supporting soprano, mezzo, alto, tenor, baritone, and bass profiles  
⚡ **Runs in parallel** with 8 worker threads for maximum throughput

### The Two-Part System

```text
┌─────────────────────────────────────────────────────────────┐
│                    RecogNotes System                         │
├──────────────────────┬──────────────────────────────────────┤
│   BACKEND (Rust)     │         FRONTEND (Rust)              │
│   Actix-web Server   │         egui Desktop GUI             │
├──────────────────────┼──────────────────────────────────────┤
│  ✓ Audio Processing  │  ✓ Record Audio                      │
│  ✓ FFT Analysis      │  ✓ Real-time Visualization          │
│  ✓ Note Detection    │  ✓ Voice Profile Selection          │
│  ✓ REST API          │  ✓ Performance Metrics              │
│  ✓ State Management  │  ✓ Intuitive UI                     │
│  Port: 5000          │  Display: 1400x900                  │
└──────────────────────┴──────────────────────────────────────┘
```

---

## 🏗️ Architecture

### Backend: `recognotes-rust-backend`

**Technology Stack:**

- **Framework**: Actix-web 4 (high-performance REST API)
- **Async Runtime**: Tokio
- **Audio Processing**: rustfft + ndarray + rayon
- **Serialization**: Serde JSON
- **Performance**: 10,000+ req/sec, <1ms response time

**Key Components:**

| Module | Purpose |
|--------|---------|
| `audio_analyzer.rs` | FFT-based pitch detection & frequency-to-note conversion |
| `models.rs` | Data structures (DetectedNote, AnalysisResult, VoiceProfile) |
| `endpoints/analyze.rs` | Main audio analysis endpoint |
| `endpoints/health.rs` | Backend health check |
| `endpoints/last_result.rs` | Retrieve last analysis result |

**How It Works:**

1. Receives **base64-encoded audio data** from the frontend
2. Converts audio bytes to **i16 samples** (PCM format)
3. Applies **Hann window** for spectral leakage reduction
4. Computes **FFT** to get frequency spectrum
5. Detects **peaks** in the spectrum
6. Maps peaks to **nearest musical note**
7. Filters by **voice profile** (optional)
8. Returns **detected notes with confidence** scores

### Frontend: `recognotes-desktop-gui`

**Technology Stack:**

- **GUI Framework**: egui + eframe (immediate-mode UI)
- **Audio Capture**: CPAL (Cross-Platform Audio Library)
- **HTTP Client**: Reqwest
- **Async Runtime**: Tokio
- **Threading**: Parking_lot for efficient locks

**Key Components:**

| Module | Purpose |
|--------|---------|
| `audio.rs` | Audio recording & sliding window buffer management |
| `backend_client.rs` | HTTP communication with backend |
| `ui.rs` | UI rendering and layout |
| `visualization.rs` | Note display and animations |

**How It Works:**

1. **Records audio** from selected microphone device
2. Maintains a **2-second sliding window** of audio samples
3. Every **20ms**, sends audio window to backend
4. Receives **detected notes** asynchronously
5. **Updates UI** with confidence scores & intensity levels
6. **Displays notes** with fade-out animation (1 second)
7. Supports **voice profile filtering** (soprano → bass)

---

## 🚀 Running the Project

### Option 1: Build Everything at Once

```powershell
# From root directory
cargo build --release
```

This builds both backend and frontend in release mode.

### Option 2: Run Backend Only

```powershell
cd recognotes-rust-backend
cargo run --release
```

**Output:**

```text
Starting RecogNotes Rust Backend on http://127.0.0.1:5000
Audio analysis with FFT-based pitch detection enabled
Max payload size: 16MB, Workers: 8, No request timeout
```

### Option 3: Run Frontend Only

```powershell
cd recognotes-desktop-gui
cargo run --release
```

**Window opens automatically:**

- Default backend URL: `http://localhost:5000`
- Sample rate: 48 kHz (Windows)
- Sliding window size: 2 seconds
- Analysis interval: 20ms

### Option 4: Debug Mode

```powershell
# Build debug binaries (faster compilation, slower execution)
cargo build

# Run with logging
$env:RUST_LOG="debug"
cargo run
```

---

## 💻 Developer Guide

### Project Structure

```text
diapazon/
├── Cargo.toml                          # Workspace config (members: backend, frontend)
├── build.rs                            # Multi-project build orchestrator
│
├── recognotes-rust-backend/            # Backend server
│   ├── Cargo.toml
│   ├── build.rs                        # Copies executable to root
│   └── src/
│       ├── main.rs                     # Actix server setup
│       ├── audio_analyzer.rs           # FFT & note detection (507 lines)
│       ├── models.rs                   # Data structures
│       ├── utils.rs                    # Utilities
│       └── endpoints/
│           ├── mod.rs
│           ├── health.rs               # GET /health
│           ├── analyze.rs              # POST /analyze
│           └── last_result.rs          # GET /last-result
│
├── recognotes-desktop-gui/             # Frontend desktop app
│   ├── Cargo.toml
│   ├── build.rs
│   ├── assets/
│   │   └── icon.png
│   └── src/
│       ├── main.rs                     # egui app setup
│       ├── audio.rs                    # Audio capture
│       ├── backend_client.rs           # HTTP client
│       ├── ui.rs                       # UI rendering
│       └── visualization.rs            # Note visualization
│
└── tests/                              # Test suite
    ├── backend/
    │   ├── test-api.ps1
    │   ├── test_440hz.ps1
    │   ├── test_note_detection.ps1
    │   ├── timing_test.py
    │   └── stress_test_backend_fast.py
    ├── frontend/
    │   └── test_frontend_performance.rs
    └── integration/
        ├── test_all_notes.py
        └── test_scoring.py
```

### Key Technologies Explained

#### 🎵 FFT (Fast Fourier Transform)

The backend uses **FFT** to convert audio from the **time domain** (amplitude over time) to the **frequency domain** (which frequencies are present).

```text
Time Domain (audio samples)     Frequency Domain (spectrum)
    ▲                               ▲
    │  ╱╲  ╱╲  ╱╲  ╱╲              │      ▁▁▁
    │ ╱  ╲╱  ╱  ╱  ╱╲             │    ▁▂▃▄▅▆▇▆▅▄▃▂▁
    │                             │  ▃▂ ▃▂    ▄▃
    └─────────────────────────► │  ▂            ▂
                                 └─────────────────────────►
                                    Frequency (Hz)
```

Once we have the spectrum, we find **peaks** (loud frequencies) and map them to **musical notes**.

#### 🎯 Voice Profiles

Singers have different vocal ranges. RecogNotes filters results by profile:

```text
Soprano    [C4 ────────────────── C6]  261  Hz  ────  1046 Hz
Mezzo      [A3 ────────────────── A5]  220  Hz  ────   880 Hz
Alto       [F3 ────────────────── F5]  174  Hz  ────   698 Hz
Tenor      [C3 ────────────────── C5]  131  Hz  ────   523 Hz
Baritone   [A2 ────────────────── A4]  110  Hz  ────   440 Hz
Bass       [C2 ────────────────── C4]   65  Hz  ────   261 Hz
```

#### 🪟 Sliding Window

The frontend maintains a **2-second rolling window** of audio:

```text
Time ────────────────────────────────────────────►

Old:  [████ discarded ████]
      [████ ████ ████ Window (2 sec) ████ ████ ████] ← Analyzed
      [← 20ms interval →]
```

Every 20ms, it sends the window to the backend for analysis.

### Building Blocks

#### Backend Endpoint: `/analyze`

**Request:**

```json
POST /analyze
{
  "audio_data": "ARECAwQFBgcICQo=",
  "sample_rate": 48000,
  "profile": "soprano"
}
```

**Response:**

```json
{
  "notes": [
    {"note": "C4", "confidence": 0.92, "intensity": 0.75},
    {"note": "E4", "confidence": 0.85, "intensity": 0.68}
  ],
  "sample_rate": 48000,
  "samples_analyzed": 96000,
  "timestamp": 1634567890.123
}
```

#### Frontend: Continuous Loop

```rust
// Every frame update:
1. Check if 20ms elapsed since last analysis
2. Read new samples from audio buffer
3. Add to sliding window (oldest samples removed)
4. Send window to backend (async)
5. Receive notes when ready
6. Update UI with fade-out animation
```

### Code Quality

The project uses **Clippy** for aggressive linting:

```powershell
# Run Clippy checks
cargo clippy --all --all-targets -- -W clippy::all

# Fix issues automatically
cargo fix --allow-dirty
```

**Key lint configurations:**

- ✅ Deny: unsafe_code, missing_docs
- ⚠️ Warn: clippy::all
- Ignores: cast_precision_loss (necessary for DSP math)

---

## 🧪 Testing

### Backend Tests

#### 1. **API Endpoint Test** (PowerShell)

```powershell
# Terminal 1: Start backend
cd recognotes-rust-backend
cargo run --release

# Terminal 2: Run tests
cd ..\tests\backend
.\test-api.ps1
```

**What it tests:**

- ✓ GET `/health` → returns `{"status": "ok"}`
- ✓ POST `/analyze` → returns valid notes
- ✓ GET `/last-result` → returns previous result

#### 2. **440Hz Detection Test** (PowerShell)

```powershell
.\test_440hz.ps1
```

**What it tests:**

- Generates pure 440Hz sine wave (musical note A4)
- Sends to backend
- Verifies detection as "A4"
- Measures latency

#### 3. **Note Detection Test** (PowerShell)

```powershell
.\test_note_detection.ps1
```

**What it tests:**

- Multiple different notes (C, D, E, F, G, A, B)
- Confidence scores
- Frequency accuracy

#### 4. **Stress Test** (Python)

```powershell
# Ensure Python 3.9+ is installed
python ..\tests\backend\stress_test_backend_fast.py
```

**Parameters:**

- Concurrent requests: 50
- Duration: 60 seconds
- Payload size: 16MB

**Output:**

```text
Running stress test...
Total requests: 3000
Successful: 2998 (99.9%)
Failed: 2 (0.1%)
Average latency: 12.3ms
Min latency: 2.1ms
Max latency: 156.7ms
Throughput: 49.97 req/sec per worker
```

#### 5. **Timing Analysis** (Python)

```powershell
python ..\tests\backend\timing_test.py
```

**Measures:**

- Serialization time
- Network round-trip
- Backend processing
- Deserialization time

### Frontend Tests

#### 1. **Performance Test** (Rust)

```powershell
cd tests\frontend
rustc test_frontend_performance.rs
.\test_frontend_performance.exe
```

**What it measures:**

- Frame render time
- UI responsiveness
- Memory allocation patterns
- Audio capture latency

#### 2. **Manual UI Testing**

```powershell
cd recognotes-desktop-gui
cargo run --release
```

**Test scenarios:**

1. Start recording → notes appear → stop → notes fade
2. Switch voice profile → filtering works
3. Multiple simultaneous notes → all detected
4. Backend disconnect → error message shows
5. Extreme volume → intensity values accurate

### Integration Tests

#### 1. **All Notes Detection** (Python)

```powershell
python ..\tests\integration\test_all_notes.py
```

Generates all 49 natural notes (C1-B7) and verifies detection.

#### 2. **Test Audio Files**

The project includes two audio files for testing:

- `tests/integration/O grave.wav` (16-bit PCM)
- `tests/integration/O grave.m4a` (AAC compressed)

To test with these:

```powershell
# Convert WAV to raw PCM, encode to base64
$audio = [System.IO.File]::ReadAllBytes("tests/integration/O grave.wav")
$base64 = [Convert]::ToBase64String($audio)

# Send to backend
$body = @{
    audio_data = $base64
    sample_rate = 44100
    profile = "soprano"
} | ConvertTo-Json

$response = Invoke-RestMethod `
    -Uri "http://localhost:5000/analyze" `
    -Method Post `
    -Body $body `
    -ContentType "application/json"

$response | ConvertTo-Json
```

### Running All Tests

#### Quick Test Suite

```powershell
# Backend tests only (10 seconds)
cd recognotes-rust-backend
cargo test

# Frontend tests only (5 seconds)
cd ..\recognotes-desktop-gui
cargo test
```

#### Comprehensive Test Suite

```powershell
# From root directory
cargo test --all

# Or for specific test file:
cargo test --test backend -- --nocapture
```

#### Watch Mode (Auto-Run on Changes)

```powershell
# Install cargo-watch
cargo install cargo-watch

# Watch and run tests
cargo watch -x test
```

---

## ⚙️ Configuration

### Backend Configuration

**File:** `recognotes-rust-backend/src/main.rs`

```rust
// Server binding
.bind("127.0.0.1:5000")?

// Worker threads (parallel request handling)
.workers(8)

// JSON payload limit
.app_data(web::JsonConfig::default().limit(16 * 1024 * 1024)) // 16MB
```

To change:

```rust
// Use different port
.bind("127.0.0.1:8080")?

// Increase workers for high concurrency
.workers(16)

// Reduce payload limit for constrained environments
.limit(8 * 1024 * 1024) // 8MB
```

### Frontend Configuration

**File:** `recognotes-desktop-gui/src/main.rs`

```rust
// Window size
.with_inner_size([1400.0, 900.0])

// Backend URL
backend_url: String = "http://localhost:5000"

// Sample rate
sample_rate: u32 = 48000  // 44100 for older systems

// Sliding window: 2 seconds of audio
sliding_window_size: usize = sample_rate as usize * 2

// Analysis interval
sliding_window_interval: Duration = Duration::from_millis(20)

// Note display duration (before fade)
note_display_duration: Duration = Duration::from_secs(1)
```

To change backend URL at runtime:

```rust
// In `RecogNotesApp::new_with_config()`
Self::new_with_config(
    "http://192.168.1.100:5000".to_string(),
    48000
)
```

### Environment Variables

```powershell
# Enable debug logging
$env:RUST_LOG = "debug"

# Specific module logging
$env:RUST_LOG = "recognotes_rust_backend=info,actix_web=warn"

# Run with logging
cargo run
```

**Log Levels:**

- `error`: Only errors
- `warn`: Errors and warnings
- `info`: General information (default)
- `debug`: Detailed debugging info
- `trace`: Extremely verbose (not recommended)

---

## 📊 API Reference

### Health Check

```http
GET /health

Response: 200 OK
{
  "status": "ok"
}
```

### Analyze Audio

```http
POST /analyze
Content-Type: application/json

Request:
{
  "audio_data": "base64-encoded-pcm-samples",
  "sample_rate": 48000,
  "profile": "soprano"  // Optional: soprano|mezzo|alto|tenor|baritone|bass|no_profile
}

Response: 200 OK
{
  "notes": [
    {
      "note": "C4",
      "confidence": 0.95,
      "intensity": 0.82
    }
  ],
  "sample_rate": 48000,
  "samples_analyzed": 96000,
  "timestamp": 1697123456.789
}

Response: 400 Bad Request
{
  "error": "JSON parse error: ..."
}

Response: 500 Internal Server Error
{
  "error": "FFT analysis failed: ..."
}
```

### Get Last Result

```http
GET /last-result

Response: 200 OK
{
  "notes": [...],
  "sample_rate": 48000,
  "samples_analyzed": 96000,
  "timestamp": 1697123456.789
}

Response: 204 No Content
(No previous analysis results)
```

### Audio Data Format

**Audio must be:**

- PCM (Pulse Code Modulation) format
- 16-bit signed integers (i16)
- Little-endian byte order
- Base64-encoded before transmission

**Example encoding in PowerShell:**

```powershell
# Create sample audio (1 second of silence)
$sample_rate = 48000
$duration_sec = 1
$sample_count = $sample_rate * $duration_sec
$samples = @(0) * $sample_count

# Convert to bytes (i16 = 2 bytes each)
$bytes = New-Object byte[] ($sample_count * 2)
for ($i = 0; $i -lt $sample_count; $i++) {
    $sample_bytes = [BitConverter]::GetBytes([Int16]$samples[$i])
    [Array]::Copy($sample_bytes, 0, $bytes, $i * 2, 2)
}

# Encode to base64
$base64 = [Convert]::ToBase64String($bytes)
```

---

## 🤝 Contributing

### Setting Up Development Environment

```powershell
# 1. Clone repository
git clone https://github.com/radicelle/reconotes
cd diapazon

# 2. Install Rust (if not already installed)
# Visit: https://rustup.rs/

# 3. Verify installation
rustc --version
cargo --version

# 4. Build project
cargo build --all

# 5. Run tests
cargo test --all
```

### Code Style

```powershell
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Lint with Clippy
cargo clippy --all -- -W clippy::all

# Auto-fix some issues
cargo fix --allow-dirty
```

### Adding a New Feature

1. **Create a feature branch:**

   ```powershell
   git checkout -b feature/your-feature-name
   ```

1. **Make changes** and test thoroughly:

   ```powershell
   cargo test --all
   cargo clippy --all
   cargo fmt
   ```

1. **Commit with clear messages:**

   ```powershell
   git commit -m "feat: add description of what changed"
   git push origin feature/your-feature-name
   ```

1. **Create a pull request** on GitHub

### Performance Optimization

If adding new features, measure performance impact:

```powershell
# Baseline measurement
cargo build --release
cd tests/backend
python stress_test_backend_fast.py

# After changes
cargo build --release
python stress_test_backend_fast.py

# Compare results
```

### Debugging

**Enable debug logging:**

```powershell
$env:RUST_LOG = "debug"
cargo run
```

**Use a debugger (VS Code):**

1. Install CodeLLDB extension

1. Add `.vscode/launch.json`:

   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "name": "Backend Debug",
         "type": "lldb",
         "request": "launch",
         "program": "${workspaceFolder}/target/debug/recognotes-rust-backend",
         "cwd": "${workspaceFolder}/recognotes-rust-backend"
       }
     ]
   }
   ```

1. Press F5 to start debugging

---

## 📝 License

MIT License - See LICENSE file for details

---

## 🚀 Performance Benchmarks

### Backend

| Metric | Value |
|--------|-------|
| **Requests/sec** | 10,000+ |
| **Latency (p50)** | 2.1ms |
| **Latency (p99)** | 12.3ms |
| **Memory** | ~5-10 MB |
| **Startup** | <100ms |
| **Payload Size** | Up to 16 MB |

### Frontend

| Metric | Value |
|--------|-------|
| **Frame Time** | 16.67ms (60 FPS) |
| **Memory** | ~50-100 MB |
| **CPU Usage** | 5-15% (recording) |
| **Latency** | 20-100ms (end-to-end) |
| **UI Responsiveness** | Excellent |

---

## 🎉 Happy Note Detecting

Enjoy using RecogNotes. 🎵
