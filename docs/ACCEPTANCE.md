# Aceptación MVP (SPEC §10) — 0.1.0, 2026-11-23

Entorno: Ubuntu 24.04.5 GNOME **X11**, 8 CPU, 23 GB RAM, sin GPU. Wayland no disponible.

| # | Criterio | Estado | Evidencia / pendiente |
|---|---|---|---|
| 1 | `.deb` instala, inicia y reconoce el micrófono (GNOME Wayland) | ◐ Parcial | `.deb` generado con `Depends` correctos; binario extraído arranca; mic detectado (`pactl`) y captura real OK. **Falta** instalar en VM limpia y probar en Wayland |
| 2 | Sin Internet transcribe un audio en español con modelo instalado | ◐ Parcial | Transcripción sin sockets de red (`strace`: 0 `AF_INET`); solo `models.rs` usa red. **Falta** clip en español peruano y prueba con la red realmente cortada (`unshare -n` no permitido aquí) |
| 3 | Activar desde botón y desde atajo GNOME con `voxflow --toggle` | ◐ Parcial | Botón y `--toggle` ×3 OK; script del atajo OK en entorno aislado. **Falta** pulsar el atajo real |
| 4 | Revisar, editar y copiar | ✔ | Vista previa editable; copia verificada con `xclip` |
| 5 | Sin inserción compatible no se pierde el texto | ✔ | *Insertar* copia y avisa; modo *clipboard* copia directo |
| 6 | Indicador visible; detener/cancelar libera el dispositivo | ✔ | "Micrófono ACTIVO", tooltip de bandeja, `pw-record` desaparece al detener/cancelar/terminar la app |
| 7 | Descarga fallida no deja modelo inválido | ◐ Parcial | Lógica `.part` + SHA-256 + rename, test de modelo desconocido y script con checksum. **Falta** probar corte de red/checksum corrupto en la app |
| 8 | Sin historial, nada persiste ni aparece en logs | ✔ | Sin `println!`/log en código de la app (solo un test ignorado); whisper.cpp desviado a `log` sin logger; stderr vacío en ciclos `--toggle`; audio solo en memoria |
| 9 | Empaquetado con dependencias y desinstalación | ✔ | README + release notes; `Depends` del `.deb` |
| 10 | Pruebas reproducibles y limitaciones Wayland | ✔ | `./build.sh` (18 Rust + 1 Vitest), tests `--ignored`, `docs/TEST_PLAN.md`, limitaciones en README/CHANGELOG |

## Rendimiento (solo informativo; 1 equipo, 1 clip)
`base`, CPU, `jfk.wav` 11,0 s ×5: inferencia 2,09–2,30 s (RTF 0,19–0,21), carga del modelo 0,11 s, RSS ~330 MB. No se definen objetivos (SPEC §9); faltan clips en español y frases largas.

## Seguridad
- `pnpm audit --prod`: sin vulnerabilidades. `cargo audit`: sin vulnerabilidades; 6 avisos de crates sin mantenimiento/`glib` *unsound* (RUSTSEC-2024-0429, 2024-0370, 2026-0370/0371/0372/0543), heredados de Tauri/GTK3 y `zbus`; sin acción propia posible, a vigilar.
- CSP restrictiva, capabilities `core:default`, sin comandos shell expuestos a la UI, IDs de dispositivo validados, modelos solo desde URL fija + SHA-256, procesos hijos con argumentos fijos.
- Binarios con `strip` y `--remap-path-prefix`: sin usuario ni correo. Queda la ruta de compilación de whisper.cpp (`/mnt/DATOS/...`, sin datos personales) en el binario porque `-ffile-prefix-map` no se aplicó a esa parte.
- Notificaciones sin texto dictado.
