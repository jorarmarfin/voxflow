<p align="center"><img src="assets/brand/logo.png" alt="VoxFlow" width="200"></p>

# VoxFlow

Dictado local en español para Ubuntu 24.04 (X11/Wayland). Audio y texto se procesan en tu equipo (whisper.cpp); sin telemetría.
Especificación: [docs/SPEC.md](docs/SPEC.md) · Decisiones: [docs/DECISIONS.md](docs/DECISIONS.md) · Pruebas: [docs/TEST_PLAN.md](docs/TEST_PLAN.md)

> **Estado: Fase 0 (PoC).** Solo existe una prueba de transcripción por línea de comandos. No hay UI, tray ni inserción de texto todavía.

## Requisitos (Ubuntu 24.04 / Linux)

```bash
sudo apt install build-essential cmake pkg-config clang libclang-dev curl ffmpeg pipewire-bin
# Rust estable: https://rustup.rs
```
`libclang-dev` es necesario para compilar los bindings (`bindgen`). Si solo tienes `libclang1-18`, `run.sh`/`build.sh` usan `LIBCLANG_PATH=/usr/lib/llvm-18/lib` por defecto.

Fases siguientes añadirán (Tauri 2): `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev`, Node + pnpm.

## Uso

```bash
./build.sh                      # compila (fmt + clippy + release)
scripts/fetch-model.sh base     # descarga explícita del modelo (~142 MB) con checksum SHA-256
./run.sh                        # transcribe tests/fixtures/mic-es.wav (o jfk.wav si no existe)
./run.sh mi_audio.wav es        # WAV 16 kHz mono 16-bit
scripts/record-test.sh 8        # graba 8 s del micrófono a tests/fixtures/mic-es.wav
```
Convertir otro audio: `ffmpeg -i in.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav`

No hay CI/CD: todo se compila y ejecuta en local con `build.sh` y `run.sh`.

## Desinstalar
Borra la carpeta del repo (incluye `models/` y `target/`). La PoC no instala nada en el sistema ni escribe fuera del repo (salvo la caché de cargo en `~/.cargo`).

## Marca
Logo y línea gráfica en `assets/brand/` (paleta: #1A0F6B, #2E3BE6, #00D4FF, #00E6C3; tipografías Montserrat/Inter).
