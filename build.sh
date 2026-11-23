#!/usr/bin/env bash
# Compila VoxFlow en local (Fase 0: PoC; luego Tauri .deb/.AppImage).
set -euo pipefail
cd "$(dirname "$0")"
export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib/llvm-18/lib}"
cargo fmt --manifest-path poc/whisper-poc/Cargo.toml
cargo clippy --manifest-path poc/whisper-poc/Cargo.toml --release -- -D warnings
cargo build --manifest-path poc/whisper-poc/Cargo.toml --release
echo "OK: poc/whisper-poc/target/release/whisper-poc"
