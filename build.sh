#!/usr/bin/env bash
# Compila VoxFlow en local: comprobaciones + paquetes .deb/.AppImage en src-tauri/target/release/bundle/.
# Uso: ./build.sh [--check]   (--check: solo fmt/lint/tests, sin empaquetar)
set -euo pipefail
cd "$(dirname "$0")"
export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib/llvm-18/lib}"
# Evita incrustar rutas locales (usuario, carpetas) en los binarios publicados.
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=/home/user --remap-path-prefix=$PWD=/src --remap-path-prefix=$HOME/.cargo=/cargo"
export CFLAGS="${CFLAGS:-} -ffile-prefix-map=$PWD=/src"
export CXXFLAGS="${CXXFLAGS:-} -ffile-prefix-map=$PWD=/src"
# whisper.cpp portable (paquetes que se distribuyen): línea base AVX2/FMA/F16C, sin optimizar para esta CPU.
export GGML_NATIVE=OFF GGML_AVX=ON GGML_AVX2=ON GGML_FMA=ON GGML_F16C=ON GGML_AVX512=OFF
[ -d node_modules ] || pnpm install --frozen-lockfile
pnpm lint
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
[ "${1:-}" = "--check" ] && { echo "OK (checks)"; exit 0; }
pnpm tauri build
B=src-tauri/target/release/bundle
V=$(node -p "require('./package.json').version")
( cd "$B" && sha256sum deb/*_${V}_*.deb appimage/*_${V}_*.AppImage | sed 's# [^ ]*/# #' > SHA256SUMS && cat SHA256SUMS )
echo "Paquetes y SHA256SUMS en: $B/"
