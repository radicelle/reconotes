# Test script to verify note detection across all frequencies
# Generates synthetic audio at each note frequency and sends to backend

param(
    [string]$BackendUrl = "http://localhost:5000",
    [int]$Duration = 1000,  # Duration in milliseconds
    [int]$SampleRate = 44100
)

# Function to generate sine wave for a given frequency
function Generate-SineWave {
    param(
        [float]$Frequency,
        [int]$DurationMs,
        [int]$SampleRate
    )
    
    $numSamples = [int]($SampleRate * $DurationMs / 1000)
    $samples = New-Object float[] $numSamples
    
    for ($i = 0; $i -lt $numSamples; $i++) {
        $t = $i / $SampleRate
        $samples[$i] = [Math]::Sin(2 * [Math]::PI * $Frequency * $t) * 0.8  # 80% amplitude to avoid clipping
    }
    
    return $samples
}

# Function to convert float samples to 16-bit PCM bytes
function Convert-FloatToPCM16 {
    param([float[]]$Samples)
    
    $bytes = New-Object byte[] ($Samples.Length * 2)
    for ($i = 0; $i -lt $Samples.Length; $i++) {
        $sample = [int]([Math]::Max(-32768, [Math]::Min(32767, $Samples[$i] * 32768)))
        $bytes[$i * 2] = $sample -band 0xFF
        $bytes[$i * 2 + 1] = ($sample -shr 8) -band 0xFF
    }
    
    return $bytes
}

# Function to encode bytes as base64
function Encode-Base64 {
    param([byte[]]$Bytes)
    return [Convert]::ToBase64String($Bytes)
}

# Note frequency mappings (from the Rust code)
$notes = @{
    "C2" = 65.41
    "D2" = 73.42
    "E2" = 82.41
    "F2" = 87.31
    "G2" = 98.00
    "A2" = 110.00
    "B2" = 123.47
    
    "C3" = 130.81
    "D3" = 146.83
    "E3" = 164.81
    "F3" = 174.61
    "G3" = 196.00
    "A3" = 220.00
    "B3" = 246.94
    
    "C4" = 261.63
    "D4" = 293.66
    "E4" = 329.63
    "F4" = 349.23
    "G4" = 392.00
    "A4" = 440.00
    "B4" = 493.88
    
    "C5" = 523.25
    "D5" = 587.33
    "E5" = 659.25
    "F5" = 698.46
    "G5" = 783.99
    "A5" = 880.00
    "B5" = 987.77
    
    "C6" = 1046.50
}

Write-Host "Testing Note Detection" -ForegroundColor Cyan
Write-Host "Backend: $BackendUrl" -ForegroundColor Cyan
Write-Host "Duration: ${Duration}ms, Sample Rate: $SampleRate Hz" -ForegroundColor Cyan
Write-Host ""

# Test each note
$results = @()
foreach ($note in $notes.GetEnumerator() | Sort-Object -Property Name) {
    $noteName = $note.Name
    $frequency = $note.Value
    
    Write-Host "Testing $noteName (${frequency} Hz)... " -NoNewline
    
    try {
        # Generate audio
        $samples = Generate-SineWave -Frequency $frequency -DurationMs $Duration -SampleRate $SampleRate
        $bytes = Convert-FloatToPCM16 -Samples $samples
        $base64Audio = Encode-Base64 -Bytes $bytes
        
        # Create request
        $body = @{
            audio_data = $base64Audio
            sample_rate = $SampleRate
        } | ConvertTo-Json
        
        # Send to backend
        $response = Invoke-RestMethod `
            -Uri "$BackendUrl/analyze" `
            -Method Post `
            -ContentType "application/json" `
            -Body $body `
            -TimeoutSec 10
        
        # Check response
        if ($response.notes -and $response.notes.Count -gt 0) {
            $detectedNote = $response.notes[0]
            $color = if ($detectedNote.note -eq $noteName) { "Green" } else { "Yellow" }
            $confidencePercent = [Math]::Round($detectedNote.confidence * 100)
            Write-Host "✓ Detected: $($detectedNote.note) ($confidencePercent%)" -ForegroundColor $color
            
            $results += [PSCustomObject]@{
                InputNote = $noteName
                InputFreq = $frequency
                DetectedNote = $detectedNote.note
                Confidence = $detectedNote.confidence
                Match = $detectedNote.note -eq $noteName
            }
        } else {
            Write-Host "✗ No notes detected!" -ForegroundColor Red
            $results += [PSCustomObject]@{
                InputNote = $noteName
                InputFreq = $frequency
                DetectedNote = "NONE"
                Confidence = 0
                Match = $false
            }
        }
    }
    catch {
        Write-Host "✗ Error: $($_.Exception.Message)" -ForegroundColor Red
        $results += [PSCustomObject]@{
            InputNote = $noteName
            InputFreq = $frequency
            DetectedNote = "ERROR"
            Confidence = 0
            Match = $false
        }
    }
    
    Start-Sleep -Milliseconds 100
}

# Summary
Write-Host ""
Write-Host "=== SUMMARY ===" -ForegroundColor Cyan
Write-Host ""

$correct = @($results | Where-Object { $_.Match }).Count
$total = @($results).Count
$accuracy = ($correct / $total) * 100

$accuracyPercent = [Math]::Round($accuracy)
Write-Host "Correct: $correct / $total ($accuracyPercent%)" -ForegroundColor $(if ($accuracy -ge 80) { "Green" } else { "Red" })
Write-Host ""

# Detailed table
Write-Host "Detailed Results:" -ForegroundColor Cyan
$results | Format-Table -AutoSize @(
    @{ Label = "Input Note"; Expression = { $_.InputNote } }
    @{ Label = "Frequency"; Expression = { "$($_.InputFreq) Hz" } }
    @{ Label = "Detected"; Expression = { $_.DetectedNote } }
    @{ Label = "Confidence"; Expression = { "$([Math]::Round($_.Confidence * 100))%" } }
    @{ Label = "Match"; Expression = { if ($_.Match) { "YES" } else { "NO" } }; Alignment = "Center" }
)

# Show failures
$failures = @($results | Where-Object { -not $_.Match })
if ($failures.Count -gt 0) {
    Write-Host ""
    Write-Host "FAILURES:" -ForegroundColor Red
    $failures | Format-Table -AutoSize @(
        @{ Label = "Input Note"; Expression = { $_.InputNote } }
        @{ Label = "Detected"; Expression = { $_.DetectedNote } }
    )
}
