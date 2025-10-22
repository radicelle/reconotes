#!/bin/bash
# Run RecogNotes: Build and launch backend + GUI

set -e

# Get the script's directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "Building..."
"$SCRIPT_DIR/build.sh"

echo ""
echo "Starting GUI..."
"$SCRIPT_DIR/target/release/recognotes-desktop-gui" &

sleep 2

echo "Starting backend..."
echo "Backend running at http://localhost:5000"
echo "Press Ctrl+C to stop"
echo ""

"$SCRIPT_DIR/target/release/recognotes-rust-backend"
