<p align="center"><img src="assets/brand/logo.png" alt="VoxFlow" width="200"></p>

# VoxFlow

Dictado local en español para Ubuntu 24.04 (X11/Wayland). Audio y texto se procesan en tu equipo (whisper.cpp); sin telemetría.
Especificación: [docs/SPEC.md](docs/SPEC.md) · Decisiones: [docs/DECISIONS.md](docs/DECISIONS.md) · Pruebas: [docs/TEST_PLAN.md](docs/TEST_PLAN.md)

> **Estado: Fase 2 (voz).** Graba del micrófono (PipeWire vía `pw-record`), transcribe en local con whisper.cpp y muestra el texto editable con Copiar. Gestión de modelos con descarga verificada. **Aún no** hay `voxflow --toggle` ni inserción automática (Fase 3). Probado en X11; Wayland sin validar.

## Requisitos (Ubuntu 24.04 / Linux)

```bash
sudo apt install build-essential cmake pkg-config clang libclang-dev curl ffmpeg pipewire-bin \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev
# Rust estable: https://rustup.rs  ·  Node 20+ y pnpm (https://pnpm.io/installation)
```
`libclang-dev` es necesario para compilar los bindings de whisper.cpp; `pipewire-bin` (`pw-record`) y `pulseaudio-utils`/`pipewire-pulse` (`pactl`) para el micrófono. Si solo tienes `libclang1-18`, los scripts usan `LIBCLANG_PATH=/usr/lib/llvm-18/lib` por defecto.

## Uso

```bash
./run.sh                 # levanta la app en desarrollo (usa ./models; antes: scripts/fetch-model.sh base)
./build.sh --check       # solo fmt + clippy + tests (Rust y Vitest)
./build.sh               # checks + paquetes .deb y .AppImage
# salida: src-tauri/target/release/bundle/{deb,appimage}/

./run-poc.sh [audio.wav] [es|en]   # PoC Fase 0 (whisper.cpp, WAV 16 kHz mono 16-bit)
scripts/fetch-model.sh base        # descarga explícita del modelo (~142 MB) con SHA-256
scripts/record-test.sh 8           # graba 8 s del micrófono a tests/fixtures/mic-es.wav
```
Instalar el `.deb`: `sudo apt install ./src-tauri/target/release/bundle/deb/VoxFlow_0.1.0_amd64.deb`. El `.AppImage` se ejecuta con `chmod +x` y doble clic. Convertir audio: `ffmpeg -i in.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.

Preferencias: `~/.config/pe.voxflow.app/settings.json` (sin audio ni texto). Modelos instalados desde la app: `~/.local/share/pe.voxflow.app/models/`.

**Primer uso:** Preferencias → elegir micrófono → *Descargar modelo* (única conexión de red, con SHA-256) → Guardar → pestaña Dictado → *Hablar con VoxFlow* → *Detener* → revisar/editar → *Copiar* y pegar con Ctrl+V.

**Pruebas con hardware/modelo** (ignoradas por defecto): `cd src-tauri && VOXFLOW_MODELS_DIR=../models cargo test --lib -- --ignored --nocapture`

No hay CI/CD: todo se compila y ejecuta en local con `build.sh` y `run.sh`.

## Desinstalar
- Instalado con `.deb`: `sudo apt remove voxflow`. AppImage: borra el archivo.
- Opcional: `rm -r ~/.config/pe.voxflow.app ~/.local/share/pe.voxflow.app` (preferencias y modelos).
- Desarrollo: borra la carpeta del repo (`models/`, `target/`, `node_modules/`).

## Marca
Logo y línea gráfica en `assets/brand/` (paleta: #1A0F6B, #2E3BE6, #00D4FF, #00E6C3; tipografías Montserrat/Inter).
