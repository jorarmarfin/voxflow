#!/usr/bin/env bash
# Graba N segundos (def. 8) del micrófono por defecto a WAV 16 kHz mono 16-bit.
# Uso: scripts/record-test.sh [segundos] [salida.wav]
set -euo pipefail
cd "$(dirname "$0")/.."
secs="${1:-8}"; out="${2:-tests/fixtures/mic-es.wav}"
echo "Habla ahora ($secs s)..."
timeout "$secs" pw-record --rate 16000 --channels 1 --format s16 "$out" || true
echo "Guardado: $out"
