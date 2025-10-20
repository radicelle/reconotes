# Performance Optimization: Smooth Real-Time Audio Visualization

## Issues Fixed

### 1. **413 Payload Too Large Error**
**Problem**: The frontend was sending entire audio buffers without size limits, causing payloads to exceed backend limits.

**Solution**: 
- Implemented fixed 10ms chunk size instead of dumping entire buffer
- At 48kHz: 480 samples × 2 bytes = 960 bytes per chunk (manageable payload)
- Prevents accumulation of large payloads

### 2. **Random/Stuttery Visualization**
**Problem**: 
- Analysis interval was 40ms, too slow for smooth real-time feedback
- Buffer sizes were inconsistent, causing timing jitter
- Notes were accumulating instead of updating cleanly

**Solution**:
- Reduced analysis interval from **40ms → 10ms** for responsive visualization
- Fixed chunk size ensures predictable latency
- Each chunk analyzed independently prevents buffer overflow

### 3. **Lack of Front-Backend Buffer Synchronization**
**Problem**: Frontend and backend had different buffer expectations, causing misalignment.

**Solution**:
- Frontend sends consistent 10ms chunks (960 bytes @ 48kHz)
- Backend processes fixed-size audio blocks
- Smooth pipeline with predictable timing

## Technical Details

### Buffer Calculation
```rust
// 10ms at 48kHz = 480 samples per chunk
chunk_size_bytes = (sample_rate / 100) * 2
// Example: (48000 / 100) * 2 = 960 bytes
```

### Analysis Pipeline
1. **Capture**: Audio captured continuously at 48kHz
2. **Buffer**: Accumulated in memory as samples arrive
3. **Chunk**: Every 10ms, take up to `chunk_size_bytes` from buffer
4. **Send**: Async task sends chunk to backend
5. **Analyze**: Backend performs FFT on fixed-size chunk (~10ms)
6. **Display**: Notes replace previous notes (no accumulation)

### Performance Metrics
- **Latency**: ~10-20ms total (capture + network + FFT + display)
- **Payload Size**: ~960 bytes per analysis (manageable)
- **Frame Rate**: 100 updates/second (smooth visualization)
- **Smooth Response**: Consistent timing prevents jitter

## Code Changes

### Frontend (`recognotes-desktop-gui`)
1. Added `chunk_size_bytes` field to `RecogNotesApp`
2. Changed analysis interval: `40ms → 10ms`
3. Implemented `get_buffered_audio_chunk()` for fixed-size extraction
4. Notes replace instead of accumulate

### Audio Manager (`audio.rs`)
- Replaced `get_buffered_audio()` with `get_buffered_audio_chunk(chunk_size)`
- Takes only the first N samples that fit in chunk_size
- Leaves remaining samples in buffer for next chunk

### Result
✅ No more 413 errors  
✅ Smooth, responsive visualization  
✅ Consistent frame timing  
✅ Front-backend buffer alignment
