# Changelog

Formato: [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) · Versionado semántico.

## [0.3.0] — 2026-11-23 (pre-release)

### Añadido
- **Windows (experimental, sin probar en equipo real):** captura WASAPI, portapapeles nativo, atajo global configurable (`Ctrl+Alt+V`), inserción con `SendInput` e instalador NSIS generado por GitHub Actions.
- Workflow de GitHub Actions **solo para Windows**, que se dispara únicamente con commits en la rama `release` y adjunta el instalador al release `v<versión>`.
- Licencia MIT (`LICENSE`) y `THIRD_PARTY_LICENSES.md`.

### Limitaciones conocidas
- Windows: micrófono, atajo, pegado, notificaciones e instalador sin probar en un PC real; instalador sin firmar (SmartScreen avisará); no se inserta en ventanas con permisos de administrador.

## [0.2.0] — 2026-11-23 (pre-release)

### Añadido
- Modo de salida **Insertar en la ventana activa** (X11 + `xdotool`/`xprop`) con guarda de ventana, detección de terminales (Ctrl+Shift+V) y restauración del portapapeles de texto. `.deb`: `Recommends: xdotool, x11-utils`.

## [0.1.0] — 2026-11-23 (prueba / pre-release)

### Añadido
- Dictado local en español con whisper.cpp (modelo `base`), sin enviar audio ni texto a Internet.
- Captura de micrófono vía PipeWire (`pw-record`), selección de dispositivo, cancelación que libera el micrófono.
- Ventana (Tauri 2 + React) con estado, vista previa editable, Copiar / Insertar / Descartar y Preferencias persistentes.
- Bandeja del sistema (Abrir, Iniciar/Detener, Salir).
- `voxflow --toggle` para atajos del escritorio; `scripts/install-gnome-shortcut.sh`.
- Portapapeles Wayland (`wl-copy`) y X11 (`xclip`/`xsel`); notificaciones sin texto dictado.
- Gestor de modelos: descarga explícita con progreso, cancelación, SHA-256 y borrado.
- Paquetes `.deb` y `.AppImage`; scripts `run.sh`, `build.sh`.

### Limitaciones conocidas
- **Sin inserción automática** de texto: se copia y se pega con Ctrl+V.
- **Wayland no validado** en sesión real (desarrollo y pruebas en X11).
- Solo modelo `base`; sin historial, diccionario, modos de pulido ni GPU (P1/P2 del spec).
- Requiere PipeWire/PulseAudio (`pw-record`, `pactl`).
- Se probó con una sola máquina; no hay objetivos de rendimiento definidos.
