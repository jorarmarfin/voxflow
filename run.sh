#!/usr/bin/env bash
# Levanta VoxFlow en local. Fase 0: ejecuta la PoC de transcripción.
# Uso: ./run.sh [audio.wav] [idioma]    (def.: tests/fixtures/mic-es.wav o jfk.wav)
set -euo pipefail
cd "$(dirname "$0")"
export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib/llvm-18/lib}"
bin=poc/whisper-poc/target/release/whisper-poc
[ -x "$bin" ] || ./build.sh
[ -f models/ggml-base.bin ] || scripts/fetch-model.sh base
audio="${1:-}"; lang="${2:-es}"
if [ -z "$audio" ]; then
  if [ -f tests/fixtures/mic-es.wav ]; then audio=tests/fixtures/mic-es.wav
  else audio=tests/fixtures/jfk.wav; lang=en; fi
fi
exec "$bin" models/ggml-base.bin "$audio" "$lang"
