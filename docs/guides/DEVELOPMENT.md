# RecogNotes Rust Backend - Development Guide

## Project Overview

This is a simplified Rust implementation of the RecogNotes backend. The original Python backend is a Flask server that:
1. Receives audio data from the frontend
2. Analyzes it to detect musical notes
3. Generates music sheets as PDFs

## Current Implementation

This is a **very simple version** (v0.1.0) that provides:
- REST API server using **Actix-web** framework
- Three basic endpoints for demonstration
- Mock audio analysis (returns sample notes instead of real processing)

### Project Structure

```
recognotes-rust-backend/
├── Cargo.toml           # Project manifest with dependencies
├── Cargo.lock          # Locked dependency versions
├── README.md           # User guide
├── DEVELOPMENT.md      # This file
├── src/
│   └── main.rs        # Main application code
└── target/            # Build artifacts
```

## Running the Server

```bash
cd recognotes-rust-backend
cargo run
```

The server will start on `http://127.0.0.1:5000`

## API Endpoints

### 1. Health Check
```bash
curl http://localhost:5000/health
```
**Response:**
```json
{"status": "ok"}
```

### 2. Analyze Audio (POST)
```bash
curl -X POST http://localhost:5000/analyze \
  -H "Content-Type: application/json" \
  -d '{"audio_data": [], "sample_rate": 44100}'
```
**Response:**
```json
{
  "notes": ["C4", "E4", "G4"],
  "frequencies": [262.0, 330.0, 392.0],
  "confidence": 0.95
}
```

### 3. Get Last Result
```bash
curl http://localhost:5000/last-result
```
**Response:** Returns the last analysis result or 204 No Content if none exists.

## Enhancement Roadmap

### Phase 1: Real Audio Processing
- [ ] Integrate `hound` crate for WAV file handling
- [ ] Integrate `rustfft` for Fast Fourier Transform (FFT)
- [ ] Implement basic pitch detection algorithm
- [ ] Parse audio samples and extract frequencies

### Phase 2: Note Detection
- [ ] Map frequencies to musical notes
- [ ] Implement confidence scoring
- [ ] Handle multiple concurrent notes (polyphony detection)
- [ ] Add time-based note sequencing

### Phase 3: Web Integration
- [ ] Add CORS support for frontend
- [ ] Implement proper file upload handling
- [ ] Add async file processing
- [ ] Implement progress tracking for long operations
- [ ] Add error responses with detailed messages

### Phase 4: Music Sheet Generation
- [ ] Integrate with LilyPond or music notation library
- [ ] Generate PDF output
- [ ] Add metronome support
- [ ] Create MusicXML export option

### Phase 5: Production Ready
- [ ] Add comprehensive error handling
- [ ] Implement request validation
- [ ] Add logging and monitoring
- [ ] Create configuration file support
- [ ] Add rate limiting and security headers
- [ ] Optimize for performance (memory pooling, etc.)
- [ ] Add unit and integration tests
- [ ] Create Docker containerization

## Dependencies

- **actix-web**: High-performance web framework
- **serde/serde_json**: JSON serialization/deserialization
- **tokio**: Async runtime
- **log/env_logger**: Logging framework
- **bytes**: Byte manipulation utilities

### Future Dependencies to Add

```toml
# Audio processing
hound = "3.5"          # WAV file I/O
rustfft = "6.0"        # Fast Fourier Transform
apodize = "1.0"        # Window functions for FFT

# Music notation
lyricist = "0.1"       # LilyPond bindings (or similar)

# Error handling
anyhow = "1.0"         # Error handling
thiserror = "1.0"      # Custom error types

# Testing
tempfile = "3.0"       # Temp files for testing
mockito = "1.0"        # HTTP mocking
```

## Rust-Specific Design Notes

### Strengths of Rust for This Project
- **Memory Safety**: No buffer overflows or null pointer issues
- **Performance**: Near-C performance for audio processing
- **Concurrency**: Excellent async/await support for handling multiple requests
- **Type System**: Strong typing prevents many bugs at compile time

### Architecture Decisions

1. **State Management**: Using `web::Data<AppState>` with `Mutex` for thread-safe state
2. **Async Runtime**: Tokio-based for non-blocking I/O
3. **Error Handling**: Will need `Result` types with custom error types
4. **Serialization**: Serde provides zero-copy JSON handling

## Building for Production

```bash
# Release build with optimizations
cargo build --release

# Binary location
./target/release/recognotes-rust-backend

# Run with specific settings
RUST_LOG=info ./target/release/recognotes-rust-backend
```

## Testing

```bash
# Run tests (when added)
cargo test

# Check code for errors without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Comparison with Original Python Backend

| Aspect | Python (Original) | Rust (This) |
|--------|------------------|-----------|
| Framework | Flask | Actix-web |
| Performance | Moderate | Very Fast |
| Memory Usage | Higher | Very Low |
| Development Speed | Fast | Moderate |
| Type Safety | Duck typing | Strong typing |
| Concurrency | GIL limitations | Excellent |
| Startup Time | ~1-2s | <100ms |

## Getting Started with Rust Audio Processing

Recommended resources:
- [rustfft Documentation](https://docs.rs/rustfft/)
- [hound - WAV I/O](https://docs.rs/hound/)
- [Introduction to DSP in Rust](https://github.com/nwhsiao/dsp-in-rust)

## Contributing

When adding features:
1. Keep the code modular
2. Write tests alongside features
3. Follow Rust naming conventions
4. Use `cargo clippy` for linting
5. Document public APIs with doc comments

## License

MIT
