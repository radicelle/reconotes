#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Detailed timing test to measure client round-trip and server performance
Tests both JSON array format and base64 format to compare performance
"""

import json
import requests
import time
import struct
import math
import sys
import io
import base64

# Fix Unicode output on Windows
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

def generate_test_audio(sample_rate: int = 48000, duration_ms: float = 10.0, frequency: float = 440.0) -> bytes:
    """Generate a sine wave audio sample."""
    num_samples = int(sample_rate * duration_ms / 1000.0)
    audio_data = []
    
    for i in range(num_samples):
        # Generate 440 Hz sine wave
        sample = math.sin(2 * math.pi * frequency * i / sample_rate) * 32767
        int_sample = int(sample)
        audio_data.append(int_sample)
    
    # Convert to bytes (little-endian 16-bit)
    audio_bytes = b''
    for sample in audio_data:
        audio_bytes += struct.pack('<h', sample)
    
    return audio_bytes

def main():
    backend_url = "http://localhost:5000"
    sample_rate = 48000
    
    print("=" * 90)
    print("⏱️  DETAILED TIMING TEST - JSON Array vs Base64")
    print("=" * 90)
    
    # Generate audio
    audio_bytes = generate_test_audio(sample_rate, duration_ms=10.0, frequency=440.0)
    print(f"\n📊 Audio Data:")
    print(f"  Duration: 10ms @ {sample_rate}Hz = 480 samples")
    print(f"  Raw bytes: {len(audio_bytes)} bytes (16-bit PCM)")
    
    # Format 1: JSON array (legacy, slow)
    print(f"\n📤 Format 1: JSON Array (Legacy - SLOW)")
    print("-" * 90)
    payload_array = {
        'audio_data': list(audio_bytes),
        'sample_rate': sample_rate
    }
    json_array = json.dumps(payload_array)
    print(f"  Payload size: {len(json_array)} bytes")
    print(f"  Size increase: {len(json_array) / len(audio_bytes):.1f}x")
    print(f"  First 100 chars: {json_array[:100]}...")
    
    # Format 2: Base64 (fast)
    print(f"\n📤 Format 2: Base64 (Recommended - FAST)")
    print("-" * 90)
    audio_b64 = base64.b64encode(audio_bytes).decode('utf-8')
    payload_b64 = {
        'audio_data': audio_b64,
        'sample_rate': sample_rate
    }
    json_b64 = json.dumps(payload_b64)
    print(f"  Payload size: {len(json_b64)} bytes")
    print(f"  Size decrease: {len(json_array) / len(json_b64):.1f}x smaller than array format")
    print(f"  First 100 chars: {json_b64[:100]}...")
    
    print(f"\n🔌 Backend Connection: {backend_url}")
    
    # Test 1: Array format
    print(f"\n� Test 1: Array Format (3 requests)")
    print("-" * 90)
    times_array = []
    
    for i in range(3):
        start = time.time()
        try:
            response = requests.post(
                f'{backend_url}/analyze',
                json=payload_array,
                timeout=10
            )
            total_ms = (time.time() - start) * 1000
            times_array.append(total_ms)
            
            if response.status_code == 200:
                data = response.json()
                notes = data.get('notes', [])
                print(f"Request {i+1}: {total_ms:7.1f}ms | {len(notes)} notes")
            else:
                print(f"Request {i+1}: FAILED (HTTP {response.status_code})")
        except Exception as e:
            print(f"Request {i+1}: ERROR - {e}")
        
        time.sleep(0.1)
    
    # Test 2: Base64 format
    print(f"\n📥 Test 2: Base64 Format (3 requests)")
    print("-" * 90)
    times_b64 = []
    
    for i in range(3):
        start = time.time()
        try:
            response = requests.post(
                f'{backend_url}/analyze',
                json=payload_b64,
                timeout=10
            )
            total_ms = (time.time() - start) * 1000
            times_b64.append(total_ms)
            
            if response.status_code == 200:
                data = response.json()
                notes = data.get('notes', [])
                print(f"Request {i+1}: {total_ms:7.1f}ms | {len(notes)} notes")
            else:
                print(f"Request {i+1}: FAILED (HTTP {response.status_code})")
        except Exception as e:
            print(f"Request {i+1}: ERROR - {e}")
        
        time.sleep(0.1)
    
    print("-" * 90)
    print(f"\n📊 RESULTS:")
    print(f"\n  Array Format:")
    if times_array:
        print(f"    Avg: {sum(times_array)/len(times_array):.1f}ms")
        print(f"    Min: {min(times_array):.1f}ms")
        print(f"    Max: {max(times_array):.1f}ms")
    
    print(f"\n  Base64 Format:")
    if times_b64:
        print(f"    Avg: {sum(times_b64)/len(times_b64):.1f}ms")
        print(f"    Min: {min(times_b64):.1f}ms")
        print(f"    Max: {max(times_b64):.1f}ms")
    
    if times_array and times_b64:
        array_avg = sum(times_array) / len(times_array)
        b64_avg = sum(times_b64) / len(times_b64)
        improvement = (array_avg - b64_avg) / array_avg * 100
        print(f"\n  🚀 Base64 is {improvement:.1f}% faster!")
    
    print("\n" + "=" * 90)
    print("✓ Check backend logs (RUST_LOG=info) for detailed timing breakdown")
    print("=" * 90)

if __name__ == '__main__':
    main()

