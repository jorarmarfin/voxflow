# Decisiones y riesgos — Fase 0 (2026-11-23)

Entorno de prueba: Ubuntu 24.04.5, sesión **X11**, 8 núcleos CPU (sin GPU usada), rustc 1.99.1, node 24, cmake 3.28.

## Licencias verificadas (a la fecha)
| Componente | Versión | Licencia |
| --- | --- | --- |
| whisper.cpp | v1.9.6 | MIT |
| whisper-rs (bindings) | 0.16.0 | Unlicense |
| hound (WAV) | 3.5.1 | Apache-2.0 |
| Modelos ggml de OpenAI Whisper | base | MIT (pesos de OpenAI Whisper). Revisar antes de distribuir comercialmente |

Pendiente: licencias de Tauri 2 y dependencias de UI (Fase 1); licencia propia del repo.

## D1 — Bindings `whisper-rs` (in-process) en vez de proceso auxiliar
Compila y transcribe con éxito. Mantiene la interfaz `SpeechEngine` sustituible. Requiere `libclang` en build (se usa `LIBCLANG_PATH`). Si el binding se queda sin mantenimiento, fallback: invocar `whisper-cli` como proceso.

## D2 — Modelo y descarga
`ggml-base.bin` desde Hugging Face (`ggerganov/whisper.cpp`), SHA-256 `60ed5bc3…2efe` verificado contra `x-linked-etag`. Descarga a `.part`, se mueve solo si el checksum coincide (cumple criterio 7 del SPEC).

## Resultado medido
`tests/fixtures/jfk.wav` (11.0 s, **inglés**, 16 kHz mono) con `base`, CPU: carga 0.11 s, inferencia ~2.0 s, RTF ~0.19. Transcripción correcta.
**Limitación:** no se ha medido español, porque no hay audio en español en el repo ni TTS local. Falta ejecutar `scripts/record-test.sh` y validar español peruano/tecnicismos. Un solo equipo y un solo clip: no son objetivos de rendimiento.

## Estado de verificación de integración Linux
| Aspecto | Estado |
| --- | --- |
| Compilación whisper.cpp vía cmake | Probado |
| Transcripción WAV offline (modelo local) | Probado (en inglés) |
| Captura micrófono con `pw-record` (PipeWire) | Herramienta presente; **no probada con habla real** |
| Clipboard Wayland (`wl-copy`) / X11 (`xclip`) | `wl-copy` presente, `xclip` no instalado; **no probado** |
| Atajo global / inserción en Wayland | **No probado** (la sesión actual es X11). No se asume soporte |
| `ydotool` | No instalado; requeriría `uinput`/privilegios, no se configura |
| Librerías Tauri 2 | Presentes: webkit2gtk-4.1, gtk-3, appindicator, rsvg. Faltan `libasound2-dev`, `libpipewire-0.3-dev`, `libclang-dev` (requieren sudo) |

## Riesgos
- Captura de audio: elegir crate (cpal/ALSA vs PipeWire) en Fase 2; `libasound2-dev` falta en este equipo.
- Wayland: probar en una sesión Wayland real antes de afirmar nada (Fase 3).
- Latencia en CPU con modelos mayores que `base`: medir antes de fijar objetivos.

# Fase 1 — Esqueleto (2026-11-23)

- **Stack:** Tauri 2.x + React 19 + Vite 7 + TypeScript; paquetes con pnpm. Sin CI/CD por decisión del usuario: solo `run.sh` y `build.sh`.
- **Sin simulaciones:** `ENGINE_AVAILABLE=false` en `src-tauri/src/lib.rs`. Iniciar grabación devuelve un error explícito y la UI deshabilita el botón. Se activa en la Fase 2.
- **Estados:** `RecState` (IDLE/RECORDING/TRANSCRIBING/READY) en `state.rs`, con tests (sin grabaciones simultáneas, cancelar vuelve a IDLE).
- **Ajustes:** `settings.rs`, JSON con escritura atómica, validación de modelo/idioma (no se aceptan rutas arbitrarias) y fallback a defaults si el archivo está corrupto.
- **Seguridad:** capabilities mínimas (`core:default`), CSP restrictiva, sin comandos shell expuestos al frontend, sin fuentes web externas (Montserrat/Inter solo si están instaladas; fallback a la fuente del sistema).
- **Tray:** Abrir / Iniciar-Detener / Salir. Inserción de texto: botón "Insertar" deshabilitado hasta la Fase 3.
- **Icono:** generado desde `assets/brand/logo.png` (con fondo blanco; pendiente versión transparente para la bandeja).

## Verificado
`./build.sh --check` OK (clippy `-D warnings`, 7 tests Rust, 1 test Vitest). `.deb` (3.5 MiB) y `.AppImage` (81 MiB) generados. El binario release arranca y se mantiene vivo 10 s sin errores en stderr.
## No verificado
Aspecto visual de la ventana y comportamiento real de la bandeja en GNOME (requiere la extensión AppIndicator); instalación del `.deb` y ejecución del AppImage en un sistema limpio; Wayland.

# Fase 2 — Voz (2026-11-23)

- **AudioCapture:** `pw-record` (PipeWire) como proceso hijo con argumentos fijos, PCM 16 kHz mono s16 por stdout, solo en memoria (tope 5 min). Se eligió esto porque `libasound2-dev` no estaba instalado y requería sudo; evita dependencias de compilación y cumple 16 kHz nativo. Micrófonos con `pactl list short sources` (`LC_ALL=C`; el JSON de pactl falla con locale no ASCII), sin monitores. IDs validados (`[A-Za-z0-9._-:]`, sin `-` inicial). Limitación: requiere PipeWire/Pulse; no hay captura ALSA pura.
- **SpeechEngine:** trait + `WhisperEngine` (whisper-rs). Contexto cacheado por modelo; threads = min(CPU, 8); audio < 0,3 s se rechaza; < 1 s se rellena con silencio.
- **Bug upstream:** `set_abort_callback_safe` de whisper-rs 0.16.0 interpreta mal su puntero (error -6 espurio). Se usa la API cruda con trampolín propio sobre un `AtomicBool`. La cancelación además descarta el resultado por `generation`.
- **Privacidad de logs:** `install_logging_hooks()` (feature `log_backend`) desvía la salida de whisper.cpp (que imprime tokens) al crate `log`, sin logger. Verificado: stderr vacío durante la transcripción.
- **ModelManager:** descarga con `ureq` a `.part`, SHA-256 en streaming, rename solo si coincide; progreso, cancelación y borrado. Estado "instalado" = archivo presente (solo se crea tras verificar). Solo URL fija de Hugging Face. `VOXFLOW_MODELS_DIR` permite usar otra carpeta (la usa `run.sh`).
- **Estados:** READY → nuevo dictado permitido. Cancelar libera el micrófono (mata el proceso) y vuelve a IDLE. Tooltip de bandeja indica GRABANDO.

## Verificado
- `./build.sh` completo (clippy -D warnings, 14 tests Rust, 1 Vitest, `.deb` y `.AppImage`).
- Test ignorado con hardware: captura real de 2 s del micrófono por defecto (~1,9 s de muestras recibidas).
- Test ignorado con modelo: `jfk.wav` transcrito correctamente por el `WhisperEngine`.
- El binario release arranca sin errores.
## No verificado
- **Flujo completo con la UI** (clic → habla → texto) y habla real en **español peruano**/tecnicismos: requiere una prueba manual del usuario.
- Descarga desde la app (solo se probó `fetch-model.sh`; el código de `models.rs` no se ejercitó contra la red) y fallo de red/espacio.
- Desconexión del micrófono durante la grabación (se detecta solo si no llega audio).
- Rendimiento: solo 1 clip (RTF ~0,19 con `base`, CPU); sin objetivos definidos.
