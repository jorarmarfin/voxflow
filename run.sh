#!/usr/bin/env bash
# Levanta VoxFlow en modo desarrollo (Tauri + Vite con recarga en caliente).
# Uso: ./run.sh            (la PoC de Fase 0: ./run-poc.sh [audio.wav] [idioma])
set -euo pipefail
cd "$(dirname "$0")"
[ -d node_modules ] || pnpm install --frozen-lockfile
exec pnpm tauri dev
