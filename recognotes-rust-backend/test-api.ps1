#!/usr/bin/env powershell
# Test script for RecogNotes Rust Backend
# Usage: .\test-api.ps1

Write-Host "RecogNotes Rust Backend - API Test Script" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host ""

$baseUrl = "http://localhost:5000"

# Test 1: Health Check
Write-Host "Test 1: Health Check" -ForegroundColor Yellow
Write-Host "GET /health" -ForegroundColor Cyan
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/health" -Method Get
    Write-Host "✓ Success: $($response | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: Analyze Audio
Write-Host "Test 2: Analyze Audio" -ForegroundColor Yellow
Write-Host "POST /analyze" -ForegroundColor Cyan
try {
    $body = @{
        audio_data = @()
        sample_rate = 44100
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "$baseUrl/analyze" -Method Post -ContentType "application/json" -Body $body
    Write-Host "✓ Success: $($response | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Get Last Result
Write-Host "Test 3: Get Last Result" -ForegroundColor Yellow
Write-Host "GET /last-result" -ForegroundColor Cyan
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/last-result" -Method Get
    Write-Host "✓ Success: $($response | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "Tests Complete!" -ForegroundColor Green
