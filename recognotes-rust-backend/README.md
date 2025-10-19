# RecogNotes Rust Backend

A simple Rust implementation of the RecogNotes audio analysis backend. This is a simplified version of the original Python Flask backend that detects musical notes from audio input.

## Features

- REST API endpoints for audio analysis
- Health check endpoint
- Simple note detection (mock implementation for demonstration)
- JSON-based API responses

## API Endpoints

### Health Check
```
GET /health
```
Returns server status.

### Analyze Audio
```
POST /analyze
Content-Type: application/json

{
  "audio_data": [array of bytes],
  "sample_rate": 44100
}
```
Analyzes audio data and returns detected notes with frequencies and confidence score.

### Get Last Result
```
GET /last-result
```
Returns the last analysis result.

## Building

Make sure you have Rust installed. If not, install it from https://rustup.rs/

```bash
cargo build --release
```

## Running

```bash
cargo run
```

The server will start on `http://127.0.0.1:5000`

## Testing

Test the health endpoint:
```bash
curl http://localhost:5000/health
```

Test the analyze endpoint:
```bash
curl -X POST http://localhost:5000/analyze \
  -H "Content-Type: application/json" \
  -d '{"audio_data": [], "sample_rate": 44100}'
```

Get last result:
```bash
curl http://localhost:5000/last-result
```

## Next Steps

- Integrate actual audio processing library (e.g., `hound` for WAV files)
- Implement real note detection algorithm
- Add pitch detection using FFT or similar techniques
- Connect to music sheet generation
- Add CORS support for frontend integration
- Implement proper error handling
- Add configuration management

## License

MIT
