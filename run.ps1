# Run RecogNotes: Build and launch backend + GUI

$ErrorActionPreference = "Stop"

Write-Host "Building..." -ForegroundColor Cyan
& "$PSScriptRoot\build.ps1"

Write-Host ""
Write-Host "Starting GUI..." -ForegroundColor Cyan
Start-Process "$PSScriptRoot\target\release\recognotes-desktop-gui.exe" -WindowStyle Normal

Start-Sleep -Seconds 2

Write-Host "Starting backend..." -ForegroundColor Green
Write-Host "Backend running at http://localhost:5000" -ForegroundColor Gray
Write-Host "Press Ctrl+C to stop" -ForegroundColor Gray
Write-Host ""

& "$PSScriptRoot\target\release\recognotes-rust-backend.exe"
