# Plan de pruebas

## Fase 0 (ejecutado)
1. `./build.sh` → fmt + clippy sin avisos + build release. ✔
2. `scripts/fetch-model.sh base` → checksum verificado; reejecución idempotente. ✔
3. `./run.sh tests/fixtures/jfk.wav en` → texto esperado (JFK) y métricas. ✔
4. Español: `scripts/record-test.sh` + `./run.sh`. ⏳ pendiente (requiere voz real).
5. Prueba negativa: WAV con formato inválido debe fallar con mensaje claro. (cubierta por validación en `main.rs`)

## Próximas fases (ver abajo)
Ver criterios de aceptación en `docs/SPEC.md` §10. Pruebas manuales Wayland/X11 documentadas en Fase 3.

## Fase 1 (ejecutado)
1. `./build.sh --check` → fmt, clippy, 7 tests Rust (estados, ajustes), 1 test Vitest. ✔
2. `./build.sh` → `.deb` y `.AppImage` generados. ✔
3. Arranque del binario release 10 s sin errores. ✔
4. Manual ⏳: abrir ventana, cambiar pestañas, guardar preferencias y comprobar `~/.config/pe.voxflow.app/settings.json`; menú de bandeja; botón grabar deshabilitado con aviso de Fase 2.

## Fase 2
Automáticas: `./build.sh` (14 Rust + 1 Vitest). Con hardware/modelo (`--ignored`): captura real y transcripción de `jfk.wav`. ✔
Manuales ⏳ (hacer antes de cerrar la fase):
1. Preferencias → *Descargar modelo* (borrando antes `~/.local/share/pe.voxflow.app/models`): progreso, instalación verificada; cortar la red a medias → no queda modelo inválido.
2. Dictar frase en español peruano con "Laravel", "React", "Rust"; frase larga (>30 s).
3. Cancelar durante grabación y durante transcripción → vuelve a IDLE y el micrófono se libera (`pactl list short source-outputs` vacío).
4. Desconectar/cambiar micrófono → error legible.
5. Verificar que `journalctl`/stderr no contienen texto dictado.

## Fase 3
Automáticas: `./build.sh` (18 Rust + 1 Vitest); `cargo test --lib -- --ignored` (clipboard X11, captura, transcripción).
Realizadas a mano (X11): ciclo `--toggle` ×3 con instancia en marcha; arranque en frío; liberación del micrófono. ✔
Pendientes ⏳:
1. Sesión **GNOME Wayland**: `voxflow --toggle` desde atajo real, `wl-copy`, notificaciones y comportamiento de `show_main`.
2. Instalar el `.deb` en VM limpia (dependencias) y probar el AppImage con `--toggle`.
3. Atajo real: `scripts/install-gnome-shortcut.sh`, pulsar la combinación, dictar en VS Code/navegador y pegar.

## Fase 5 (X11)
Hechas: Tk Entry, cambio de ventana (negativa), gnome-terminal. Con `PATH`/`LD_LIBRARY_PATH` apuntando a un xdotool local si no está instalado.
Pendientes ⏳: VS Code (editor y terminal integrada), Chrome/Firefox (campo de texto y contenteditable), apps Electron, Wayland (debe quedar en fallback a portapapeles).

## Fase 6 (Windows)
Automáticas (CI Windows): lint + Vitest, `cargo test --lib` (resampler, ajustes, guarda de ventana), build NSIS.
Manuales ⏳ en un PC con Windows 10/11:
1. Instalar el `.exe` (SmartScreen), abrir, descargar modelo, elegir micrófono.
2. Dictar con el botón y con `Ctrl+Alt+V`; comprobar que el micrófono se libera (icono de micrófono desaparece).
3. Modo *Insertar* en Bloc de notas, VS Code, Chrome, Windows Terminal; cambio de ventana durante la grabación (no debe insertar); ventana como administrador (solo copia).
4. Notificaciones (sin texto dictado), cambio de atajo, atajo en uso por otra app.
5. Español peruano y tecnicismos; rendimiento CPU (RTF).

## 0.3.1 (Windows): diagnóstico del cierre al detener
1. Instalar el `.exe` de Inno Setup; comprobar que existe `C:\Program Files\VoxFlow\logs\voxflow.log`.
2. Dictar y pulsar *Detener*. Si se cierra: abrir el log y buscar `PANIC`, `EXCEPCION NATIVA` y la última línea de `grabación…`/`motor…`/`transcripción…`; adjuntarlo (no contiene texto dictado).
3. Revisar la línea `CPU: avx2=… avx512f=…`.
