# VoxFlow 0.1.0 (prueba)

Dictado local en español para Ubuntu 24.04. Pre-release para validar el MVP; **no es una versión estable**.

## Instalación
```bash
# .deb (instala dependencias)
sudo apt install ./VoxFlow_0.1.0_amd64.deb
# AppImage
chmod +x VoxFlow_0.1.0_amd64.AppImage && ./VoxFlow_0.1.0_amd64.AppImage
sha256sum -c SHA256SUMS      # verificar descargas
```
Primer uso: Preferencias → *Descargar modelo* (~142 MB, verificado con SHA-256) → Guardar → Dictado.
Atajo: `voxflow --toggle` (GNOME: `scripts/install-gnome-shortcut.sh` del repositorio).
Desinstalar: `sudo apt remove voxflow` y, si quieres, `rm -r ~/.config/pe.voxflow.app ~/.local/share/pe.voxflow.app`.

## Privacidad
Audio solo en memoria; nada de audio ni texto en disco ni en logs; sin telemetría. Única conexión de red: descarga del modelo que tú inicias.

## Compatibilidad y limitaciones
- Probado en Ubuntu 24.04 **X11**. **Wayland: sin validar**; el atajo lo gestiona el escritorio.
- **Sin inserción automática**: el texto queda en el portapapeles (Ctrl+V).
- Requiere PipeWire/PulseAudio, `wl-clipboard` o `xclip`, `libnotify-bin`.
- Licencias: whisper.cpp (MIT), modelo Whisper de OpenAI (MIT), whisper-rs (Unlicense). Licencia del proyecto: MIT (desde el commit posterior a este release).

Ver [CHANGELOG](../CHANGELOG.md) y [DECISIONS](DECISIONS.md).
