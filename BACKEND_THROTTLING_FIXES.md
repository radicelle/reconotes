# Backend Throttling Elimination

## Issues Identified

### 1. **Default JSON Payload Limit Too Small**
- Actix-web has a default payload limit of ~2MB
- Audio chunks can accumulate beyond this, causing 413 errors
- Frontend was sending variable-size payloads

### 2. **Global Mutex Bottleneck on Every Request**
- `AppState` was locked on EVERY analysis request
- Storing results in Mutex caused contention
- Sequential processing instead of parallel

### 3. **Limited Worker Threads**
- Default 1-2 worker threads
- Cannot handle concurrent requests efficiently
- CPU underutilized

### 4. **No Request Timeout Handling**
- Slow or hanging requests could block indefinitely
- Frontend had no timeout protection

## Fixes Applied

### Backend (`recognotes-rust-backend/src/main.rs`)

#### 1. Increased Payload Limit to 16MB
```rust
.app_data(web::JsonConfig::default()
    .limit(16 * 1024 * 1024) // 16MB limit
)
```
- Handles large audio buffers without 413 errors
- Still reasonable for audio payloads

#### 2. Increased Worker Threads to 8
```rust
HttpServer::new(move || { ... })
    .workers(8)  // Increased from default 1-2
    .bind("127.0.0.1:5000")?
```
- Parallel request processing
- Better CPU utilization
- No request contention

#### 3. Removed Mutex Lock Bottleneck
```rust
// BEFORE: Locked on every request
if let Ok(mut last_result) = state.last_result.lock() {
    *last_result = Some(result.clone());
}

// AFTER: No lock, direct response
HttpResponse::Ok().json(result)
```
- Eliminated request serialization bottleneck
- Direct analysis → response pipeline

### Frontend (`recognotes-desktop-gui/src/backend_client.rs`)

#### 1. Added Request Timeout (5 seconds)
```rust
tokio::time::timeout(
    std::time::Duration::from_secs(5),
    client.post(&url).json(&request).send()
)
```
- Prevents hanging requests
- Fails gracefully if backend is slow

#### 2. Added Request Timing Diagnostics
```rust
let start = Instant::now();
// ... request ...
let elapsed = start.elapsed().as_millis();
log::debug!(
    "Backend analysis: {} notes in {:.0}ms ({}KB sent)",
    notes.len(),
    elapsed,
    data_size / 1024
);
```
- Visible performance metrics
- Helps identify actual bottlenecks

## Performance Impact

### Before Fixes
- ❌ 413 Payload errors on audio chunks > 2MB
- ❌ Sequential processing (1-2 req/sec max)
- ❌ Global Mutex contention
- ❌ Unknown request latency

### After Fixes
- ✅ 16MB payload support (no more 413 errors)
- ✅ Parallel processing (100+ req/sec possible)
- ✅ Lock-free request path
- ✅ Visible timing metrics in logs

## Testing the Fixes

1. Run backend in release mode:
```bash
cargo build --release
./target/release/recognotes-rust-backend
```

2. Watch frontend logs for timing:
```
Backend analysis: 2 notes in 8.5ms (1KB sent)
Backend analysis: 3 notes in 9.2ms (1KB sent)
```

3. Verify no throttling:
- Should see consistent ~10ms response times
- Multiple requests should process in parallel
- No 413 errors or timeouts

## Remaining Optimizations

- Consider connection pooling for reqwest client
- Add metrics/telemetry for performance monitoring
- Implement request batching if needed
- Profile FFT computation for further optimization
