<p align="center"><img src="assets/brand/logo.png" alt="VoxFlow" width="200"></p>

# VoxFlow

Dictado local en español para Ubuntu 24.04 (X11/Wayland). Audio y texto se procesan en tu equipo (whisper.cpp); sin telemetría.
Especificación: [docs/SPEC.md](docs/SPEC.md) · Decisiones: [docs/DECISIONS.md](docs/DECISIONS.md) · Pruebas: [docs/TEST_PLAN.md](docs/TEST_PLAN.md)

> **Estado: Fase 1 (esqueleto).** Hay ventana (Tauri 2 + React), bandeja, preferencias persistentes y máquina de estados. **Aún no graba ni transcribe** (Fase 2) ni inserta texto (Fase 3); el botón de grabar está deshabilitado y lo indica. La PoC de transcripción CLI sigue en `poc/` (`./run-poc.sh`).

## Requisitos (Ubuntu 24.04 / Linux)

```bash
sudo apt install build-essential cmake pkg-config clang libclang-dev curl ffmpeg pipewire-bin \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev
# Rust estable: https://rustup.rs  ·  Node 20+ y pnpm (https://pnpm.io/installation)
```
`libclang-dev` solo es necesario para la PoC/Fase 2 (bindings de whisper.cpp). Si solo tienes `libclang1-18`, los scripts usan `LIBCLANG_PATH=/usr/lib/llvm-18/lib` por defecto. La Fase 2 añadirá `libasound2-dev`.

## Uso

```bash
./run.sh                 # levanta la app en modo desarrollo (pnpm tauri dev)
./build.sh --check       # solo fmt + clippy + tests (Rust y Vitest)
./build.sh               # checks + paquetes .deb y .AppImage
# salida: src-tauri/target/release/bundle/{deb,appimage}/

./run-poc.sh [audio.wav] [es|en]   # PoC Fase 0 (whisper.cpp, WAV 16 kHz mono 16-bit)
scripts/fetch-model.sh base        # descarga explícita del modelo (~142 MB) con SHA-256
scripts/record-test.sh 8           # graba 8 s del micrófono a tests/fixtures/mic-es.wav
```
Instalar el `.deb`: `sudo apt install ./src-tauri/target/release/bundle/deb/VoxFlow_0.1.0_amd64.deb`. El `.AppImage` se ejecuta con `chmod +x` y doble clic. Convertir audio: `ffmpeg -i in.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.

Preferencias: `~/.config/pe.voxflow.app/settings.json` (sin audio ni texto).

No hay CI/CD: todo se compila y ejecuta en local con `build.sh` y `run.sh`.

## Desinstalar
- Instalado con `.deb`: `sudo apt remove voxflow`. AppImage: borra el archivo.
- Opcional: `rm -r ~/.config/pe.voxflow.app` (preferencias).
- Desarrollo: borra la carpeta del repo (`models/`, `target/`, `node_modules/`).

## Marca
Logo y línea gráfica en `assets/brand/` (paleta: #1A0F6B, #2E3BE6, #00D4FF, #00E6C3; tipografías Montserrat/Inter).
