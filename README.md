# 🎵 RecogNotes

> **A musical note recognition system** built with **Rust** and **egui**
>
> Records audio from your microphone, runs FFT analysis, and displays detected notes with colorful bars. Features a two-process architecture (backend + frontend) because we believe in making simple things complicated.

---

## 📖 Quick Navigation

- [Quick Start](#-quick-start) • [What It Does](#-what-this-thing-actually-does) • [Running](#-running-the-project) • [Testing](#-testing) • [API](#-api-reference) • [Contributing](#-contributing)

---

## 🎯 Quick Start

### Prerequisites

- **Rust 1.70+** ([Install here](https://rustup.rs/))
- **cargo-make** (Install: `cargo install cargo-make`)
- **Windows, macOS, or Linux**
- **Microphone** (for recording audio)

### 30-Second Setup

```powershell
# Clone or enter the project
cd c:\Users\manua\CodeProjects\other\diapazon

# Install cargo-make (one-time setup)
cargo install cargo-make

# Build both backend and GUI
cargo make

# Terminal 1: Start the backend server
.\recognotes-rust-backend.exe

# Terminal 2: Start the desktop GUI (in a new terminal)
.\recognotes-desktop-gui.exe
```

✨ **That's it!** The GUI will open automatically and connect to the backend.

---

## 📚 What This Thing Actually Does

**RecogNotes** is a musical note recognition system that listens to your microphone and tries its best to figure out what notes you're singing (or playing, we don't judge).

### The Pitch (pun intended)

🎤 **Records audio** from your microphone – because apparently we needed another reason to use Rust for desktop apps  
🎵 **Detects musical notes** using FFT (Fast Fourier Transform) – yes, we're doing actual math here  
📊 **Visualizes results** with colorful bars – because what's a music app without gratuitous graphics?  
🎯 **Filters by voice profile** – if you tell it you're a soprano, it'll politely ignore that low C you just sang  
⚡ **Runs with 8 worker threads** – because we read that Actix uses them, so we're definitely "high performance"

### The Two-Part Architecture (Because Microservices Are Cool)

| **BACKEND (Rust)** | **FRONTEND (Rust)** |
|:-------------------|:--------------------|
| _Actix-web Server_ | _egui Desktop GUI_  |
| ✓ Audio Processing | ✓ Record Audio |
| ✓ FFT Analysis | ✓ Send to Backend Every 20ms |
| ✓ Note Detection | ✓ Draw Pretty Bars |
| ✓ REST API | ✓ Voice Profile Dropdown |
| ✓ State Management | ✓ Fade Animations (fancy!) |
| **Port:** 5000 | **Display:** 1400x900 |

**Translation:** We have a server that does FFT math and a GUI that records audio and draws bars. They talk via HTTP because... reasons. Could this be one program? Sure. But where's the fun in that?

---

## 🏗️ Architecture (The Technical Bits)

### Backend: `recognotes-rust-backend`

**Technology Stack:**

- **Framework**: Actix-web 4 (because async Rust is "fun")
- **Async Runtime**: Tokio (required by Actix)
- **Audio Processing**: rustfft + ndarray + rayon (the actual workers)
- **Serialization**: Serde JSON (because base64-encoded audio over HTTP is totally normal)

**Key Components:**

| Module                    | Purpose                          | Reality Check                                 |
|---------------------------|----------------------------------|-----------------------------------------------|
| `audio_analyzer.rs`       | FFT-based pitch detection        | Does math on audio samples, ~500 lines        |
| `models.rs`               | Data structures                  | Structs for notes, results, voice profiles    |
| `endpoints/analyze.rs`    | Main audio analysis endpoint     | Receives base64 audio, returns notes          |
| `endpoints/health.rs`     | Backend health check             | Returns `{"status": "ok"}` every time         |
| `endpoints/last_result.rs`| Retrieve last analysis result    | Keeps one result in memory, that's it         |

**How It Actually Works:**

1. Receives **base64-encoded audio data** from the frontend (yes, really)
2. Decodes base64 to raw bytes, converts to **i16 samples**
3. Applies a **Hann window** (math to reduce FFT artifacts)
4. Computes **FFT** to convert audio to frequency spectrum
5. Finds **peaks** in the spectrum (loud frequencies)
6. Maps peaks to **nearest musical note** using pre-computed lookup table
7. Filters by **voice profile** if you asked for it (otherwise gives you everything)
8. Returns top 3 notes with "confidence" scores (really just amplitude ratios)

### Frontend: `recognotes-desktop-gui`

**Technology Stack:**

- **GUI Framework**: egui + eframe (immediate-mode UI, no retained state)
- **Audio Capture**: CPAL (Cross-Platform Audio Library)
- **HTTP Client**: Reqwest (to talk to our totally-necessary backend)
- **Async Runtime**: Tokio (because everything is async now)

**Key Components:**

| Module | Purpose | What It Actually Does |
|--------|---------|----------------------|
| `audio.rs` | Audio recording & buffer management | Captures mic input, stores samples |
| `backend_client.rs` | HTTP communication | POSTs audio every 20ms, deserializes response |
| `ui.rs` | UI rendering and layout | Draws the whole interface |
| `visualization.rs` | Note display and animations | Colorful bars that fade out (the important part) |

**How It Actually Works:**

1. **Records audio** from your selected microphone (or default if you're lazy)
2. Keeps a **2-second sliding window** of audio samples in a ring buffer
3. Every **20ms**, base64-encodes the entire 2-second window and HTTP POSTs it to the backend
4. Waits for response (asynchronously, of course)
5. **Updates UI** with detected notes (bars light up in pretty colors)
6. Notes **fade out** over 600ms (because animations make everything better)
7. Voice profile dropdown actually filters the display range, not just the detection

---

## 🚀 Running the Project

### Build System

This project uses **[cargo-make](https://github.com/sagiegurari/cargo-make)** for build automation, providing a platform-independent and maintainable build system.

**One-Time Setup:**

```powershell
cargo install cargo-make
```

**Benefits:**

- ✅ Platform-independent (works on Windows, macOS, Linux)
- ✅ Simplified build process (single command to build everything)
- ✅ Maintainable (all build logic in `Makefile.toml`)
- ✅ Flexible (easy to add new tasks)
- ✅ Automatic executable copying (binaries placed in project root)

**Available Tasks:**

```powershell
# Build everything (default task)
cargo make                    # Builds both backend and GUI, copies executables to root

# Build individual components
cargo make build-backend      # Build backend in release mode
cargo make build-gui          # Build GUI in release mode

# Copy executables to root
cargo make copy-backend       # Build and copy backend executable
cargo make copy-gui           # Build and copy GUI executable

# Code quality
cargo make format             # Format all code with rustfmt
cargo make format-check       # Check formatting without modifying
cargo make format-backend     # Format backend only
cargo make format-gui         # Format GUI only
cargo make clippy             # Run clippy with aggressive settings on both projects
cargo make clippy-backend     # Clippy on backend only
cargo make clippy-gui         # Clippy on GUI only

# Development tasks
cargo make check              # Fast check of both projects
cargo make check-backend      # Check backend only
cargo make check-gui          # Check GUI only

# Cleanup
cargo make clean              # Clean all build artifacts
cargo make clean-backend      # Clean backend only
cargo make clean-gui          # Clean GUI only
```

**Legacy Build Scripts:**

The project includes legacy build scripts (`build.bat`, `build.ps1`, `build.sh`) that were used before cargo-make. These are maintained for backward compatibility but **cargo-make is now the recommended approach** for building the project.

### Option 1: Build Everything at Once (Recommended)

```powershell
# From root directory
cargo make
```

This builds both backend and frontend in release mode and copies executables to the project root. You can then run them directly:

```powershell
# Terminal 1: Start the backend
.\recognotes-rust-backend.exe

# Terminal 2: Start the GUI
.\recognotes-desktop-gui.exe
```

**Alternative:** Traditional cargo build (slower, doesn't auto-copy executables)

```powershell
cargo build --release
```

### Option 2: Run Backend Only

```powershell
# Using cargo-make (builds and runs)
cargo make build-backend
.\recognotes-rust-backend.exe

# Or build and run in one step from backend directory
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
# Using cargo-make (builds and runs)
cargo make build-gui
.\recognotes-desktop-gui.exe

# Or build and run in one step from GUI directory
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

# Or from subdirectories
cd recognotes-rust-backend
cargo run  # Runs in debug mode by default

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
├── Makefile.toml                       # Cargo-make build automation
├── build.bat                           # Legacy Windows build script
├── build.ps1                           # Legacy PowerShell build script
├── build.sh                            # Legacy Unix build script
│
├── recognotes-rust-backend/            # Backend server
│   ├── Cargo.toml
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

### Key Technologies Explained (ELI5 Edition)

#### 🎵 FFT (Fast Fourier Transform)

The backend uses **FFT** to convert audio from the **time domain** (wiggly lines of amplitude) to the **frequency domain** (which pitches are present).

```text
Time Domain (what you record)       Frequency Domain (what we analyze)
         ▲                                    ▲
         │   ╱╲   ╱╲   ╱╲   ╱╲              │       ▁▁▁
         │  ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲            │     ▁▂▃▄▅▆▇▆▅▄▃▂▁
         │ ╱    ╲    ╱    ╱    ╲           │   ▃▂  ▃▂     ▄▃
         │╱                       ╲         │  ▂             ▂
         └──────────────────────────►       └───────────────────────►
                Time                              Frequency (Hz)
```

Translation: It's magic math that tells us "this audio contains 440 Hz" instead of "here's 96,000 numbers in a list."

#### 🎯 Voice Profiles

Different voices sing different ranges. We filter the results based on what's physically possible for each voice type:

```text
Voice Type    Note Range                 Frequency Range
─────────────────────────────────────────────────────────────
Soprano       [C4 ══════════════════ C6]  261 Hz ──── 1046 Hz
Mezzo         [A3 ══════════════════ A5]  220 Hz ────  880 Hz
Alto          [F3 ══════════════════ F5]  174 Hz ────  698 Hz
Tenor         [C3 ══════════════════ C5]  131 Hz ────  523 Hz
Baritone      [A2 ══════════════════ A4]  110 Hz ────  440 Hz
Bass          [C2 ══════════════════ C4]   65 Hz ────  261 Hz
```

If you select "soprano" and sing a low C2, we'll pretend we didn't hear it. You're welcome.

#### 🪟 Sliding Window

The frontend keeps a **2-second rolling window** of audio samples. Think of it as a constantly-updating buffer:

```text
Time ────────────────────────────────────────────────────────────►

Old samples:  [discarded and forgotten]
                   │
Current:      [████ ████ Window (2 sec) ████ ████] ← Sent to backend
              └──────────────┬──────────────┘
                    Every 20ms, shift and repeat
```

Why 2 seconds? Because shorter windows make FFT less accurate, and longer windows make the UI feel laggy. It's a compromise.

### Building Blocks (What Actually Happens)

#### Backend Endpoint: `/analyze`

**Request:**

```json
POST /analyze
{
  "audio_data": "ARECAwQFBgcICQo=",  // 2 seconds of audio, base64-encoded
  "sample_rate": 48000,               // Usually 48000 on Windows
  "profile": "soprano"                // Optional filtering
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

Note: "confidence" is really just "how loud this frequency was relative to others." We call it confidence because it sounds more sophisticated.

#### Frontend: The Loop

```rust
// Every 20ms:
1. Check if backend is alive (health check)
2. Grab 2 seconds of audio from the ring buffer
3. Base64-encode it (yes, all of it, every time)
4. POST to /analyze
5. Wait for response (async)
6. Light up the bars for detected notes
7. Start fading them out after 600ms
8. Repeat forever (or until you stop recording)
```

Could we optimize this? Sure. Will we? Probably not. It works.

### Code Quality

The project uses **Clippy** with aggressive settings (because we like to be yelled at by the compiler):

```powershell
# Run Clippy checks
cargo make clippy

# Traditional approach
cargo clippy --all --all-targets -- -W clippy::all

# Fix issues automatically (when possible)
cargo fix --allow-dirty
```

**Key lint configurations:**

- ✅ Deny: unsafe_code, missing_docs (we're civilized here)
- ⚠️ Warn: clippy::all, clippy::pedantic, clippy::nursery (pain is growth)
- 🤷 Allow: cast_precision_loss (because FFT math requires float conversions, clippy, deal with it)

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
- Payload size: 16MB (because we like to test the limits)

**What it actually tells you:**

```text
Running stress test...
Total requests: 3000
Successful: 2998 (99.9%)
Failed: 2 (0.1%)  ← These failed because Windows, probably
Average latency: 12.3ms
Min latency: 2.1ms
Max latency: 156.7ms  ← That one time the GC kicked in
Throughput: 49.97 req/sec per worker
```

Translation: The backend can handle way more traffic than a single desktop GUI will ever throw at it. But hey, at least we know.

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

### Configuration

### Backend Configuration

**File:** `recognotes-rust-backend/src/main.rs`

```rust
// Server binding
.bind("127.0.0.1:5000")?

// Worker threads (Actix does this automatically, but we can pretend it's for performance)
.workers(8)

// JSON payload limit (because 2 seconds of audio at 48kHz is ~190KB)
.app_data(web::JsonConfig::default().limit(16 * 1024 * 1024)) // 16MB, just to be safe
```

To change:

```rust
// Use different port
.bind("127.0.0.1:8080")?

// Increase workers if you're expecting a lot of concurrent users (you won't)
.workers(16)

// Reduce payload limit if you want to save memory (you probably don't need to)
.limit(8 * 1024 * 1024) // 8MB
```

### Frontend Configuration

**File:** `recognotes-desktop-gui/src/main.rs`

```rust
// Window size (arbitrary, but it fits nicely on a laptop)
.with_inner_size([1400.0, 900.0])

// Backend URL (hardcoded because dynamic config is for enterprise apps)
backend_url: String = "http://localhost:5000"

// Sample rate (48000 on Windows, 44100 on older systems)
sample_rate: u32 = 48000

// Sliding window: 2 seconds of audio (trade-off between accuracy and latency)
sliding_window_size: usize = sample_rate as usize * 2

// Analysis interval (how often we bother the backend)
sliding_window_interval: Duration = Duration::from_millis(20)

// Note display duration before fade (long enough to see, short enough to not clutter)
note_display_duration: Duration = Duration::from_millis(600)
```

### Environment Variables

```powershell
# Enable debug logging (prepare for spam)
$env:RUST_LOG = "debug"

# Specific module logging (more reasonable)
$env:RUST_LOG = "recognotes_rust_backend=info,actix_web=warn"

# Run with logging
cargo run
```

**Log Levels:**

- `error`: Only errors (when things break)
- `warn`: Errors and warnings (when things might break)
- `info`: General information (default, reasonably quiet)
- `debug`: Detailed debugging info (chatty)
- `trace`: Extremely verbose (why would you do this to yourself?)

---

## 📊 API Reference

### Health Check (Does Literally Nothing)

```http
GET /health

Response: 200 OK
{
  "status": "ok"
}
```

This endpoint always returns success. It doesn't check if FFT is working, if memory is available, or if the universe is collapsing. It just says "ok" because it exists.

### Analyze Audio (The Actual Work Happens Here)

```http
POST /analyze
Content-Type: application/json

Request:
{
  "audio_data": "base64-encoded-pcm-samples",  // Your audio, but in base64
  "sample_rate": 48000,                        // How many samples per second
  "profile": "soprano"                         // Optional: which notes to prioritize
}

Response: 200 OK
{
  "notes": [
    {
      "note": "C4",
      "confidence": 0.95,    // How loud this frequency was (0-1)
      "intensity": 0.82      // Same thing but scaled differently
    }
  ],
  "sample_rate": 48000,
  "samples_analyzed": 96000,  // Usually sample_rate * 2
  "timestamp": 1697123456.789
}

Response: 400 Bad Request
{
  "error": "JSON parse error: ..."  // You sent garbage
}

Response: 500 Internal Server Error
{
  "error": "FFT analysis failed: ..."  // Something went wrong (rare)
}
```

**About the voice profiles:**

- `soprano|mezzo|alto|tenor|baritone|bass`: Filters to that voice range
- `no_profile` or omitted: Returns everything we find (chaos mode)

### Get Last Result (In Case You Forgot)

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
(Nothing analyzed yet, or the server just started)
```

This endpoint keeps exactly one result in memory. Not two, not zero (unless nothing's been analyzed). Just one. Because that's all we need.

### Audio Data Format (The Tedious Part)

**Audio must be:**

- PCM (Pulse Code Modulation) format – the raw, uncompressed kind
- 16-bit signed integers (i16) – values from -32768 to 32767
- Little-endian byte order – because that's what x86 uses
- Base64-encoded before transmission – because JSON can't handle binary data

**Why base64?** Because we're sending binary audio over JSON/HTTP, and base64 is the duct tape that makes it work. Is it efficient? No. Does it work? Yes. Could we use binary protocols? Sure, but then we'd have to explain protobuf to people.

**Example encoding in PowerShell (if you're into that sort of thing):**

```powershell
# Create sample audio (1 second of silence, thrilling stuff)
$sample_rate = 48000
$duration_sec = 1
$sample_count = $sample_rate * $duration_sec
$samples = @(0) * $sample_count

# Convert to bytes (i16 = 2 bytes per sample)
$bytes = New-Object byte[] ($sample_count * 2)
for ($i = 0; $i -lt $sample_count; $i++) {
    $sample_bytes = [BitConverter]::GetBytes([Int16]$samples[$i])
    [Array]::Copy($sample_bytes, 0, $bytes, $i * 2, 2)
}

# Encode to base64
$base64 = [Convert]::ToBase64String($bytes)
# Result: A very long string that represents absolute silence
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

# 3. Install cargo-make (one-time setup)
cargo install cargo-make

# 4. Verify installation
rustc --version
cargo --version
cargo make --version

# 5. Build project
cargo make

# 6. Run tests
cargo test --all
```

### Code Style

```powershell
# Format code (using cargo-make)
cargo make format

# Check formatting without modifying
cargo make format-check

# Format individual projects
cargo make format-backend
cargo make format-gui

# Lint with Clippy (aggressive settings)
cargo make clippy

# Lint individual projects
cargo make clippy-backend
cargo make clippy-gui

# Traditional cargo commands also work
cargo fmt --all
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
   # Quick check (fast)
   cargo make check
   
   # Run tests
   cargo test --all
   
   # Format and lint
   cargo make format
   cargo make clippy
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
# Build with cargo-make
cargo make

# Baseline measurement
cd tests/backend
python stress_test_backend_fast.py

# After changes
cd ..\..
cargo make
cd tests/backend
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

## 🎉 Enjoy RecogNotes

This project started as an experiment in Rust audio processing and somehow ended up with a two-process architecture, HTTP-based communication, and colorful bars. Does it work? Yeah, actually. Is it over-engineered? Absolutely. Would we have it any other way? Probably not.

Go forth and detect some notes. 🎵
