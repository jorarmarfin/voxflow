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
