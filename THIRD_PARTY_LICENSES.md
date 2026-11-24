# Licencias de terceros

VoxFlow se distribuye bajo licencia [MIT](LICENSE). Resumen de componentes (revisión del 2026-11-23, versión 0.2.0). No sustituye a las licencias originales, que cada paquete incluye en su código fuente.

## Componentes principales
| Componente | Licencia |
| --- | --- |
| whisper.cpp (vía `whisper-rs-sys`) | MIT |
| `whisper-rs` | Unlicense |
| Modelos Whisper de OpenAI (se descargan aparte, `ggml-base.bin`) | MIT |
| Tauri 2 y plugins | MIT OR Apache-2.0 |
| React, React DOM | MIT |
| `hound`, `ureq`, `sha2`, `serde` | MIT OR Apache-2.0 |

## Solo Windows
`cpal` (Apache-2.0), `arboard` (MIT OR Apache-2.0), `windows-sys` (MIT OR Apache-2.0), `tauri-plugin-global-shortcut` y `tauri-plugin-notification` (Apache-2.0 OR MIT). Pendiente revisar el árbol completo de licencias con el target Windows.

## Dependencias Rust (≈520 crates, `cargo metadata`)
Casi todas MIT, Apache-2.0, "MIT OR Apache-2.0", BSD-3-Clause, ISC, Zlib, Unicode-3.0, Unlicense, 0BSD o CC0. Sin GPL/AGPL.
- **MPL-2.0** (copyleft a nivel de archivo, usadas sin modificar): `cssparser`, `cssparser-macros`, `selectors`, `dtoa-short`, `option-ext`.
- **`r-efi`**: triple licencia `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; se usa bajo MIT.

## Bibliotecas del sistema (enlace dinámico, no se redistribuyen)
GTK 3, WebKitGTK 4.1, libayatana-appindicator (LGPL/GPL-compatibles) se instalan desde los repositorios de la distribución como dependencias del `.deb`.
El `.AppImage` empaqueta estas bibliotecas del sistema, cada una con su propia licencia (en su mayoría LGPL).

## Herramientas externas (no incluidas; se ejecutan si están instaladas)
`pw-record`, `pactl`, `wl-copy`, `xclip`, `xsel`, `xdotool`, `xprop`, `notify-send`.

## Modelos
El modelo de voz no está en el repositorio ni en los instaladores: se descarga a petición del usuario desde Hugging Face (`ggerganov/whisper.cpp`). Revisa su licencia si lo redistribuyes.
