# Testing Guide - Verify All Three Fixes

## Quick Start

1. **Start the backend**:
```powershell
cd c:\Users\manua\CodeProjects\other\diapazon\recognotes-rust-backend
.\target\release\recognotes-rust-backend.exe
```

2. **Start the GUI** (in new terminal):
```powershell
cd c:\Users\manua\CodeProjects\other\diapazon\recognotes-desktop-gui
.\target\debug\recognotes-desktop-gui.exe
```

3. **You should see**: 
   - Backend: "Starting RecogNotes Rust Backend on http://127.0.0.1:5000"
   - GUI: Window opens with "● Backend Connected" in green

---

## Test Case 1: Multiple Notes Accumulate (Issue #1 Fix)

### Setup
- Backend running
- GUI running
- Both showing "Connected"

### Test Steps
1. Click "🎤 Start Continuous Recording"
2. Sing a scale slowly: **C D E F G** (or hummed equivalent)
   - Sing each note for ~1 second
   - Clear, distinct notes
3. Observe GUI display

### Expected Results ✓
- See all 5 notes accumulate in list:
```
📝 Detected 5 Note(s) - Last 3 Seconds

[1] 🎵 C - 85% confidence
[2] 🎵 D - 78% confidence
[3] 🎵 E - 82% confidence
[4] 🎵 F - 76% confidence
[5] 🎵 G - 81% confidence
```

### NOT Expected ✗ (Would indicate Issue #1 NOT fixed)
- Only one note showing at a time
- Notes disappearing immediately
- Always showing just the last note

---

## Test Case 2: Confidence Color Coding (Bonus Feature)

### Setup
- Same as above (notes visible)

### Expected Results ✓
- **Green text** for high confidence (≥80%): C E G
- **Yellow text** for medium confidence (60-79%): D F
- **Blue text** for low confidence (<60%): (probably none if singing well)

### What This Tells You
- Green = Definitely correct, very reliable
- Yellow = Probably correct, could be a partial detection
- Blue = Uncertain, might be partial or harmonic

---

## Test Case 3: No Ghost Notes on Silence (Issue #3 Fix)

### Setup
- GUI running
- Backend running

### Test Steps
1. Click "🎤 Start Continuous Recording"
2. **Do NOT sing** - stay completely silent
3. Wait 5-10 seconds
4. Watch for any notes appearing

### Expected Results ✓
- **Empty display** with no notes listed
- OR at most 1-2 low-confidence notes that disappear quickly
- No continuous stream of random "noise notes"

### NOT Expected ✗ (Would indicate Issue #3 NOT fixed)
- Random notes appearing like: B2, G#3, F5, etc.
- Notes appearing every few seconds even during silence
- Many notes with confidence 20-40%

---

## Test Case 4: Sing After Silence

### Setup
- Just finished Test Case 3 (GUI showing empty)
- Backend and GUI still running

### Test Steps
1. Stay silent for 2 seconds
2. Sing one clear note (e.g., "Ahhh" on middle C)
3. Hold it for 2 seconds
4. Observe when note appears and disappears

### Expected Results ✓
- Note appears in GUI shortly after you start singing (~100-150ms lag)
- Note shows with high confidence (80%+) in GREEN
- Note stays visible while you're singing
- Note disappears shortly after you stop

### What This Tests
- Backend is actively analyzing
- Confidence filter is working (real notes show, noise doesn't)
- History system is working (note appears then disappears after 3 seconds)

---

## Test Case 5: Continuous Stream (Real Usage)

### Setup
- Backend and GUI running

### Test Steps
1. Click "🎤 Start Continuous Recording"
2. Sing a phrase naturally (e.g., "Mary had a little lamb")
3. Keep singing for 10-15 seconds
4. Stop and observe what's shown in GUI

### Expected Results ✓
- Notes accumulate as you sing
- Should see 8-15 notes in GUI (depending on how long you sang)
- All notes visible with numbering [1] [2] [3] ...
- When you stop, notes persist for ~3 seconds then disappear

### Performance Check
- GUI should remain responsive
- No freezing or lag
- Backend processing should be fast enough to show notes

---

## Debug Logging - What to Look For

### In Backend Console
```
[Expected Good Output]
INFO: Starting RecogNotes Rust Backend on http://127.0.0.1:5000
INFO: Audio analysis with FFT-based pitch detection enabled

[When GUI Analyzes]
DEBUG: POST /analyze - "200 OK" (response logged)
```

### In GUI Console
```
[Expected Good Output]
INFO: 🎵 Received 2 notes from backend
INFO:    - C4 (85% confidence)
INFO:    - E4 (78% confidence)
INFO: 📝 History now contains 5 notes (showing 5 in UI)
```

### NOT Expected ✗
```
[Bad Output - High False Positive Rate]
DEBUG: Backend response received with notes
INFO: 🎵 Received 1 notes from backend
INFO:    - C4 (35% confidence) ← TOO LOW, should be filtered

[Bad Output - Network Issue]
ERROR: Backend error: request timeout
ERROR: Failed to connect to backend
```

---

## Common Issues and Troubleshooting

### Issue: "Backend Offline" shown in GUI

**Solution:**
1. Check backend is running in terminal
2. Verify backend console shows "http://127.0.0.1:5000"
3. Try clicking "Check Backend Connection" button in GUI
4. If still offline, backend may have crashed

### Issue: No notes appear when singing clearly

**Possible Causes:**
1. Microphone not working - test in system settings
2. Audio level too low - sing louder or check mic input level
3. Backend processing too slow - check backend console for errors
4. Wrong sample rate mismatch - backend and GUI should both use 48kHz

**Debugging:**
- Check terminal logs for errors
- Try singing into same mic with other audio app to verify it works

### Issue: Too many low-confidence notes appearing

**This might be:**
- Normal with quiet singing (system is being conservative)
- Try singing louder and more clearly
- Low-confidence notes (60%+) should disappear after 3 seconds

### Issue: Timing seems very laggy (>500ms)

**Possible Causes:**
1. Network latency spikes - check if backend is on same computer
2. Backend overloaded - check CPU usage
3. GUI processing backed up - check if GUI is slow

**Note:** 50-150ms latency is normal and expected for network audio.

---

## Success Criteria - All Three Issues Fixed

✓ **Issue #1 Fixed**: 
- Multiple notes visible at once
- Notes accumulate in rolling 3-second window
- Each note numbered [1] [2] [3] etc

✓ **Issue #2 Improved**: 
- Timestamp infrastructure in place
- Foundation for future optimization
- (50-150ms lag is acceptable)

✓ **Issue #3 Fixed**: 
- No ghost notes during silence
- Confidence > 50% filter eliminates noise
- Only singing produces notes in UI

---

## Quick Verification Script

After implementing fixes, run this quick test:

1. Start backend
2. Start GUI
3. Sing "Do Re Mi" (3 notes)
4. **Check**: See [1] [2] [3] with colors?
5. Stop singing, wait 1 second
6. **Check**: Notes still visible? (3-second history)
7. Wait 3 more seconds silently
8. **Check**: Notes disappeared? (history expired)
9. Stay silent for 5 more seconds
10. **Check**: No random notes appeared?

If all 10 checks pass → **All three issues fixed! ✅**

---

## Performance Baseline

After fixes, you should see:
- Backend: ~5ms per analysis
- GUI: Responsive with <100ms display latency
- Memory: Stable (no continuous growth)
- CPU: Low usage on both processes

If you see:
- Analysis >100ms → Backend might be overloaded
- Memory growing → Possible memory leak in history accumulation
- CPU maxed → Too many analyzes happening
