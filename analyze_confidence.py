#!/usr/bin/env python3
"""
Test to show confidence values for different frequencies
"""

import math

def calculate_confidence(input_freq, base_freq):
    """Calculate confidence using the same formula as the backend"""
    cents_diff = 1200.0 * abs(math.log2(input_freq / base_freq))
    confidence = max(0.0, min(1.0, 1.0 - (cents_diff / 100.0)))
    return confidence

# Test around A4 (440 Hz)
print("=" * 60)
print("CONFIDENCE CALCULATION FOR A4 (440 Hz)")
print("=" * 60)

base_freq = 440.0
test_freqs = [
    (435, "-5 Hz"),
    (437, "-3 Hz"),
    (438, "-2 Hz"),
    (439, "-1 Hz"),
    (440, "exact"),
    (441, "+1 Hz"),
    (442, "+2 Hz"),
    (443, "+3 Hz"),
    (445, "+5 Hz"),
]

for freq, label in test_freqs:
    conf = calculate_confidence(freq, base_freq)
    cents = 1200.0 * abs(math.log2(freq / base_freq))
    status = "✓ PASS" if conf > 0.5 else "✗ FAIL"
    print(f"{label:>10}: {freq:3d} Hz → {cents:6.2f} cents → confidence {conf:.3f} {status}")

print()
print("Key insight:")
print("  - Confidence < 0.5 means note gets FILTERED OUT")
print("  - ±5 Hz from 440 Hz = ~22 cents difference")
print("  - Confidence = 1.0 - (22/100) = 0.78 (should PASS)")
print()
print("But the diagnostic showed ±5 Hz FAILED...")
print("This means the POWER-based confidence is dropping it below 0.5")
