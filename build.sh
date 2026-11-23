#!/usr/bin/env bash
# Compila VoxFlow en local: comprobaciones + paquetes .deb/.AppImage en src-tauri/target/release/bundle/.
# Uso: ./build.sh [--check]   (--check: solo fmt/lint/tests, sin empaquetar)
set -euo pipefail
cd "$(dirname "$0")"
export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib/llvm-18/lib}"
[ -d node_modules ] || pnpm install --frozen-lockfile
pnpm lint
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
[ "${1:-}" = "--check" ] && { echo "OK (checks)"; exit 0; }
pnpm tauri build
echo "Paquetes en: src-tauri/target/release/bundle/"
