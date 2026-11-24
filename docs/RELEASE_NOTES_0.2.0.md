# VoxFlow 0.2.0 (prueba)

Pre-release: añade **inserción del texto en la posición del cursor** (X11). Sigue siendo una versión de prueba, no estable.

## Novedades
- Nuevo modo de salida **Insertar en la ventana activa** (Preferencias). Se usa con el atajo `voxflow --toggle`: al empezar a grabar se guarda la ventana activa y, al terminar, el texto se pega allí (Ctrl+V; Ctrl+Shift+V en terminales).
- **Guarda de seguridad:** si cambias de ventana mientras dictas, o no se puede confirmar el destino, **no se escribe nada**; el texto queda copiado y se avisa del motivo.
- Restaura el texto que tenías antes en el portapapeles tras pegar (solo texto).
- Diagnóstico en Preferencias indica si la inserción está disponible.

## Requisitos de la inserción
Sesión **X11**, `xdotool` y `xprop` (`x11-utils`). El `.deb` los *recomienda*; sin ellos todo funciona por portapapeles.

## Probado / no probado
- Probado (X11, Ubuntu 24.04): campo de texto Tk, cambio de ventana (no inserta), gnome-terminal con la app completa.
- **Sin probar:** VS Code, Chrome/Firefox, apps Electron, Claude CLI en distintas terminales, Wayland (solo portapapeles), teclados no estándar. xterm no pega con Ctrl+Shift+V por defecto.
- No lo uses con campos de contraseña: VoxFlow no sabe qué hay en el campo enfocado.

## Instalación
```bash
sudo apt install ./VoxFlow_0.2.0_amd64.deb      # o: chmod +x VoxFlow_0.2.0_amd64.AppImage
sha256sum -c SHA256SUMS --ignore-missing
```
Atajo GNOME: `scripts/install-gnome-shortcut.sh` (repositorio). Actualizar desde 0.1.0: instalar encima; los ajustes se conservan.
Desinstalar: `sudo apt remove voxflow`.

## Heredado de 0.1.0
Sin LICENSE aún; Wayland sin validar; solo modelo `base`; sin historial ni telemetría. Ver [CHANGELOG](../CHANGELOG.md).
