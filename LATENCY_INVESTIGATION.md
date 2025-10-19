# Performance Investigation: 2-Second Latency Root Cause Analysis

## Summary
The application has a ~2-second latency per audio analysis request. Investigation shows:
- ✓ Backend is NOT throttling (50 concurrent requests work consistently)
- ✓ FFT planner caching is implemented (global Lazy<Mutex> initialized)
- ✓ Confidence filtering is working (>50% gate on detections)
- ✓ GUI note display is correct (replaces instead of accumulates)
- ❓ **Bottleneck location UNKNOWN** - logs show analysis at 0ms but total is 2000ms

## Instrumentation Added

### Backend Timing (recognotes-rust-backend/src/)
1. **main.rs `analyze_audio()` endpoint**:
   - Logs: `REQUEST: bytes={}, analysis={}ms, convert={}us, serialize={}ms, TOTAL={}ms, notes={}`
   - Measures: JSON deserialization (implicit), audio analysis, Vec conversion, JSON serialization, total request time

2. **audio_analyzer.rs `compute_fft()` function**:
   - Logs: `compute_fft({}): lock={}us, plan={}us, convert={}us, process={}us, psd={}us`
   - Measures: Mutex lock time, FFT planning time, buffer conversion, FFT processing, PSD calculation

3. **audio_analyzer.rs `analyze_raw_bytes()` function**:
   - Logs: `analyze_raw_bytes: total={}ms, convert={}ms, analysis={}ms, filter={}ms`
   - Measures: Byte-to-sample conversion, FFT analysis chain, confidence filtering

### GUI Timing (recognotes-desktop-gui/src/main.rs)
1. **`continuous_analysis()` function**:
   - Logs client-side round-trip time: `Backend response in {}ms: {} notes from {}B audio`
   - Also logs errors with timing: `Backend error after {}ms: {}`

## To Reproduce the Issue

### Step 1: Start Backend with Logging
```powershell
cd recognotes-rust-backend
$env:RUST_LOG = "info,recognotes_rust_backend=debug"
.\target\release\recognotes-rust-backend.exe
```

### Step 2: Run Timing Test
```powershell
cd ..
python .\timing_test.py
```

### Step 3: Analyze Logs
Look for "REQUEST:" and "compute_fft:" lines to see:
- How long JSON deserialization takes
- How long analysis takes
- How long serialization takes
- Where the ~2000ms is spent

## Hypothesis Chain

### Hypothesis 1: JSON Serialization Bottleneck
- Client sends 960 integers as JSON: `{"audio_data": [127, 45, 89, ...], "sample_rate": 48000}`
- This creates a ~3-5KB JSON payload (10x larger than raw 960 bytes)
- Actix-web deserializes: Expensive with serde_json
- Server serializes response back: Another expensive JSON operation

**Evidence**: Consistent 2000ms regardless of load profile

**Test**: Check if `serialize_ms` is large in logs

### Hypothesis 2: Mutex Lock Contention
- Global FFT_PLANNER is protected by Mutex
- Even though we cache the planner, getting the lock might be slow
- Multiple concurrent requests queue on the same lock

**Evidence**: Stress test showed consistent 2050ms for all 50 concurrent requests (not scaling)

**Test**: Check if `lock={}us` is large in FFT logs

### Hypothesis 3: FFT Planning Overhead
- Even with caching, `planner.plan_fft_forward(480)` might be doing expensive work each time
- The planner might be re-planning for the same size over and over

**Evidence**: Unknown - need to check `plan={}us` in logs

**Test**: Add timing breakpoint before and after `plan_fft_forward()` call

### Hypothesis 4: System-Level Issue
- Tokio runtime overhead?
- Windows-specific thread scheduling?
- Network stack latency (even on localhost)?

**Evidence**: All measurements around 2000ms (very consistent, suggests not random)

**Test**: Run same test directly with Rust client, not Python HTTP client

## Expected Timeline

If we run the optimized backend with the timing logs we just added:

1. **Fast path (<100ms)**: Problem is likely network/serialization
   - Solution: Use binary format instead of JSON (MessagePack, bincode, etc.)

2. **Moderate path (500-1000ms)**: FFT planning is slow
   - Solution: Pre-plan FFTs for common sizes (480, 512, 1024)
   - Or use FFTPack or other library with pre-planned algorithms

3. **Slow path (1500-2000ms)**: Problem is more fundamental
   - Could be Actix-web configuration (workers, thread pool, etc.)
   - Could be Windows-specific (need profiler to investigate)

## Key Files Modified

- `recognotes-rust-backend/src/main.rs`: Added granular request timing
- `recognotes-rust-backend/src/audio_analyzer.rs`: Added FFT stage timing
- `recognotes-desktop-gui/src/main.rs`: Added client-side round-trip timing
- `timing_test.py`: New test to measure payload sizes and timings

## Next Steps

1. **RUN**: Start backend with RUST_LOG=info, run timing_test.py
2. **CAPTURE**: Copy backend server logs
3. **ANALYZE**: Look for which component takes ~2000ms
4. **OPTIMIZE**: Apply targeted fix based on findings
5. **VERIFY**: Re-run timing test to confirm improvement

## Build Commands

```powershell
# Build backend with optimizations
cd recognotes-rust-backend
cargo build --release

# Build GUI
cd ../recognotes-desktop-gui
cargo build --release
```

Both builds should complete successfully with no errors (only warnings).
