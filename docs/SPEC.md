# VoxFlow — Especificación funcional y técnica

**Versión:** 1.0 (MVP)  
**Fecha:** 23 de noviembre de 2026  
**Plataforma objetivo:** Ubuntu 24.04 LTS (GNOME Wayland y X11)  
**Producto:** aplicación de dictado inteligente local, inspirada funcionalmente en Wispr Flow (sin copiar marca, interfaz ni código).

## 1. Visión

Permitir al usuario hablar en español y convertir su voz en texto de calidad, insertándolo en la aplicación donde esté trabajando (navegador, editor de código, mensajería o documentos). Priorizar privacidad, velocidad, control del usuario y funcionamiento sin Internet después de descargar el modelo.

### Objetivos del MVP
- Iniciar/detener grabación con una acción rápida y una alternativa visible desde la bandeja del sistema.
- Capturar audio del micrófono seleccionado y mostrar estado de grabación/procesamiento/error.
- Transcribir español con un motor local basado en `whisper.cpp`.
- Mostrar vista previa editable de la transcripción antes de insertarla cuando sea necesario.
- Copiar al portapapeles y ofrecer pegado/inserción asistida en la aplicación activa.
- Permitir configuraciones básicas y descargar/gestionar modelos de forma explícita.
- Distribuir `.deb` y `.AppImage` para Linux.

### No objetivos del MVP
- Compatibilidad móvil o Windows/macOS.
- Transcripción en tiempo real palabra por palabra.
- Sincronización en la nube, cuentas, suscripciones o servidor remoto.
- Corrección gramatical con un LLM como requisito del MVP.
- Prometer inyección universal de texto en todas las aplicaciones Wayland.

## 2. Usuarios y casos de uso

**Usuarios iniciales:** desarrolladores, redactores, docentes y usuarios frecuentes de asistentes de IA en Ubuntu.

1. El usuario se encuentra en VS Code o un navegador, activa VoxFlow, dicta y finaliza.
2. La aplicación transcribe localmente y muestra un aviso discreto de finalización.
3. Si la inserción es compatible, ofrece pegar el texto en el campo previamente enfocado; si no, copia al portapapeles y notifica cómo pegarlo.
4. El usuario puede revisar y corregir el resultado en un panel antes de enviarlo.
5. El usuario puede seleccionar micrófono, modelo e idioma desde preferencias.

## 3. Stack tecnológico

| Capa | Elección | Motivo |
| --- | --- | --- |
| Shell desktop | Tauri 2 | Integración desktop y empaquetado Linux |
| Interfaz | React + TypeScript + Vite | Ligereza y familiaridad |
| Lógica nativa | Rust estable | Audio, estado, archivos e integración Linux |
| STT | `whisper.cpp` mediante bindings Rust mantenidos o proceso auxiliar | Inferencia local; desacoplamiento del motor |
| Captura audio | Biblioteca Rust compatible con ALSA/PulseAudio/PipeWire, por evaluar | Audio de micrófono |
| Persistencia | JSON/TOML para ajustes; SQLite solo si el historial se incorpora | Simplicidad |
| Pruebas | Rust unit/integration, Vitest y pruebas manuales de escritorio | Cobertura incremental |
| Distribución | `.deb`, `.AppImage` | Ubuntu y Linux |

**Regla:** confirmar mantenimiento, licencia y compatibilidad de las bibliotecas antes de fijar versiones. Mantener la interfaz `SpeechEngine` para sustituir `whisper.cpp` sin cambiar la UI.

## 4. Arquitectura

```text
React UI (configuración, indicador, vista previa)
         │ comandos/eventos Tauri
         ▼
Rust Application Core
  ├── AudioCapture (dispositivos, grabación, formato PCM)
  ├── RecordingStateMachine
  ├── SpeechEngine (adaptador whisper.cpp)
  ├── TextOutput (clipboard / integración escritorio)
  ├── ModelManager (descarga, checksum, progreso)
  ├── SettingsStore
  └── Diagnostics (logs sin audio ni transcripciones)
         │
         ▼
Linux Desktop (PipeWire/PulseAudio, X11/Wayland, clipboard)
```

### Estados
`IDLE → RECORDING → TRANSCRIBING → PREVIEW/READY → IDLE`; errores recuperables vuelven a `IDLE`. Impedir grabaciones simultáneas y limpiar recursos al cancelar.

### Interfaces conceptuales
- `AudioCapture.start(device_id?) -> recording_id`
- `AudioCapture.stop(recording_id) -> audio_buffer`
- `SpeechEngine.transcribe(audio_buffer, language, model) -> transcript`
- `ModelManager.install(model_id) -> progress/result`
- `TextOutput.copy(transcript) -> result`
- `TextOutput.insert(transcript, target_context) -> success | unsupported | failure`

No exponer operaciones privilegiadas ni comandos shell arbitrarios al frontend.

## 5. Requisitos funcionales (priorizados)

### P0 — Obligatorios
**RF-01.** Detectar micrófonos, elegir dispositivo y manejar desconexiones/permisos.  
**RF-02.** Grabar audio mono con formato adecuado para STT; resamplear cuando proceda.  
**RF-03.** Activación/desactivación desde UI y bandeja; ofrecer comando CLI `voxflow --toggle` para asignarlo a un atajo del entorno de escritorio, especialmente en Wayland.  
**RF-04.** Transcripción local en español, sin enviar audio a Internet.  
**RF-05.** Vista previa editable, copiar al portapapeles y mecanismo de inserción seguro que no escriba en un destino inesperado.  
**RF-06.** Gestión de modelos: estado, tamaño, descarga explícita, verificación checksum, errores y eliminación.  
**RF-07.** Preferencias persistentes: dispositivo, modelo, idioma, modalidad de salida, inicio minimizado y atajo documentado.  
**RF-08.** Indicadores visuales claros de grabación y procesamiento; cancelar grabación y transcripción.  
**RF-09.** Logs técnicos mínimos, sin almacenar audio ni texto por defecto.  
**RF-10.** Instalables `.deb` y `.AppImage`, README de instalación y desinstalación.

### P1 — Después del MVP
**RF-11.** Modos de dictado: literal y texto pulido mediante reglas locales (puntuación y muletillas, evitando cambiar el significado).  
**RF-12.** Diccionario personalizado de nombres y términos técnicos.  
**RF-13.** Historial local opcional, desactivado por defecto, con borrado completo.  
**RF-14.** Activación push-to-talk cuando sea viable por plataforma.  
**RF-15.** Modelos alternativos y aceleración GPU opcional.

### P2 — Futuro
- Revisión con LLM local u opción de proveedor remoto con consentimiento explícito.
- Perfiles por aplicación y comandos de voz configurables.
- Soporte Windows y macOS tras estabilizar Linux.

## 6. Integración Linux: riesgos obligatorios

### Wayland
Wayland restringe la observación global de teclas, el seguimiento de foco y la escritura en otras aplicaciones. No prometer compatibilidad universal.

**Orden de implementación:**
1. Atajo configurado en GNOME/KDE para ejecutar `voxflow --toggle` sin escucha global de teclado.
2. Clipboard como vía universal de respaldo; indicar claramente al usuario cuándo debe pegar manualmente.
3. Inserción automática solo mediante mecanismo documentado, autorizado y validado para el compositor elegido.
4. Si se evalúa `ydotool`, informar requerimientos de `uinput`, privilegios y riesgos; nunca configurar acceso privilegiado silenciosamente.
5. En X11, evaluar un adaptador específico, sin asumir que funciona en Wayland.

**Seguridad del foco:** nunca introducir texto automáticamente si no se puede garantizar razonablemente que la ventana de destino es la esperada; solicitar confirmación o dejarlo copiado.

## 7. Privacidad y seguridad
- Audio procesado localmente y eliminado de memoria/temporales tras finalizar, salvo consentimiento explícito para guardarlo.
- Sin telemetría por defecto ni registro de transcripciones.
- Descarga de modelos exclusivamente desde fuentes configuradas/fiables, con checksum y progreso visibles.
- Evitar permisos root y accesos globales al teclado salvo necesidad justificada y consentimiento.
- No registrar tokens, variables secretas ni rutas sensibles.
- Mostrar indicador inequívoco mientras el micrófono está activo.
- Informar de las licencias de motor, modelos y dependencias antes de distribución comercial.

## 8. Experiencia de usuario

**Pantalla principal:** estado del micrófono, botón de grabar/detener, vista previa y acciones Copiar / Insertar / Descartar.  
**Bandeja:** estado, iniciar/detener, abrir aplicación, salir.  
**Preferencias:** micrófono, modelo, idioma, atajo, modo de salida, privacidad.  
**Primer uso:** comprobar audio, explicar Wayland, ofrecer descarga del modelo y transcripción de prueba.  
**Errores:** mensajes claros para micrófono no disponible, modelo ausente, descarga interrumpida, falta de espacio e inserción no compatible.

Diseño sobrio, accesible, tema claro/oscuro, teclado y estados perceptibles. Marca de trabajo: VoxFlow; validar disponibilidad de nombre/dominio antes de publicación comercial.

## 9. Requisitos no funcionales y medición

- Funcionamiento sin conexión después de instalar un modelo.
- La UI no debe bloquearse durante grabación, descarga o inferencia.
- Control de uso de CPU/memoria y cancelación efectiva.
- Todas las operaciones críticas reportan errores legibles.
- Registrar métricas *solo locales y opcionales* para benchmarking: duración del audio, latencia STT y tipo de dispositivo.
- Definir objetivos de velocidad después de medir hardware real; no inventar tiempos de transcripción.
- Pruebas con español peruano, tecnicismos (Laravel, React, Rust) y frases largas.

## 10. Criterios de aceptación del MVP

1. En Ubuntu 24.04 GNOME Wayland, se instala el `.deb`, inicia y reconoce el micrófono.
2. Sin Internet, transcribe un audio de prueba en español con modelo ya instalado.
3. Se puede activar desde el botón y desde un atajo de GNOME que invoque `voxflow --toggle`.
4. La transcripción se puede revisar, editar y copiar.
5. Si no hay inserción compatible, no se pierde texto; se ofrece pegar manualmente.
6. Durante grabación hay indicador visible, y detener/cancelar libera el dispositivo.
7. Una descarga fallida no deja un modelo inválido como instalado.
8. Sin activar historial, ningún texto dictado queda persistido ni aparece en logs.
9. El empaquetado incluye instrucciones para dependencias del sistema y desinstalación.
10. Existe un conjunto reproducible de pruebas y una lista explícita de limitaciones Wayland.

## 11. Plan de ejecución para el agente

### Fase 0 — Investigación técnica breve
- Verificar requisitos actuales de Tauri 2 en Ubuntu 24.04, versiones y licencias.
- Probar `whisper.cpp` en un ejecutable Rust mínimo con un archivo WAV.
- Probar captura del micrófono y salida de texto para GNOME Wayland y X11.
- Documentar decisiones y riesgos en `docs/DECISIONS.md`.

### Fase 1 — Esqueleto
- Crear proyecto Tauri 2 + React/Vite/TypeScript + Rust.
- UI principal, tray, preferencias, comandos y estado.
- CI con formatting, lint y pruebas mínimas.

### Fase 2 — Voz
- Implementar `AudioCapture` y `SpeechEngine`.
- Integrar modelo local y descarga verificada.
- Mostrar transcript editable, errores y progreso.

### Fase 3 — Integración Linux
- Implementar `--toggle` y guía para atajo de GNOME.
- Clipboard y adaptadores de inserción con fallback seguro.
- Validar comportamiento en Wayland y X11.

### Fase 4 — Calidad y entrega
- Pruebas de aceptación, seguridad y rendimiento.
- Crear `.deb`, `.AppImage`, changelog y release notes.
- Publicar release de prueba sin datos personales ni secretos.

**Regla para el agente:** completar cada fase con código compilable, pruebas, README actualizado y listado de limitaciones; no afirmar funcionalidades que no se hayan probado.

## 12. Estructura de repositorio sugerida

```text
voxflow/
├── AGENTS.md
├── README.md
├── LICENSE                  # elegir tras revisión de dependencias
├── docs/
│   ├── SPEC.md
│   ├── DECISIONS.md
│   └── TEST_PLAN.md
├── src/                     # React UI
├── src-tauri/
│   └── src/
│       ├── audio/
│       ├── speech/
│       ├── output/
│       ├── models/
│       ├── settings/
│       └── main.rs
├── tests/
└── .github/workflows/
```

## 13. Publicación

**Código:** GitHub, repositorio privado mientras se desarrolla, o Codeberg si se prefiere código abierto desde el principio.  
**Instaladores:** GitHub Releases; publicar `.deb` y `.AppImage` con versiones semánticas, checksums y notas de compatibilidad.  
**Sitio de producto (opcional):** landing propia cuando exista MVP validado. No necesita servidor Hetzner para el dictado local.  
**Licencia:** decidir antes de liberar públicamente, considerando licencias de dependencias, modelos y distribución de binarios.

## 14. Referencias técnicas
- Tauri v2 Linux: https://v2.tauri.app/distribute/debian/
- Tauri v2 AppImage: https://v2.tauri.app/distribute/appimage/
- Whisper.cpp: https://github.com/ggml-org/whisper.cpp
- Rust bindings: https://github.com/tazz4843/whisper-rs
- Restricciones Wayland: https://github.com/tauri-apps/tauri/issues/3578

---

**Instrucción principal:** construir un producto de dictado local confiable para Ubuntu, priorizando privacidad y la recuperación del texto por encima de automatismos frágiles de teclado.
