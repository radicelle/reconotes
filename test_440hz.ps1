# PowerShell script to generate 440Hz test audio and send to backend

function Generate-440HzSineWave {
    param(
        [int]$DurationMs = 100,
        [int]$SampleRate = 48000
    )
    
    Write-Host "Generating 440Hz sine wave..."
    
    $durationS = $DurationMs / 1000.0
    $numSamples = [int]($SampleRate * $durationS)
    $frequency = 440.0
    $amplitude = 0.8
    
    $audioData = @()
    for ($i = 0; $i -lt $numSamples; $i++) {
        $t = $i / $SampleRate
        $sample = $amplitude * [Math]::Sin(2 * [Math]::PI * $frequency * $t)
        # Convert to 16-bit PCM
        $pcmValue = [int]($sample * 32767)
        $pcmValue = [Math]::Max([Math]::Min($pcmValue, 32767), -32768)
        
        # Convert to bytes (little-endian 16-bit integer)
        $bytes = [BitConverter]::GetBytes([int16]$pcmValue)
        foreach ($byte in $bytes) {
            $audioData += [int]$byte
        }
    }
    
    Write-Host "Generated 440Hz sine wave:"
    Write-Host "  Duration: ${DurationMs}ms"
    Write-Host "  Sample rate: ${SampleRate}Hz"
    Write-Host "  Samples: $numSamples"
    Write-Host "  Expected: A4 (440Hz)"
    
    return $audioData
}

function Test-Backend {
    param(
        [int[]]$AudioData,
        [string]$BackendUrl = "http://localhost:5000"
    )
    
    Write-Host "`nSending to backend at $BackendUrl..."
    
    $payload = @{
        audio_data = $AudioData
        sample_rate = 48000
    } | ConvertTo-Json
    
    try {
        $response = Invoke-WebRequest -Uri "$BackendUrl/analyze" `
            -Method Post `
            -ContentType "application/json" `
            -Body $payload `
            -TimeoutSec 5
        
        $result = $response.Content | ConvertFrom-Json
        
        Write-Host "`nBackend Response:"
        $result | ConvertTo-Json | Write-Host
        
        if ($result.notes -and $result.notes.Count -gt 0) {
            Write-Host "`nDetected $($result.notes.Count) note(s):"
            foreach ($note in $result.notes) {
                $confidence = [math]::Round($note.confidence * 100, 1)
                Write-Host "  - $($note.note): ${confidence}% confidence"
                
                if ($note.note -like "*A4*") {
                    if ($note.confidence -gt 0.7) {
                        Write-Host "✅ PASS: A4 detected with ${confidence}% confidence" -ForegroundColor Green
                    } else {
                        Write-Host "⚠️  WARNING: A4 detected but low confidence (${confidence}%)" -ForegroundColor Yellow
                    }
                } else {
                    Write-Host "❌ FAIL: Expected A4 but got $($note.note)" -ForegroundColor Red
                }
            }
        } else {
            Write-Host "❌ FAIL: No notes detected" -ForegroundColor Red
        }
        
    } catch {
        Write-Host "❌ ERROR: $_" -ForegroundColor Red
    }
}

# Main
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "440Hz A Note Backend Test" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

$audioData = Generate-440HzSineWave -DurationMs 100 -SampleRate 48000
Test-Backend -AudioData $audioData -BackendUrl "http://localhost:5000"

Write-Host "`n============================================================" -ForegroundColor Cyan
