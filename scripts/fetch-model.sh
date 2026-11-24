#!/usr/bin/env bash
# Descarga explícita de un modelo ggml de whisper.cpp con verificación SHA-256.
# Uso: scripts/fetch-model.sh [base|small|medium|large-v3-turbo-q5_0|large-v3-turbo|large-v3-q5_0|large-v3]
set -euo pipefail
cd "$(dirname "$0")/.."
name="${1:-base}"
case "$name" in
  base)                sha=60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe ;;
  small)               sha=1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b ;;
  medium)              sha=6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208 ;;
  large-v3-turbo-q5_0) sha=394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2 ;;
  large-v3-turbo)      sha=1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69 ;;
  large-v3-q5_0)       sha=d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1 ;;
  large-v3)            sha=64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2 ;;
  *) echo "modelo no soportado: $name (base|small|medium|large-v3-turbo-q5_0|large-v3-turbo|large-v3-q5_0|large-v3)" >&2; exit 2 ;;
esac
[ -n "$sha" ] || { echo "sin checksum registrado para '$name'; no se descarga" >&2; exit 2; }
mkdir -p models
dest="models/ggml-$name.bin"
tmp="$dest.part"
if [ -f "$dest" ] && echo "$sha  $dest" | sha256sum -c --quiet 2>/dev/null; then
  echo "ya instalado y verificado: $dest"; exit 0
fi
curl -fL --progress-bar -o "$tmp" "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-$name.bin"
if echo "$sha  $tmp" | sha256sum -c --quiet; then mv "$tmp" "$dest"; echo "OK: $dest"
else rm -f "$tmp"; echo "checksum inválido; descarga descartada" >&2; exit 1; fi
