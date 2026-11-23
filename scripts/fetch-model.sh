#!/usr/bin/env bash
# Descarga explícita de un modelo ggml de whisper.cpp con verificación SHA-256.
# Uso: scripts/fetch-model.sh [base|small]   (por defecto: base)
set -euo pipefail
cd "$(dirname "$0")/.."
name="${1:-base}"
case "$name" in
  base)  sha=60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe ;;
  small) sha="" ;; # TODO: fijar checksum al adoptar este modelo
  *) echo "modelo no soportado: $name" >&2; exit 2 ;;
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
