#!/bin/bash
set -e

export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

echo "=== Ripple Windows Cross-Compilation (macOS -> x86_64-pc-windows-msvc) ==="

cd "$(dirname "$0")/ripple-app"

echo "[1/3] Building frontend..."
npm run build

echo "[2/3] Cross-compiling Rust backend..."
cd src-tauri
cargo xwin build --target x86_64-pc-windows-msvc --release

echo "[3/3] Creating MSI installer..."
cargo tauri build --bundles msi --target x86_64-pc-windows-msvc

echo ""
echo "=== Done! ==="
echo "Output: ripple-app/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/"
