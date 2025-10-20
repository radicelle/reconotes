# RecogNotes - Workspace Build Guide

With the workspace `Cargo.toml` at the root, you can now use standard Cargo commands for both projects:

## Quick Commands

```bash
# Check both projects
cargo check

# Build both projects
cargo build

# Build in release mode
cargo build --release

# Run clippy on both
cargo clippy --all-targets -- -W clippy::all -W clippy::pedantic -W clippy::nursery

# Auto-fix clippy issues
cargo clippy --fix --allow-dirty --all-targets -- -W clippy::all -W clippy::pedantic -W clippy::nursery

# Run tests
cargo test

# Format code
cargo fmt

# Clean
cargo clean
```

## Run Specific Project

```bash
# Build only backend
cargo build -p recognotes-rust-backend

# Build only frontend  
cargo build -p recognotes-desktop-gui

# Run backend
cargo run -p recognotes-rust-backend

# Run frontend
cargo run -p recognotes-desktop-gui
```

## Done

The workspace `Cargo.toml` handles everything. No need for separate build scripts.
