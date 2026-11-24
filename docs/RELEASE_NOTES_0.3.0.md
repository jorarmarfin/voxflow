# VoxFlow 0.3.0 (prueba)

Pre-release: primera versión con **soporte para Windows** y con **licencia MIT**. Sigue siendo una versión de prueba.

## Novedades
- **Windows 10/11 (experimental):** captura de micrófono (WASAPI), portapapeles nativo, atajo global configurable (`Ctrl+Alt+V` por defecto), inserción en la ventana activa (`SendInput`) e instalador `VoxFlow_0.3.0_x64-setup.exe`.
- **Licencia MIT** (`LICENSE`, `THIRD_PARTY_LICENSES.md`); el `.deb` la incluye.
- Linux sin cambios funcionales respecto a 0.2.0 (inserción en el cursor en X11 incluida).

## Archivos
| Archivo | Sistema |
|---|---|
| `VoxFlow_0.3.0_amd64.deb`, `VoxFlow_0.3.0_amd64.AppImage` + `SHA256SUMS` | Ubuntu 24.04 / Linux |
| `VoxFlow_0.3.0_x64-setup.exe` + `SHA256SUMS-windows` | Windows (lo compila GitHub Actions y lo adjunta a este release unos minutos después de publicarlo) |

## Windows: lee esto
- **Sin probar en un PC real.** Compila y pasa sus tests en GitHub Actions, pero el micrófono, el atajo global, el pegado, las notificaciones y el instalador no se han ejecutado en un equipo Windows.
- **Instalador sin firmar:** SmartScreen mostrará "Windows protegió su PC" → *Más información* → *Ejecutar de todos modos*. Verifica el checksum con `Get-FileHash`.
- Instalación por usuario (sin administrador). Si no graba: Configuración → Privacidad y seguridad → Micrófono.
- Insertar no funciona en ventanas ejecutadas como administrador (solo se copia).

## Linux
```bash
sudo apt install ./VoxFlow_0.3.0_amd64.deb
sha256sum -c SHA256SUMS --ignore-missing
```
Atajo GNOME: `scripts/install-gnome-shortcut.sh` (repositorio). Probado en X11; Wayland solo con portapapeles. No lo uses con campos de contraseña en modo *Insertar*.

Ver [CHANGELOG](../CHANGELOG.md) y [README](../README.md).
