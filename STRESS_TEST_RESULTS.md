# Backend Stress Test Results - Analysis Report

## Executive Summary

✅ **BACKEND IS STABLE** - No throttling detected
- ✓ All 50 concurrent requests succeeded (100% success rate)
- ✓ Response times consistent: ~2050ms average
- ✓ No variance indicating throttling
- ✓ Works well under parallel load

⚠️ **IMPORTANT FINDING**: Response time is ~2 seconds per request

---

## Test Results

### Test 1: Health Check
```
Status: ✓ OK
Time: 2058.6ms
Response: Version 0.2.0-fft
```

### Test 2: Single Request
```
Status: ✓ OK
Time: 2062.5ms
Notes: 1 (G4 at 82% confidence)
```

### Test 3: Sequential Requests (10 requests, 100ms apart)
```
Average: 2069.5ms
Min: 2053.5ms
Max: 2083.6ms
Success Rate: 10/10 (100%)
```
**Observation**: Very consistent timing, no degradation with multiple requests

### Test 4: Concurrent Requests (50 parallel, 10 workers)
```
Average: 2051.6ms
Min: 2043.1ms
Max: 2071.4ms
StdDev: 4.8ms
Success Rate: 50/50 (100%)
```
**Observation**: Excellent - even under heavy concurrent load, responses are consistent

### Test 5: Rapid-Fire Test (20 requests as fast as possible)
```
Total Time: 41092.4ms for 20 requests
Throughput: 0.5 requests/second
Average: 2054.0ms per request
Min: 2036.4ms
Max: 2074.8ms
```
**Observation**: No throttling detected - variance is only 38ms (1.8% of average)

---

## Key Findings

### ✅ Backend Performance
- **No throttling**: All requests processed successfully
- **Consistent response time**: ±1.8% variance even under stress
- **High concurrency**: Handles 50 parallel requests without degradation
- **Low variance**: Standard deviation of only 4.8ms across 50 requests

### ⚠️ Critical Issue Identified
**Response time is ~2 seconds (2000-2100ms) per request**

This explains your timing issues! Let me break down why:

```
User sings note at T=0ms
  ↓ +15ms: Audio reaches cpal
  ↓ +10ms: GUI buffer accumulates  
  ↓ +50ms: HTTP request sent to backend
  ↓ +2050ms: ← BACKEND PROCESSING (BOTTLENECK!)
  ↓ +50ms: Response returns to GUI
  ────────────────────────────
  Total: ~2200ms latency (2.2 seconds!)
```

### Root Cause Analysis

The 2-second latency is likely due to:

1. **FFT computation** on small chunks (480 samples)
   - Current: 480 samples = 10ms of audio
   - FFT needs larger windows for accuracy
   - But increasing chunk size increases latency further

2. **Possible issues in audio_analyzer.rs**:
   - Hann window calculation
   - FFT padding or computation
   - Peak detection algorithm
   - Frequency-to-note lookup

3. **HTTP overhead** (should be minimal ~50ms)

---

## Why This Affects You

### Before (Expected):
```
Real-time singing: C D E F G
Expected display lag: 50-100ms (acceptable for audio)
Actual display lag: 2000ms+ (way too much!)
```

### After these fixes:
```
Notes clear on each response: ✓ Fixed in this update
Backend throttling: ✓ NOT present (good news)
Latency issue: ⚠️ Still present (needs deeper investigation)
```

---

## Recommendations

### Priority 1: Profile the Backend
You need to measure WHERE the 2 seconds is spent:

```rust
// Add timing breakpoints in audio_analyzer.rs
let start = Instant::now();
let windowed = self.apply_hann_window(audio_data);
eprintln!("Window: {}ms", start.elapsed().as_millis());

let start = Instant::now();
let psd = self.compute_fft(&windowed, sample_rate);
eprintln!("FFT: {}ms", start.elapsed().as_millis());

let start = Instant::now();
let peaks = self.find_all_peaks(&psd, sample_rate, audio_data.len());
eprintln!("Peaks: {}ms", start.elapsed().as_millis());
```

### Priority 2: Check FFT Implementation
- Is FFT being called multiple times unnecessarily?
- Are we using the right FFT size?
- Is the lookup table being recomputed each time?

### Priority 3: Optimize Request Path
- Consider sending **more audio** per request (e.g., 2048 samples instead of 480)
- This amortizes the fixed overhead across more samples
- Trade-off: higher latency per request, but more efficient

### Priority 4: Consider Local Analysis
For even lower latency:
- Run basic pitch detection on GUI (rough estimate)
- Send to backend for verification only
- Would reduce round-trips

---

## Test Environment
- Backend: Rust (Actix-web framework)
- Test Method: Python concurrent requests
- Audio: 960 bytes (480 samples @ 16-bit, 48kHz) = 10ms duration
- Payload: JSON with base64-encoded audio

---

## Conclusions

### ✅ What's Good
1. Backend is NOT throttling - it's processing all requests
2. Concurrency works well - no blocking issues
3. Response consistency is excellent - very predictable
4. Actix-web framework is handling load well

### ⚠️ What Needs Attention
1. **2-second latency is the real problem** - not throttling
2. Need to profile FFT computation to find bottleneck
3. Consider optimizing chunk size or FFT window
4. May need to investigate if FFT library is slow or if algorithm is inefficient

### Next Steps
1. Add timing logs to backend (`eprintln!` at each stage)
2. Run backend with these logs to identify bottleneck
3. Verify FFT isn't being called redundantly
4. Check if we can batch multiple chunks or use larger windows

**This is good news**: The backend infrastructure is solid. The issue is algorithmic/performance, not architectural.
