#!/usr/bin/env bash
# PoC de Fase 0: transcribe un WAV con whisper.cpp. Uso: ./run-poc.sh [audio.wav] [idioma]
set -euo pipefail
cd "$(dirname "$0")"
export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib/llvm-18/lib}"
cargo build --release --manifest-path poc/whisper-poc/Cargo.toml
[ -f models/ggml-base.bin ] || scripts/fetch-model.sh base
audio="${1:-}"; lang="${2:-es}"
if [ -z "$audio" ]; then
  if [ -f tests/fixtures/mic-es.wav ]; then audio=tests/fixtures/mic-es.wav
  else audio=tests/fixtures/jfk.wav; lang=en; fi
fi
exec poc/whisper-poc/target/release/whisper-poc models/ggml-base.bin "$audio" "$lang"
