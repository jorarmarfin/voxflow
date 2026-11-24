<p align="center"><img src="assets/brand/logo.png" alt="VoxFlow" width="200"></p>

# VoxFlow

Dictado local en español para Ubuntu 24.04 (X11/Wayland). Audio y texto se procesan en tu equipo (whisper.cpp); sin telemetría.
Especificación: [docs/SPEC.md](docs/SPEC.md) · Decisiones: [docs/DECISIONS.md](docs/DECISIONS.md) · Pruebas: [docs/TEST_PLAN.md](docs/TEST_PLAN.md)

> **Estado: Fase 6 (Windows, sin probar en un equipo real).** Linux (Ubuntu 24.04, X11 probado) y Windows 10/11 (compilado por GitHub Actions; el micrófono, el atajo global y la inserción de Windows **aún no se han probado en un PC real**). Wayland: solo portapapeles.

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
Instalar el `.deb`: `sudo apt install ./src-tauri/target/release/bundle/deb/VoxFlow_0.3.0_amd64.deb`. El `.AppImage` se ejecuta con `chmod +x` y doble clic. Convertir audio: `ffmpeg -i in.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.

Preferencias: `~/.config/pe.voxflow.app/settings.json` (sin audio ni texto). Modelos instalados desde la app: `~/.local/share/pe.voxflow.app/models/`.

**Primer uso:** Preferencias → elegir micrófono → *Descargar modelo* (única conexión de red, con SHA-256) → Guardar → pestaña Dictado → *Hablar con VoxFlow* → *Detener* → revisar/editar → *Copiar* y pegar con Ctrl+V.

**Pruebas con hardware/modelo** (ignoradas por defecto): `cd src-tauri && VOXFLOW_MODELS_DIR=../models cargo test --lib -- --ignored --nocapture`

No hay CI/CD: todo se compila y ejecuta en local con `build.sh` y `run.sh`.

## Windows (10/11)

Instalador `VoxFlow_x.y.z_x64-setup.exe` (Inno Setup) generado por GitHub Actions: pestaña *Actions* → último run *Windows* → artefacto `voxflow-windows`, o adjunto al release de esa versión. **No está firmado**: SmartScreen mostrará "Windows protegió su PC" → *Más información* → *Ejecutar de todos modos*. Verifica `SHA256SUMS-windows` con `Get-FileHash`.
- **Se instala en** `C:\Program Files\VoxFlow` (pide permisos de administrador). Instala WebView2 si falta (descarga el instalador oficial de Microsoft). Requiere una CPU con **AVX2** (casi cualquier PC desde ~2013).
- **Registro de diagnóstico:** `C:\Program Files\VoxFlow\logs\voxflow.log` (si no se puede escribir ahí, `%LOCALAPPDATA%\pe.voxflow.app\logs`). Registra el arranque, la CPU, el micrófono, los pasos de grabación/transcripción, errores, pánicos y **excepciones nativas** (p. ej. `0xC000001D` instrucción ilegal). **Nunca** guarda audio ni texto dictado. Preferencias → Diagnóstico muestra la ruta. Si algo falla, adjunta este archivo.
- **Primer uso:** Preferencias → *Descargar modelo* (~142 MB) → elegir micrófono → Guardar. Si no graba: Configuración → Privacidad y seguridad → Micrófono → permitir apps de escritorio.
- **Atajo global:** por defecto `Ctrl+Alt+V` (editable en Preferencias); primera pulsación graba, la segunda detiene y transcribe. También `voxflow.exe --toggle`.
- **Insertar en la ventana activa:** modo de salida *Insertar* → pega con Ctrl+V (`SendInput`) solo si la ventana en primer plano sigue siendo la misma; si no, copia y avisa. Windows no permite enviar teclas a ventanas ejecutadas como administrador (en ese caso solo se copia).
- Datos: `%APPDATA%\pe.voxflow.app` (ajustes) y `%LOCALAPPDATA%\pe.voxflow.app\models`. Desinstalar: *Configuración → Aplicaciones* (borra `logs`; los ajustes y modelos se conservan).
- Compilar en local (Windows): Rust (MSVC), Node 22, pnpm, CMake, LLVM (`LIBCLANG_PATH`), *Build Tools* de VS e Inno Setup 6. Variables para whisper.cpp portable: `GGML_NATIVE=OFF GGML_AVX2=ON GGML_FMA=ON GGML_F16C=ON GGML_AVX512=OFF`. Luego `pnpm install`, `pnpm tauri build --no-bundle` y `iscc /DAppVersion=x.y.z installer\voxflow.iss`.

## Atajo de teclado en Linux (`voxflow --toggle`)

`voxflow --toggle` alterna la grabación en la instancia en marcha (o abre la app y empieza a grabar si no hay ninguna): 1.ª vez graba, 2.ª detiene y transcribe. No roba el foco; avisa con notificaciones (que nunca incluyen el texto). Según el **modo de salida**: *Copiar directo* → copia y notifica "Pega con Ctrl+V"; *Vista previa* → abre la ventana con el texto.

**GNOME** (script que solo añade/quita su propia entrada y conserva tus otros atajos):
```bash
scripts/install-gnome-shortcut.sh                          # Super+Alt+V → voxflow --toggle (.deb instalado)
scripts/install-gnome-shortcut.sh --binding '<Ctrl><Alt>d' --command /ruta/VoxFlow_0.3.0_amd64.AppImage
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

## Publicar una versión (mantenedor)
1. Sube la versión en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`; actualiza `CHANGELOG.md` y `docs/RELEASE_NOTES_x.y.z.md`.
2. `./build.sh` (Linux) → `.deb`, `.AppImage` y `SHA256SUMS` en `src-tauri/target/release/bundle/`.
3. Crea el tag y el release de GitHub (`gh release create vX.Y.Z --prerelease ...`) con esos archivos.
4. Empuja a la rama **`release`** (`git push origin main:release`): **solo ese push dispara GitHub Actions**, que compila el instalador de Windows y lo adjunta al release `vX.Y.Z`.

## Licencia
[MIT](LICENSE) © 2026 Luis Mayta. Licencias de dependencias y modelos: [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## Marca
Logo y línea gráfica en `assets/brand/` (paleta: #1A0F6B, #2E3BE6, #00D4FF, #00E6C3; tipografías Montserrat/Inter).
