#!/usr/bin/env python3
"""
Test note detection across all human voice frequencies
Generates sine wave audio for each natural note (C2 to C6) and tests backend response
"""

import socket
import json
import struct
import math
import base64
import sys
import io

# Fix Unicode output on Windows
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

# Note frequencies for natural notes (C, D, E, F, G, A, B) in octaves 2-6
NOTE_FREQUENCIES = {
    "C2": 65.41,
    "D2": 73.42,
    "E2": 82.41,
    "F2": 87.31,
    "G2": 98.00,
    "A2": 110.00,
    "B2": 123.47,
    
    "C3": 130.81,
    "D3": 146.83,
    "E3": 164.81,
    "F3": 174.61,
    "G3": 196.00,
    "A3": 220.00,
    "B3": 246.94,
    
    "C4": 261.63,
    "D4": 293.66,
    "E4": 329.63,
    "F4": 349.23,
    "G4": 392.00,
    "A4": 440.00,
    "B4": 493.88,
    
    "C5": 523.25,
    "D5": 587.33,
    "E5": 659.25,
    "F5": 698.46,
    "G5": 783.99,
    "A5": 880.00,
    "B5": 987.77,
    
    "C6": 1046.50,
    "D6": 1174.66,
    "E6": 1318.51,
    "F6": 1396.91,
    "G6": 1567.98,
}

def generate_sine_wave(frequency: float, duration_ms: float = 500, sample_rate: int = 44100) -> bytes:
    """Generate a sine wave at the given frequency."""
    num_samples = int(sample_rate * duration_ms / 1000.0)
    audio_data = []
    
    for i in range(num_samples):
        # Generate sine wave
        sample = math.sin(2 * math.pi * frequency * i / sample_rate) * 0.8 * 32767
        int_sample = int(sample)
        # Clamp to 16-bit range
        int_sample = max(-32768, min(32767, int_sample))
        audio_data.append(int_sample)
    
    # Convert to bytes (little-endian 16-bit)
    audio_bytes = b''
    for sample in audio_data:
        audio_bytes += struct.pack('<h', sample)
    
    return audio_bytes

def send_analyze_request(host: str, port: int, audio_bytes: bytes, sample_rate: int) -> dict:
    """Send audio to backend and get analysis result."""
    try:
        # Encode audio as base64
        audio_b64 = base64.b64encode(audio_bytes).decode('utf-8')
        
        # Create JSON payload
        payload = json.dumps({
            'audio_data': audio_b64,
            'sample_rate': sample_rate
        })
        
        # Create socket and send request
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        
        # Build HTTP request
        body = payload
        headers = (
            f"POST /analyze HTTP/1.1\r\n"
            f"Host: {host}:{port}\r\n"
            f"Content-Type: application/json\r\n"
            f"Content-Length: {len(body)}\r\n"
            f"Connection: close\r\n"
            f"\r\n"
        )
        
        # Send request
        sock.sendall(headers.encode() + body.encode())
        
        # Receive response
        response = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            response += chunk
        
        sock.close()
        
        # Parse response
        response_str = response.decode('utf-8', errors='ignore')
        
        # Split headers and body
        if '\r\n\r\n' in response_str:
            body_start = response_str.index('\r\n\r\n') + 4
            body_content = response_str[body_start:]
        else:
            body_content = response_str
        
        # Parse JSON
        data = json.loads(body_content)
        return {'success': True, 'data': data}
        
    except Exception as e:
        return {'success': False, 'error': str(e)}

def main():
    host = "localhost"
    port = 5000
    sample_rate = 44100
    duration_ms = 500
    
    print("=" * 80)
    print("🎵 HUMAN VOICE NOTE FREQUENCY TEST")
    print("=" * 80)
    print(f"Backend: {host}:{port}")
    print(f"Duration: {duration_ms}ms, Sample Rate: {sample_rate} Hz")
    print()
    print("Testing natural notes (C, D, E, F, G, A, B) from octaves 2-6")
    print(f"Frequency range: C2 (65.41 Hz) to C6 (1046.50 Hz)")
    print()
    print("This covers all professional voice types:")
    print("  • Bass:           E2 to E4")
    print("  • Baritone:       A2 to A4")
    print("  • Tenor:          C3 to C5")
    print("  • Countertenor:   E3 to E5")
    print("  • Contralto:      F3 to E5")
    print("  • Mezzo Soprano:  A3 to A5")
    print("  • Soprano:        C4 to C6")
    print()
    print("-" * 80)
    
    results = []
    correct = 0
    total = 0
    
    for note_name in sorted(NOTE_FREQUENCIES.keys()):
        frequency = NOTE_FREQUENCIES[note_name]
        total += 1
        
        # Generate audio
        audio_bytes = generate_sine_wave(frequency, duration_ms, sample_rate)
        
        # Send to backend
        result = send_analyze_request(host, port, audio_bytes, sample_rate)
        
        if result['success']:
            data = result['data']
            notes = data.get('notes', [])
            
            if notes:
                detected_note = notes[0]['note']
                confidence = notes[0]['confidence']
                confidence_pct = int(confidence * 100)
                
                # Check if detected note matches input
                is_match = detected_note == note_name
                if is_match:
                    correct += 1
                    symbol = "✓"
                    color_start = "\033[92m"  # Green
                else:
                    symbol = "✗"
                    color_start = "\033[93m"  # Yellow
                
                color_end = "\033[0m"
                print(f"{symbol} {note_name:3s} ({frequency:7.2f} Hz) → {color_start}{detected_note:3s}{color_end} ({confidence_pct:3d}%)")
                
                results.append({
                    'input': note_name,
                    'frequency': frequency,
                    'detected': detected_note,
                    'confidence': confidence,
                    'match': is_match
                })
            else:
                print(f"✗ {note_name:3s} ({frequency:7.2f} Hz) → NO NOTES DETECTED")
                results.append({
                    'input': note_name,
                    'frequency': frequency,
                    'detected': 'NONE',
                    'confidence': 0,
                    'match': False
                })
        else:
            print(f"✗ {note_name:3s} ({frequency:7.2f} Hz) → ERROR: {result['error']}")
            results.append({
                'input': note_name,
                'frequency': frequency,
                'detected': 'ERROR',
                'confidence': 0,
                'match': False
            })
    
    print("-" * 80)
    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    
    accuracy = (correct / total) * 100 if total > 0 else 0
    accuracy_color = "\033[92m" if accuracy >= 80 else "\033[91m"  # Green if >= 80%, Red otherwise
    accuracy_end = "\033[0m"
    
    print(f"Correct: {correct}/{total} ({accuracy_color}{accuracy:.1f}%{accuracy_end})")
    print()
    
    # Show failures
    failures = [r for r in results if not r['match']]
    if failures:
        print(f"FAILURES ({len(failures)}):")
        print("-" * 80)
        for f in failures:
            print(f"  {f['input']:3s} ({f['frequency']:7.2f} Hz) detected as {f['detected']:3s}")
    else:
        print("🎉 ALL TESTS PASSED!")
    
    print()

if __name__ == '__main__':
    main()
