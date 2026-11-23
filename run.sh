#!/usr/bin/env bash
# Levanta VoxFlow en modo desarrollo (Tauri + Vite con recarga en caliente).
# Uso: ./run.sh            (la PoC de Fase 0: ./run-poc.sh [audio.wav] [idioma])
set -euo pipefail
cd "$(dirname "$0")"
export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib/llvm-18/lib}"
# En desarrollo se reutilizan los modelos del repo (scripts/fetch-model.sh base).
export VOXFLOW_MODELS_DIR="${VOXFLOW_MODELS_DIR:-$PWD/models}"
[ -d node_modules ] || pnpm install --frozen-lockfile
exec pnpm tauri dev
