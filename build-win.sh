#!/bin/bash
set -e

export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

echo "=== Ripple Windows Cross-Compilation (macOS -> x86_64-pc-windows-msvc) ==="
echo ""
echo "Note: macOS Tauri bundler cannot create MSI/NSIS installers."
echo "This script cross-compiles the Windows .exe binary only."
echo "For full Windows installer, use GitHub Actions CI."
echo ""

cd "$(dirname "$0")/ripple-app"

echo "[1/2] Building frontend..."
npm run build

echo "[2/2] Cross-compiling Rust backend for Windows..."
cd src-tauri
cargo xwin build --target x86_64-pc-windows-msvc --release

echo ""
echo "=== Done! ==="
EXE="target/x86_64-pc-windows-msvc/release/ripple.exe"
ls -lh "$EXE"
echo "Output: ripple-app/src-tauri/$EXE"
