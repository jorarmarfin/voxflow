# AGENTS.md — VoxFlow

Eres el agente responsable de implementar VoxFlow, una aplicación de dictado local para Ubuntu.

## Fuente de verdad
Lee `docs/SPEC.md` completo antes de modificar código. Prioriza P0 y el orden de fases definido allí.

## Forma de trabajar
1. Inspecciona el repositorio antes de crear archivos.
2. Define un plan breve de la fase actual y explica riesgos verificables.
3. Implementa cambios pequeños, compilables y probados.
4. No declares que Wayland soporta atajos globales o escritura universal sin pruebas reales.
5. Prefiere servicios Rust desacoplados, frontend React simple y mínimo privilegio.
6. Mantén transcripción y audio locales. No agregues telemetría ni servicios externos sin autorización.
7. Nunca guardes transcripciones en logs ni almacenes el audio por defecto.
8. Si una API o librería falla, documenta el motivo y usa un fallback seguro, no una simulación engañosa.
9. Ejecuta formato, lint y pruebas relevantes al final de cada cambio, registrando qué pudo y qué no pudo verificarse.
10. Mantén README, decisiones y pruebas alineadas con el código.

## Primera tarea
Implementa la Fase 0 de `docs/SPEC.md`: verifica stack/licencias y crea una prueba de concepto mínima de transcripción local con `whisper.cpp` para Ubuntu 24.04. Presenta resultados reproducibles antes de pasar a la UI.

## Definición de terminado
Código y documentación coherentes, pruebas ejecutadas, limitaciones explícitas y sin comprometer privacidad o seguridad.
