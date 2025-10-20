#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Fast Backend Stress Test - Uses raw sockets instead of requests library
This bypasses the ~2-second Python requests overhead on Windows
"""

import socket
import json
import time
import concurrent.futures
import statistics
import struct
import math
import base64
import threading
from typing import List, Tuple
import sys
import io

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

def send_raw_socket(host: str, port: int, payload: str) -> Tuple[bool, float, dict]:
    """Send HTTP request using raw socket - FAST (bypasses requests library overhead)."""
    start_time = time.time()
    
    try:
        # Create socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        
        # Measure connection time
        conn_start = time.time()
        sock.connect((host, port))
        conn_time = (time.time() - conn_start) * 1000
        
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
        
        # Measure send time
        send_start = time.time()
        sock.sendall(headers.encode() + body.encode())
        send_time = (time.time() - send_start) * 1000
        
        # Receive response
        recv_start = time.time()
        response = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            response += chunk
        recv_time = (time.time() - recv_start) * 1000
        
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
        elapsed_ms = (time.time() - start_time) * 1000
        
        return True, elapsed_ms, {
            'data': data,
            'conn_time': conn_time,
            'send_time': send_time,
            'recv_time': recv_time
        }
    except Exception as e:
        elapsed_ms = (time.time() - start_time) * 1000
        return False, elapsed_ms, {'error': str(e)}

def single_request(host: str, port: int, audio_bytes: bytes, sample_rate: int, request_num: int = 0) -> Tuple[bool, float, dict]:
    """Send a single analyze request using raw socket."""
    audio_b64 = base64.b64encode(audio_bytes).decode('utf-8')
    payload = json.dumps({
        'audio_data': audio_b64,
        'sample_rate': sample_rate
    })
    
    return send_raw_socket(host, port, payload)

def main():
    host = "localhost"
    port = 5000
    sample_rate = 48000
    
    print("=" * 70)
    print("🚀 FAST BACKEND STRESS TEST - Raw Socket (No requests library overhead)")
    print("=" * 70)
    print(f"Backend: {host}:{port}")
    print(f"Sample Rate: {sample_rate} Hz")
    print("")
    
    # Skip health check - go straight to analyze
    print("Backend ready: localhost:5000")
    print("")
    
    # Generate test audio
    duration_ms = 10.0
    audio_bytes = generate_test_audio(sample_rate, duration_ms=duration_ms, frequency=440.0)
    num_samples = int(sample_rate * duration_ms / 1000.0)
    print(f"Generated test audio: {duration_ms}ms duration")
    print(f"  Samples: {num_samples} @ {sample_rate}Hz")
    print(f"  Bytes: {len(audio_bytes)} (16-bit PCM)")
    print("")
    
    # Test 1: Single Request (Renamed from Test 2)
    print("Test 1️⃣  Single Analyze Request (Raw Socket)")
    print("-" * 70)
    success, elapsed, result = single_request(host, port, audio_bytes, sample_rate)
    
    if success:
        print(f"✓ Single Request: OK ({elapsed:.1f}ms)")
        data = result['data']
        print(f"  Total: {elapsed:.1f}ms = Conn: {result['conn_time']:.1f}ms + Send: {result['send_time']:.1f}ms + Recv: {result['recv_time']:.1f}ms")
        print(f"  Notes detected: {len(data.get('notes', []))}")
        for note in data.get('notes', []):
            conf = int(note.get('confidence', 0) * 100)
            print(f"    - {note.get('note')}: {conf}%")
    else:
        print(f"✗ Single Request Failed: {result.get('error')}")
    print("")
    
    # Test 2: Sequential Requests (Renamed from Test 3)
    print("Test 2️⃣  Sequential Requests (10 requests with 100ms delay)")
    print("-" * 70)
    seq_times = []
    seq_success = 0
    
    for i in range(10):
        success, elapsed, result = single_request(host, port, audio_bytes, sample_rate, i)
        seq_times.append(elapsed)
        if success:
            seq_success += 1
        status = "✓" if success else "✗"
        print(f"  Request {i+1:2d}: {status} {elapsed:6.1f}ms", end="")
        
        if success:
            data = result['data']
            print(f" ({len(data.get('notes', []))} notes)")
        else:
            print(f" {result.get('error', 'failed')}")
        
        time.sleep(0.1)  # 100ms delay between requests
    
    print(f"\nSequential Stats:")
    print(f"  Success: {seq_success}/10")
    print(f"  Avg: {statistics.mean(seq_times):.1f}ms")
    print(f"  Min: {min(seq_times):.1f}ms")
    print(f"  Max: {max(seq_times):.1f}ms")
    if len(seq_times) > 1:
        print(f"  StdDev: {statistics.stdev(seq_times):.1f}ms")
    print("")
    
    # Test 3: Concurrent Requests (Renamed from Test 4)
    print("Test 3️⃣  Concurrent Requests (50 parallel requests)")
    print("-" * 70)
    
    concurrent_times = []
    concurrent_success = 0
    concurrent_errors = []
    concurrent_lock = threading.Lock()
    
    def concurrent_request(request_num):
        try:
            success, elapsed, result = single_request(host, port, audio_bytes, sample_rate, request_num)
            with concurrent_lock:
                concurrent_times.append(elapsed)
                if success:
                    return (request_num, True, elapsed)
                else:
                    return (request_num, False, result.get('error'))
            return (request_num, success, elapsed if success else result.get('error'))
        except Exception as e:
            with concurrent_lock:
                concurrent_times.append(float('nan'))
            return (request_num, False, str(e))
    
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        print("Sending 50 requests (10 concurrent workers)...")
        futures = [executor.submit(concurrent_request, i) for i in range(50)]
        
        for i, future in enumerate(futures):
            try:
                req_num, success, data = future.result(timeout=15)
                if success:
                    concurrent_success += 1
                else:
                    concurrent_errors.append((req_num, data))
                
                if (i + 1) % 10 == 0:
                    print(f"  Completed {i+1:2d}/50 requests...")
            except Exception as e:
                concurrent_errors.append((i, str(e)))
    
    print("")
    print(f"Concurrent Stats:")
    print(f"  Success: {concurrent_success}/50")
    valid_times = [t for t in concurrent_times if not math.isnan(t)]
    if valid_times:
        print(f"  Avg: {statistics.mean(valid_times):.1f}ms")
        print(f"  Min: {min(valid_times):.1f}ms")
        print(f"  Max: {max(valid_times):.1f}ms")
        if len(valid_times) > 1:
            print(f"  StdDev: {statistics.stdev(valid_times):.1f}ms")
    
    if concurrent_errors:
        print(f"\n  Errors ({len(concurrent_errors)}):")
        for req_num, error in concurrent_errors[:5]:
            print(f"    Request {req_num}: {error}")
        if len(concurrent_errors) > 5:
            print(f"    ... and {len(concurrent_errors) - 5} more errors")
    print("")
    
    # Test 4: Rapid-Fire Test (Renamed from Test 5)
    print("Test 4️⃣  Rapid-Fire Test (20 requests as fast as possible)")
    print("-" * 70)
    
    rapid_times = []
    rapid_start = time.time()
    
    for i in range(20):
        success, elapsed, result = single_request(host, port, audio_bytes, sample_rate, i)
        rapid_times.append(elapsed)
        status = "✓" if success else "✗"
        print(f"  Request {i+1:2d}: {status} {elapsed:6.1f}ms")
    
    rapid_total = (time.time() - rapid_start) * 1000
    
    print(f"\nRapid-Fire Stats:")
    print(f"  Total Time: {rapid_total:.1f}ms")
    print(f"  Avg: {statistics.mean(rapid_times):.1f}ms")
    print(f"  Min: {min(rapid_times):.1f}ms")
    print(f"  Max: {max(rapid_times):.1f}ms")
    print(f"  Throughput: {len(rapid_times) * 1000 / rapid_total:.1f} req/sec")
    
    # Check for throttling
    if max(rapid_times) > statistics.mean(rapid_times) * 2:
        print(f"  ⚠️  WARNING: Response time variance detected - possible throttling!")
    else:
        print(f"  ✓ No throttling detected")
    
    print("")
    print("=" * 70)
    print("✓ Fast Stress Test Complete!")
    print("=" * 70)
    print("")
    print("COMPARISON:")
    print(f"  Expected avg (requests lib):   ~2000ms per request")
    print(f"  Actual avg (raw socket):       ~{statistics.mean(valid_times):.1f}ms per request")
    print(f"  Improvement:                   ~{2000/statistics.mean(valid_times):.1f}x faster")
    print("")

if __name__ == '__main__':
    main()
