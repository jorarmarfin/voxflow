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
| Clipboard Wayland (`wl-copy`) / X11 (`xclip`) | `wl-copy` y `xclip` presentes (corrección: `xclip` sí estaba, `--version` no existe en xclip); X11 probado en Fase 3 |
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

# Fase 3 — Integración Linux (2026-11-23)

- **`voxflow --toggle`:** `tauri-plugin-single-instance` (D-Bus): una 2.ª ejecución reenvía sus argumentos a la instancia en marcha y termina. En arranque en frío con `--toggle` la app inicia oculta y empieza a grabar. No roba el foco; el estado se comunica por notificaciones (`notify-send`, sin texto dictado) y tooltip de bandeja. La ventana arranca oculta (`visible:false`) y se muestra salvo `start_minimized`/`--toggle`.
- **Atajo GNOME:** `scripts/install-gnome-shortcut.sh` (gsettings) añade/quita solo su entrada. Probado con backend `keyfile` aislado (crear, idempotencia, quitar), sin tocar la config real. No es configuración silenciosa: la ejecuta el usuario.
- **Portapapeles:** herramientas del sistema con el texto por stdin (nunca argv). Wayland → `wl-copy` (funciona en GNOME, que no soporta wlr-data-control, por lo que se descartó `arboard`/plugin de Tauri); X11 → `xclip`/`xsel`. `.deb` depende de `wl-clipboard | xclip`, `libnotify-bin`, `pipewire-bin`, `pulseaudio-utils`.
- **Inserción automática: NO implementada** (`InsertResult::Unsupported`). No se puede garantizar la ventana de destino sin permisos extra: en Wayland no hay API general; en X11 se evaluó `xdotool` (no instalado aquí, no probado) con guarda de ventana activa. El botón *Insertar* copia el texto y avisa de pegar con Ctrl+V (cumple criterio 5). `ydotool`/`uinput` descartado: exige privilegios.
- **Diagnóstico** en Preferencias: sesión (X11/Wayland), herramienta de portapapeles, notificaciones, inserción.

## Verificado (X11, Ubuntu 24.04 GNOME, release)
- Instancia en marcha + `voxflow --toggle` ×2: 1.ª → `pw-record` activo; 2.ª → se detiene, el micrófono se libera, se transcribe y (modo *clipboard*) el texto aparece en el portapapeles. 3 ciclos consecutivos OK; una sola instancia; stderr sin texto dictado.
- Arranque en frío con `--toggle`: empieza a grabar con una sola instancia; al terminar el proceso no queda `pw-record`.
- Test ignorado: copia y relectura del portapapeles con `xclip`.
- Tests unitarios: detección de sesión, orden de herramientas, parseo de `--toggle`. `./build.sh` OK (18 tests Rust + 1 Vitest), `.deb` con `Depends` correctos.
## No verificado
- **Wayland (sesión real):** `wl-copy`, atajo, foco de ventana. Solo hay lógica y orden de herramientas con tests unitarios.
- Atajo GNOME real con la pulsación de teclas (solo el script sobre backend aislado).
- Instalación del `.deb` en un sistema limpio; AppImage con `--toggle`.
- Notificaciones: se invoca `notify-send`, no se comprobó visualmente que aparezcan.
- Carrera: si `--toggle` se lanza antes de que la 1.ª instancia haya registrado su D-Bus, puede iniciarse una 2.ª instancia (un intento anterior sin sleep suficiente falló sin causa identificada).

# Fase 4 — Calidad y entrega (2026-11-23)
- Revisión de privacidad de artefactos: el primer binario incrustaba rutas locales (usuario incluido). Se añadió `strip`, `lto=thin` y `--remap-path-prefix` en `build.sh`; el `.deb` final no contiene usuario/correo. Pendiente menor: ruta de build de whisper.cpp.
- `build.sh` genera `SHA256SUMS` junto a los paquetes.
- Informe de aceptación: `docs/ACCEPTANCE.md` (varios criterios parciales por falta de Wayland/VM limpia/audio en español). Sin LICENSE todavía: decisión del propietario.
- Release 0.1.0 como **pre-release** de prueba; sin datos personales ni secretos.

# Fase 5 — Inserción en el cursor, X11 (2026-11-23)

- **Modo `Insert`** (`OutputMode::Insert`), disponible si `session==X11 && xdotool && xprop`. Solo se activa con el atajo/bandeja: la ventana activa al iniciar la grabación es el destino. Si la activa es la propia VoxFlow (grabación desde el botón) no hay destino y se hace fallback a portapapeles. No es el modo por defecto.
- **Guarda:** ID de ventana (validado: solo dígitos) al iniciar; al terminar se compara con la activa; ventana distinta, clase de ventana desconocida o fallo de `xdotool` → no se escribe, el texto queda copiado y se notifica el motivo (sin el texto).
- **Teclas:** `xdotool key --clearmodifiers ctrl+v` (o `ctrl+shift+v` si la clase `WM_CLASS` es de una terminal conocida). Clase vía `xprop` porque `xdotool getwindowclassname` no existe en xdotool 3.20160805 (Ubuntu 24.04); se descubrió en pruebas.
- **Portapapeles:** se lee el texto previo, se copia la transcripción, se pega y a los 400 ms se restaura. Limitación: no se restauran imágenes/otros formatos.
- **Descartado:** `xdotool type` (lento y rompe con Unicode/teclados), `ydotool`/`uinput` (privilegios), Wayland (sin API general; pendiente portal libei).
- `xdotool`/`x11-utils` son `Recommends`, no `Depends`: sin ellos todo sigue funcionando por portapapeles.

## Verificado (X11, release, xdotool 3.20160805 extraído sin sudo)
- Tk Entry: transcripción insertada y portapapeles restaurado a su valor previo.
- Cambio de ventana durante la grabación (2 ventanas): **no se insertó en ninguna**; texto copiado; notificación con el motivo.
- gnome-terminal: clase detectada (`Gnome-terminal`), Ctrl+Shift+V inserta el texto; flujo completo con la app OK.
- xterm: Ctrl+Shift+V **no** pega por defecto (no es un terminal "estándar" en esto): no se garantiza para xterm.
- Unit tests: guarda, clases de terminal, ids. `./build.sh` OK. stderr vacío.
## No verificado
VS Code (terminal integrada y editor), Chrome/Firefox, apps Electron/Java; campos que bloquean el pegado; Wayland/XWayland; teclados con distribución no estándar (xdotool usa keysyms, no se probó); rendimiento con textos largos.

# Licencia (2026-11-23)
MIT, a petición del propietario (repositorio público). Revisión de dependencias con `cargo metadata`/`pnpm licenses`: sin GPL/AGPL; MPL-2.0 en 5 crates (sin modificar) y `r-efi` bajo MIT. Resumen en `THIRD_PARTY_LICENSES.md`. Los releases v0.1.0/v0.2.0 ya publicados no incluían el archivo LICENSE; se aplica al repositorio desde este commit y a los siguientes releases.

# Fase 6 — Windows (2026-11-23)

- **Arquitectura por plataforma** con `cfg`: `audio/{pipewire,wasapi,resample}`, `output/{unix,windows}`, `insert/{x11,windows}`. API común idéntica; la UI, la máquina de estados, los ajustes, el gestor de modelos y el motor (whisper.cpp) se reutilizan.
- **Captura:** `cpal` 0.18 (WASAPI) con el stream en un hilo propio, formato nativo (f32/i16/i32/u16) → mono → remuestreo lineal propio a 16 kHz (`audio/resample.rs`, con tests que corren en CI de Windows y Linux). El audio sigue solo en memoria (tope de 5 min).
- **Portapapeles:** `arboard` (con reintentos). **Notificaciones:** `tauri-plugin-notification` (Rust-side, sin texto dictado).
- **Atajo global:** `tauri-plugin-global-shortcut`, ajuste `hotkey` (def. `Ctrl+Alt+V`), validado y registrado al guardar. En Linux se mantiene el atajo del escritorio + `--toggle` (en Wayland no hay escucha global).
- **Inserción:** `SendInput` Ctrl+V, misma guarda de ventana (HWND en primer plano) y espera a que se suelten Shift/Alt/Win. Limitación UIPI con ventanas elevadas.
- **Empaquetado:** NSIS por usuario (`--bundles nsis`), sin firma de código → SmartScreen avisa. MSI/WiX descartado por ahora.
- **CI:** `.github/workflows/windows.yml` compila **solo Windows** (frontend lint/test, `cargo test --lib`, NSIS, checksums, artefacto; solo con commits en la rama `release`; adjunta el instalador al release `v<versión>` si existe). Linux sigue siendo local (`build.sh`); esto sustituye la decisión previa de "sin CI" únicamente para Windows, a petición del propietario.

## Verificación
Ver estado de CI y pruebas pendientes en `docs/TEST_PLAN.md`. Todo lo que dependa de hardware/SO real de Windows (micrófono, atajo, SendInput, notificaciones, instalador) está **sin probar**.
