<p align="center"><img src="assets/brand/logo.png" alt="VoxFlow" width="200"></p>

# VoxFlow

Dictado local en español para Ubuntu 24.04 (X11/Wayland). Audio y texto se procesan en tu equipo (whisper.cpp); sin telemetría.
Especificación: [docs/SPEC.md](docs/SPEC.md) · Decisiones: [docs/DECISIONS.md](docs/DECISIONS.md) · Pruebas: [docs/TEST_PLAN.md](docs/TEST_PLAN.md)

> **Estado: Fase 5 (inserción en el cursor, X11).** Dictado por ventana o atajo (`voxflow --toggle`), copia al portapapeles y, en **X11 con `xdotool`**, inserción directa en la ventana que tenías activa. **Wayland: sin inserción** (solo portapapeles) y sin validar en sesión real.

## Requisitos (Ubuntu 24.04 / Linux)

```bash
sudo apt install xdotool x11-utils wl-clipboard xclip libnotify-bin pulseaudio-utils \
  build-essential cmake pkg-config clang libclang-dev curl ffmpeg pipewire-bin \
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
Instalar el `.deb`: `sudo apt install ./src-tauri/target/release/bundle/deb/VoxFlow_0.2.0_amd64.deb`. El `.AppImage` se ejecuta con `chmod +x` y doble clic. Convertir audio: `ffmpeg -i in.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.

Preferencias: `~/.config/pe.voxflow.app/settings.json` (sin audio ni texto). Modelos instalados desde la app: `~/.local/share/pe.voxflow.app/models/`.

**Primer uso:** Preferencias → elegir micrófono → *Descargar modelo* (única conexión de red, con SHA-256) → Guardar → pestaña Dictado → *Hablar con VoxFlow* → *Detener* → revisar/editar → *Copiar* y pegar con Ctrl+V.

**Pruebas con hardware/modelo** (ignoradas por defecto): `cd src-tauri && VOXFLOW_MODELS_DIR=../models cargo test --lib -- --ignored --nocapture`

No hay CI/CD: todo se compila y ejecuta en local con `build.sh` y `run.sh`.

## Atajo de teclado (`voxflow --toggle`)

`voxflow --toggle` alterna la grabación en la instancia en marcha (o abre la app y empieza a grabar si no hay ninguna): 1.ª vez graba, 2.ª detiene y transcribe. No roba el foco; avisa con notificaciones (que nunca incluyen el texto). Según el **modo de salida**: *Copiar directo* → copia y notifica "Pega con Ctrl+V"; *Vista previa* → abre la ventana con el texto.

**GNOME** (script que solo añade/quita su propia entrada y conserva tus otros atajos):
```bash
scripts/install-gnome-shortcut.sh                          # Super+Alt+V → voxflow --toggle (.deb instalado)
scripts/install-gnome-shortcut.sh --binding '<Ctrl><Alt>d' --command /ruta/VoxFlow_0.2.0_amd64.AppImage
scripts/install-gnome-shortcut.sh --remove
```
Manual: Ajustes → Teclado → Atajos personalizados → comando `voxflow --toggle`. En KDE: Preferencias → Atajos → Comando personalizado con el mismo comando.

## Insertar en la posición del cursor (X11)

Requiere sesión **X11**, `xdotool` y `xprop` (`x11-utils`); el `.deb` los *recomienda*. Preferencias → Modo de salida → **Insertar en la ventana activa**. Funciona **solo con el atajo** (`voxflow --toggle`): al empezar a grabar se guarda la ventana activa y, al terminar, se pega (Ctrl+V; Ctrl+Shift+V en terminales) **solo si sigue siendo esa ventana**. Si cambiaste de ventana, no es seguro o falta algo, **no se escribe nada**: el texto queda copiado y se avisa. Tras pegar se restaura el texto que tenías antes en el portapapeles (si era texto; imágenes u otros formatos no se restauran).
Evita usarlo con campos de contraseña o ventanas sensibles: VoxFlow no puede saber qué hay en el campo enfocado. El botón *Insertar* de la ventana solo copia (el foco está en VoxFlow).

**Limitaciones (Wayland):** no hay escucha global de teclas ni inserción en otras ventanas; por eso el atajo lo gestiona el escritorio y el texto se pega a mano. Con la ventana en segundo plano, Wayland puede impedir que VoxFlow la traiga al frente.

## Desinstalar
- Instalado con `.deb`: `sudo apt remove voxflow`. AppImage: borra el archivo.
- Opcional: `rm -r ~/.config/pe.voxflow.app ~/.local/share/pe.voxflow.app` (preferencias y modelos).
- Desarrollo: borra la carpeta del repo (`models/`, `target/`, `node_modules/`).

## Licencia
[MIT](LICENSE) © 2026 Luis Mayta. Licencias de dependencias y modelos: [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## Marca
Logo y línea gráfica en `assets/brand/` (paleta: #1A0F6B, #2E3BE6, #00D4FF, #00E6C3; tipografías Montserrat/Inter).
