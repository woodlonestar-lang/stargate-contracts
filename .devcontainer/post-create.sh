#!/bin/bash
set -e

echo "Installing Rust toolchain components..."
rustup target add wasm32-unknown-unknown

echo "Installing Soroban CLI..."
cargo install soroban-cli

echo "Dev container setup complete!"
echo "Run: cargo test"
