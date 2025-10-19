#!/usr/bin/env python3
import requests
import base64
import wave

# Load WAV file
with wave.open('test_baritone_wav.wav', 'rb') as wf:
    sample_rate = wf.getframerate()
    frames = wf.readframes(sample_rate)  # Read exactly 1 second

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
