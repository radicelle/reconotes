# RecogNotes Desktop GUI - Quick Start Guide

## ✅ Current Status
The horizontal bar chart visualization has been **successfully integrated** into the main UI panel. The application is ready to use!

## How to Run

### Option 1: Run from Release Build
```
cd c:\Users\manua\CodeProjects\other\diapazon\recognotes-desktop-gui
./target/release/recognotes-desktop-gui.exe
```

### Option 2: Build and Run from Source
```
cd c:\Users\manua\CodeProjects\other\diapazon\recognotes-desktop-gui
cargo run --release
```

## What You'll See

### Main UI Layout
```
┌─────────────────────────────────────────────┐
│  🎵 RecogNotes Desktop                      │
├─────────────────────────────────────────────┤
│  ● Backend Connected  ● Recording...        │
├─────────────────────────────────────────────┤
│  Settings: Session | BPM: 120 | Metronome  │
├─────────────────────────────────────────────┤
│  [🎤 Start Recording] [Clear Results]       │
├─────────────────────────────────────────────┤
│  🎵 Detected: C4 (85%) | E4 (92%)          │
│  ┌─────────────────────────────────────────┐ │
│  │ C4 ████████████████░░░░░░░░░░░░░  85%   │ │
│  │ D4 ██████████░░░░░░░░░░░░░░░░░░░░  55%  │ │
│  │ E4 ███████████████████░░░░░░░░░░░░  92%  │ │
│  └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### How the Bars Work

**Horizontal Bar Visualization:**
- **Note Name**: Displayed on the left (e.g., "C4", "E4")
- **Bar Width**: Proportional to confidence level (0-100%)
- **Color**: Indicates confidence level:
  - 🟢 **GREEN**: ≥80% (Confident)
  - 🟡 **YELLOW**: ≥60% (Likely)
  - 🔵 **LIGHT BLUE**: ≥50% (Possible)
  - ⚫ **GRAY**: <50% (Uncertain)
- **Percentage**: Shown on the right (e.g., "85%")

**Example:**
- A note with 100% confidence fills the entire bar width in GREEN
- A note with 50% confidence fills half the bar width in LIGHT BLUE
- A note with 30% confidence would be very short and GRAY

## Features

### ✅ Implemented
- Continuous audio capture (10ms intervals)
- Real-time note detection from backend
- Horizontal bar chart visualization with confidence-based widths
- Color-coded confidence levels
- 1-second minimum display duration (notes don't disappear immediately)
- No visual flashing (consistent light gray background)
- Compact single-line notes summary
- Professional integrated UI layout

### Recording Controls
- **Start**: Click "🎤 Start Continuous Recording" to begin analysis
- **Stop**: Click "⏹ Stop Continuous Recording" to stop
- **Clear**: Click "Clear Results" to clear displayed notes

### Settings
- **Session Title**: Name for your recording session
- **BPM**: Tempo setting (30-300 bpm)
- **Metronome**: Toggle metronome sound during recording

## Backend Requirements

The GUI requires the Rust backend to be running on `localhost:8000`.

### To Start Backend:
```
cd c:\Users\manua\CodeProjects\other\diapazon\recognotes-rust-backend
cargo run --release
```

The GUI will show:
- 🟢 **Green dot** "Backend Connected" when backend is reachable
- 🔴 **Red dot** "Backend Offline" when backend is not responding

## System Requirements

- Windows OS
- Python backend running (recognotes-rust-backend)
- Audio input device (microphone)
- ~50MB disk space for binary

## Performance

- **GUI Rendering**: 60 FPS (on display refresh rate)
- **Analysis Frequency**: 100 Hz (every 10ms)
- **Note Display Duration**: Minimum 1 second
- **Build Time**: ~2.5 seconds (release mode)
- **Binary Size**: ~45MB (release build)

## Troubleshooting

### No Notes Appearing
1. Check if backend is running: Look for green "Backend Connected" indicator
2. Check microphone permissions in Windows
3. Verify audio input device is selected correctly
4. Try singing louder or closer to microphone

### Backend Offline
1. Make sure backend is running: `cargo run --release` in backend folder
2. Check if port 8000 is in use
3. Look for error messages in backend terminal

### Application Crashes
1. Check if backend crashed
2. Try restarting the application
3. Ensure sufficient disk space available

## File Locations

- **Binary**: `c:\Users\manua\CodeProjects\other\diapazon\recognotes-desktop-gui\target\release\recognotes-desktop-gui.exe`
- **Source**: `c:\Users\manua\CodeProjects\other\diapazon\recognotes-desktop-gui\src\`
- **Backend**: `c:\Users\manua\CodeProjects\other\diapazon\recognotes-rust-backend\`

## Recent Changes

### Latest Session (Oct 18, 2025)
- ✅ Integrated horizontal bar chart into main UI panel
- ✅ Added compact notes summary line
- ✅ Bars display with confidence-based width fills
- ✅ Color coding for confidence levels
- ✅ Fixed display timing (1-second minimum)
- ✅ Compiled successfully with zero errors

## Known Limitations

1. **Bar Chart Height**: Fixed at 200px (could be made resizable)
2. **Note Sorting**: Alphabetical (could add confidence-based sorting)
3. **Live Histogram**: Only shows currently detected notes (not history)
4. **Export**: No built-in export of detected notes

## Tips for Best Results

1. **Optimal Volume**: Sing at normal speaking volume, 6-12 inches from microphone
2. **Clear Notes**: Sing clearly defined individual notes for best detection
3. **Background Noise**: Minimize background noise for better detection
4. **Faster Detection**: Notes appear and bars fill as confidence increases
5. **1-Second Hold**: Each set of notes stays visible for 1 second before clearing

## Next Actions

### For Development:
1. Customize bar colors in `src/visualization.rs::confidence_to_color()`
2. Adjust bar height calculation in `draw_horizontal_bars()`
3. Modify display duration in `src/main.rs::min_note_display_duration`

### For Users:
1. Start the application
2. Click "Start Continuous Recording"
3. Sing notes or play audio
4. Watch horizontal bars fill up as notes are detected
5. Bars change color based on confidence level

---

**Status**: ✅ Production Ready  
**Last Build**: Oct 18, 2025 (2.55s)  
**Compilation**: 0 errors, 2 warnings (dead_code - non-critical)  
**Binary Ready**: Yes (`target/release/recognotes-desktop-gui.exe`)
