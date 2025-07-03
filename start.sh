#!/bin/bash

# Voltron Terminal Startup Script

echo "Starting Voltron Terminal..."

# Ensure cargo is in PATH first
export PATH="$HOME/.cargo/bin:$PATH"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed or not in PATH"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if node_modules exist
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    pnpm install
fi

# Kill any existing processes on port 5173
lsof -ti:5173 | xargs kill -9 2>/dev/null || true

# Run the app
echo "Launching Voltron..."
pnpm run tauri dev