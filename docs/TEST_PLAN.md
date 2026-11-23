# Plan de pruebas

## Fase 0 (ejecutado)
1. `./build.sh` → fmt + clippy sin avisos + build release. ✔
2. `scripts/fetch-model.sh base` → checksum verificado; reejecución idempotente. ✔
3. `./run.sh tests/fixtures/jfk.wav en` → texto esperado (JFK) y métricas. ✔
4. Español: `scripts/record-test.sh` + `./run.sh`. ⏳ pendiente (requiere voz real).
5. Prueba negativa: WAV con formato inválido debe fallar con mensaje claro. (cubierta por validación en `main.rs`)

## Próximas fases
Ver criterios de aceptación en `docs/SPEC.md` §10. Pruebas manuales Wayland/X11 documentadas en Fase 3.
