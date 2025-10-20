# Log Analysis Guide

## What to Look For

When running the timing test with backend logs enabled, you'll see messages like:

### Request-Level Timing
```
[2024-01-18T12:34:56.789Z INFO  recognotes_rust_backend] REQUEST: bytes=960, analysis=5ms, convert=23us, serialize=12ms, TOTAL=18ms, notes=1
```

**Key Values:**
- `bytes=960` - Input audio size
- `analysis=5ms` - FFT processing time
- `convert=23us` - Result conversion time
- `serialize=12ms` - JSON response generation
- `TOTAL=18ms` - **This is the main metric**
- `notes=1` - Number of notes detected

**What to expect:**
- Array format: `TOTAL=1800-2200ms` (slow JSON parsing)
- Base64 format: `TOTAL=50-200ms` (if hypothesis correct)

---

### FFT-Level Timing
```
[2024-01-18T12:34:56.790Z DEBUG recognotes_rust_backend] compute_fft(480): lock=125us, plan=234us, convert=45us, process=1234us, psd=89us
```

**Key Values:**
- `lock=125us` - Time to acquire Mutex for FFT planner
- `plan=234us` - Time to plan FFT algorithm
- `convert=45us` - Time to convert to complex numbers
- `process=1234us` - Actual FFT computation
- `psd=89us` - Power spectral density calculation

**What to expect:**
- Total: `lock + plan + convert + process + psd` = 1-2ms typically
- If `lock` is huge: Mutex contention problem
- If `process` is huge: FFT algorithm is slow (shouldn't be)
- If all small but REQUEST `TOTAL` is huge: Problem outside audio analysis

---

### Analysis-Level Timing
```
[2024-01-18T12:34:56.791Z DEBUG recognotes_rust_backend] analyze_raw_bytes: total=6ms, convert=1ms, analysis=4ms, filter=1ms
```

**Key Values:**
- `total=6ms` - Total analysis time
- `convert=1ms` - Convert bytes to samples
- `analysis=4ms` - Run FFT and peak finding
- `filter=1ms` - Confidence filtering

**What to expect:**
- All values very small (1-5ms each)
- If huge: Indicates slowness in audio processing (but stress test said it's fine!)

---

## Scenario Analysis

### Scenario 1: Array Format is Slow, Base64 is Fast ✅ (Hypothesis Correct)

**Array Format Logs:**
```
REQUEST: bytes=960, analysis=5ms, convert=23us, serialize=12ms, TOTAL=1950ms, notes=1
```

Problem: `TOTAL=1950ms` but sum of parts = 5+0.023+12 = 17ms
**Missing: 1933ms somewhere!**

This means Actix-web JSON parsing is taking ~1.9 seconds!

**Base64 Format Logs:**
```
REQUEST: bytes=960, analysis=5ms, convert=23us, serialize=12ms, TOTAL=45ms, notes=1
```

**Great!** Total now makes sense: 5 + 0.023 + 12 + 28 (base64 decode) = ~45ms

**Conclusion**: JSON integer array parsing was the bottleneck!

---

### Scenario 2: Both Are Still Slow ⚠️ (Hypothesis Wrong)

**Array Format Logs:**
```
REQUEST: bytes=960, analysis=5ms, convert=23us, serialize=12ms, TOTAL=2050ms, notes=1
```

**Base64 Format Logs:**
```
REQUEST: bytes=960, analysis=5ms, convert=23us, serialize=12ms, TOTAL=1950ms, notes=1
```

Problem persists with both formats. Missing ~2000ms in both cases.

**Possible causes:**
1. Actix-web framework overhead (worker threads, async scheduling)
2. Windows kernel-level delays
3. Something in HTTP stack (socket handling, network buffer management)
4. Tokio runtime overhead

**Next steps:**
- Compare with direct Rust client (no HTTP overhead)
- Profile with Windows performance tools
- Check if issue is architecture-dependent

---

### Scenario 3: Degradation with High Load 📊

If running stress_test.py and seeing:
- Single request: 50ms
- 10 sequential requests: 55ms each
- 50 concurrent requests: 2000ms each

**Interpretation:** Lock contention on FFT planner Mutex!

**Look for:** `lock=2500us` or higher in FFT logs during concurrent test

**Solution:** Need to optimize Mutex usage or pre-plan FFTs

---

## Example Complete Log Output

Here's what a successful test run would look like:

```
$ python timing_test.py

============================================================
⏱️  DETAILED TIMING TEST - JSON Array vs Base64
============================================================

📊 Audio Data:
  Duration: 10ms @ 48000Hz = 480 samples
  Raw bytes: 960 bytes (16-bit PCM)

📤 Format 1: JSON Array (Legacy - SLOW)
  Payload size: 3147 bytes
  Size increase: 3.3x
  First 100 chars: {"audio_data": [127, 45, 89, 200, 234, ...

📤 Format 2: Base64 (Recommended - FAST)
  Payload size: 1347 bytes
  Size decrease: 2.3x smaller than array format
  First 100 chars: {"audio_data": "f3+/vr+/v7//v/+/v/9/v/8...

🔌 Backend Connection: http://localhost:5000

📥 Test 1: Array Format (3 requests)
------
Request 1: 2045.3ms | 1 notes
Request 2: 2051.2ms | 1 notes
Request 3: 2048.7ms | 1 notes

📥 Test 2: Base64 Format (3 requests)
------
Request 1:  145.2ms | 1 notes
Request 2:  142.8ms | 1 notes
Request 3:  148.5ms | 1 notes

------
📊 RESULTS:

  Array Format:
    Avg: 2048.4ms
    Min: 2045.3ms
    Max: 2051.2ms

  Base64 Format:
    Avg: 145.5ms
    Min: 142.8ms
    Max: 148.5ms

  🚀 Base64 is 93.3% faster!
```

**Backend logs would show:**

```
Array format requests:
[INFO] REQUEST: bytes=960, analysis=6ms, convert=25us, serialize=11ms, TOTAL=2048ms, notes=1

Base64 format requests:
[INFO] REQUEST: bytes=960, analysis=6ms, convert=23us, serialize=11ms, TOTAL=145ms, notes=1

[DEBUG] compute_fft(480): lock=125us, plan=234us, convert=45us, process=1234us, psd=89us
```

**Key observation:** 
- Array: TOTAL (2048ms) >> parts sum (17ms) = JSON parsing bottleneck ✅
- Base64: TOTAL (145ms) ≈ parts sum (40ms) = expected overhead ✅

---

## Metrics to Track

As you run the test multiple times, track:

| Metric | Array | Base64 | Target |
|--------|-------|--------|--------|
| Avg latency (ms) | 2000+ | TBD | <100 |
| Min latency (ms) | 1900+ | TBD | <50 |
| Payload size (B) | 3200+ | 1300 | <1500 |
| Std deviation | <50ms | TBD | <20ms |

---

## How to Collect Logs

### Option 1: Capture to File
```powershell
$env:RUST_LOG = "info"
.\target\release\recognotes-rust-backend.exe > logs.txt 2>&1
```

Then review logs.txt after test runs.

### Option 2: Live View
```powershell
$env:RUST_LOG = "info"
.\target\release\recognotes-rust-backend.exe | Select-String "REQUEST|compute_fft|analyze_raw_bytes"
```

This filters to show only timing messages.

### Option 3: Debug Detail
```powershell
$env:RUST_LOG = "debug"
.\target\release\recognotes-rust-backend.exe 2>&1 | Tee-Object logs.txt
```

Shows even more detail (including all FFT analysis steps).

---

## Summary

**Best Case (Hypothesis Correct):**
- Array format: ~2000ms
- Base64 format: <150ms
- Improvement: 90%+ faster
- Conclusion: Problem was JSON parsing

**Worst Case (Hypothesis Wrong):**
- Array format: ~2000ms
- Base64 format: ~2000ms
- No improvement
- Conclusion: Problem is elsewhere in HTTP stack or framework

Either way, the detailed logs will tell us exactly what's happening and where to optimize next!
