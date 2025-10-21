# Simple build script for RecogNotes
# Works on Windows with PowerShell

$ErrorActionPreference = "Stop"

# Check if cargo is installed
try {
    $null = cargo --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo not found"
    }
} catch {
    Write-Host "❌ Error: Rust/Cargo is not installed" -ForegroundColor Red
    Write-Host "Please install from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Install cargo-make if not present
$installed = cargo install --list | Select-String "cargo-make"
if (-not $installed) {
    Write-Host "� Installing cargo-make..." -ForegroundColor Cyan
    cargo install cargo-make
}

# Build using cargo-make
cargo make
