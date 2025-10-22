#!/bin/bash
# Simple build script for RecogNotes
# Works on Linux and macOS

set -e

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is not installed"
    echo "Please install from https://rustup.rs/"
    exit 1
fi

# Install cargo-make if not present and build
cargo install --list | grep -q "cargo-make" || {
    echo "� Installing cargo-make..."
    cargo install cargo-make
}

# Build using cargo-make
cargo make
