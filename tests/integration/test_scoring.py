#!/usr/bin/env python3
import requests
import base64
import wave
import os

# Load WAV file - Use 'O grave.wav' from test fixtures
# Falls back to generating audio if file not found
test_audio_path = os.path.join(os.path.dirname(__file__), 'O grave.wav')

if not os.path.exists(test_audio_path):
    print(f"Warning: {test_audio_path} not found. Make sure to run from project root.")
    print("Using: python tests/integration/test_scoring.py")
    test_audio_path = 'O grave.wav'

try:
    with wave.open(test_audio_path, 'rb') as wf:
        sample_rate = wf.getframerate()
        frames = wf.readframes(sample_rate)  # Read exactly 1 second
except FileNotFoundError:
    print(f"Error: {test_audio_path} not found!")
    print("Available audio files in tests/integration/:")
    print("  - O grave.wav")
    print("  - O grave.m4a")
    exit(1)

# Encode and send
payload = {
    'audio_data': base64.b64encode(frames).decode('utf-8'),
    'sample_rate': sample_rate
}

print("Sending 1-second chunk (chunk 1) to backend...")
response = requests.post('http://localhost:5000/analyze', json=payload)
result = response.json()

print("\n✅ CHUNK 1 (0.00-1.00s) - Notes returned by backend (in order):")
for i, note in enumerate(result['notes'], 1):
    print(f"  {i}. {note['note']}: conf={note['confidence']:.1%}, intensity={note['intensity']:.1%}")

print("\n📊 Analysis samples:", result['samples_analyzed'])
