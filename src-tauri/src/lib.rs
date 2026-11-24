mod audio;
mod diag_log;
mod insert;
mod models;
mod output;
mod settings;
mod speech;
mod state;

use serde::Serialize;
use settings::Settings;
use speech::{SpeechEngine, WhisperEngine};
use state::{Machine, RecState};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};

struct App {
    machine: Mutex<Machine>,
    settings: Mutex<Settings>,
    settings_path: PathBuf,
    models_dir: PathBuf,
    recording: Mutex<Option<audio::Recording>>,
    engine: Mutex<Option<(String, Arc<WhisperEngine>)>>,
    /// Bandera de cancelación de la transcripción en curso.
    cancel: Mutex<Arc<AtomicBool>>,
    /// Se incrementa al cancelar: descarta resultados de transcripciones obsoletas.
    generation: AtomicU64,
    /// Ventana activa al empezar a grabar (modo Insert, X11); guarda de seguridad.
    target: Mutex<Option<String>>,
    /// La operación en curso se inició desde el atajo/bandeja (ventana posiblemente oculta).
    background: AtomicBool,
    downloading: AtomicBool,
    dl_cancel: Arc<AtomicBool>,
}

#[derive(Serialize)]
struct Status {
    audio_ok: bool,
    model_installed: bool,
}

#[derive(Serialize, Clone)]
struct Progress {
    downloaded: u64,
    total: u64,
}

#[derive(Serialize, Clone)]
struct DownloadDone {
    ok: bool,
    error: Option<String>,
}

fn set_state(h: &AppHandle, s: RecState) {
    let _ = h.emit("state-changed", s);
    if let Some(t) = h.tray_by_id("main") {
        let tip = match s {
            RecState::Recording => "VoxFlow — GRABANDO (micrófono activo)",
            RecState::Transcribing => "VoxFlow — transcribiendo…",
            _ => "VoxFlow",
        };
        let _ = t.set_tooltip(Some(tip));
    }
}

fn notice(h: &AppHandle, msg: impl Into<String>) {
    let _ = h.emit("notice", msg.into());
}

fn model_installed(app: &App) -> bool {
    let id = app.settings.lock().unwrap().model.clone();
    models::path(&app.models_dir, &id).is_file()
}

#[tauri::command]
fn get_state(app: State<App>) -> RecState {
    app.machine.lock().unwrap().state()
}

#[tauri::command]
fn get_status(app: State<App>) -> Status {
    Status {
        audio_ok: audio::available(),
        model_installed: model_installed(&app),
    }
}

#[tauri::command]
async fn list_devices() -> Result<Vec<audio::Device>, String> {
    audio::list_devices()
}

fn start_recording(h: &AppHandle, app: &App, background: bool) -> Result<RecState, String> {
    if !model_installed(app) {
        return Err("Falta el modelo de voz. Descárgalo en Preferencias.".into());
    }
    let (device, mode) = {
        let st = app.settings.lock().unwrap();
        (st.device_id.clone(), st.output_mode.clone())
    };
    let mut m = app.machine.lock().unwrap();
    if !matches!(m.state(), RecState::Idle | RecState::Ready) {
        return Err("Operación en curso; cancela primero.".into());
    }
    let rec = audio::start(device.as_deref())?;
    *app.recording.lock().unwrap() = Some(rec);
    let s = m.start().map_err(|e| e.to_string())?;
    drop(m);
    app.background.store(background, Ordering::SeqCst);
    diag_log::log("grabación iniciada");
    *app.target.lock().unwrap() = if mode == settings::OutputMode::Insert {
        insert::capture_target()
    } else {
        None
    };
    set_state(h, s);
    if background {
        output::notify("Grabando… ejecuta el atajo de nuevo para detener.");
    }
    Ok(s)
}

fn stop_and_transcribe(h: &AppHandle, app: &App) -> Result<RecState, String> {
    let rec = app.recording.lock().unwrap().take();
    let samples = {
        let mut m = app.machine.lock().unwrap();
        let s = m.stop().map_err(|e| e.to_string());
        match (s, rec) {
            (Ok(_), Some(r)) => match r.stop() {
                Ok(v) => v,
                Err(e) => {
                    let st = m.cancel();
                    drop(m);
                    set_state(h, st);
                    return Err(e);
                }
            },
            (Err(e), _) => return Err(e),
            (Ok(_), None) => {
                m.cancel();
                return Err("No había grabación activa.".into());
            }
        }
    };
    diag_log::log(&format!(
        "grabación detenida: {} muestras (~{:.1} s)",
        samples.len(),
        samples.len() as f32 / 16_000.0
    ));
    set_state(h, RecState::Transcribing);
    if app.background.load(Ordering::SeqCst) {
        output::notify("Transcribiendo…");
    }

    let flag = Arc::new(AtomicBool::new(false));
    *app.cancel.lock().unwrap() = flag.clone();
    let gen = app.generation.load(Ordering::SeqCst);
    let h2 = h.clone();
    // Pila grande: la inferencia de ggml puede usar mucha pila (hilo por defecto en Windows: 2 MB).
    let spawned = std::thread::Builder::new()
        .name("transcribe".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let app = h2.state::<App>();
            let (model, language) = {
                let s = app.settings.lock().unwrap();
                (s.model.clone(), s.language.clone())
            };
            let result = (|| {
                let engine = {
                    let mut slot = app.engine.lock().unwrap();
                    match slot.as_ref() {
                        Some((id, e)) if *id == model => e.clone(),
                        _ => {
                            diag_log::log("motor: cargando modelo");
                            let e = Arc::new(WhisperEngine::load(&models::path(
                                &app.models_dir,
                                &model,
                            ))?);
                            diag_log::log("motor: modelo cargado");
                            *slot = Some((model.clone(), e.clone()));
                            e
                        }
                    }
                };
                diag_log::log("transcripción: inicio");
                engine.transcribe(&samples, &language, flag)
            })();
            match &result {
                Ok(t) => diag_log::log(&format!(
                    "transcripción: fin ({} caracteres)",
                    t.chars().count()
                )),
                Err(e) => diag_log::log(&format!("transcripción: error: {e}")),
            }
            if app.generation.load(Ordering::SeqCst) != gen {
                return; // cancelada: el estado ya volvió a IDLE
            }
            let background = app.background.load(Ordering::SeqCst);
            let target = app.target.lock().unwrap().take();
            let mode = app.settings.lock().unwrap().output_mode.clone();
            let mut m = app.machine.lock().unwrap();
            match result {
                Ok(text) => {
                    if m.finish().is_ok() {
                        drop(m);
                        let _ = h2.emit("transcript", &text);
                        set_state(&h2, RecState::Ready);
                        deliver(&h2, &text, mode, background, target);
                    }
                }
                Err(e) => {
                    let st = m.cancel();
                    drop(m);
                    set_state(&h2, st);
                    fail(&h2, e, background);
                }
            }
        });
    if let Err(e) = spawned {
        let st = app.machine.lock().unwrap().cancel();
        set_state(h, st);
        return Err(format!("No se pudo iniciar la transcripción: {e}"));
    }
    Ok(RecState::Transcribing)
}

/// Entrega el texto según el modo de salida. Las notificaciones no incluyen el texto.
fn deliver(
    h: &AppHandle,
    text: &str,
    mode: settings::OutputMode,
    background: bool,
    target: Option<String>,
) {
    match mode {
        settings::OutputMode::Insert => insert_flow(h, text, target),
        settings::OutputMode::Clipboard => match output::copy(text) {
            Ok(_) => {
                notice(h, "Texto copiado al portapapeles. Pega con Ctrl+V.");
                output::notify("Texto copiado. Pega con Ctrl+V.");
            }
            Err(e) => {
                show_main(h);
                notice(h, e);
            }
        },
        settings::OutputMode::Preview => {
            if background {
                show_main(h);
                output::notify("Texto listo en VoxFlow: revísalo y copia.");
            }
        }
    }
}

/// Modo Insert: copia, pega solo si la guarda de ventana lo permite y restaura el
/// portapapeles anterior. Si no se inserta, el texto queda copiado y se avisa.
fn insert_flow(h: &AppHandle, text: &str, target: Option<String>) {
    let previous = output::read();
    if let Err(e) = output::copy(text) {
        show_main(h);
        notice(h, e);
        return;
    }
    match insert::paste(target.as_deref()) {
        Ok(()) => {
            output::notify("Texto insertado.");
            notice(h, "Texto insertado en la ventana activa.");
            // Dar tiempo a la aplicación para leer el portapapeles antes de restaurarlo.
            std::thread::sleep(std::time::Duration::from_millis(400));
            if let Some(p) = previous {
                let _ = output::copy(&p);
            }
        }
        Err(why) => {
            let msg = format!("No se insertó: {why}. El texto está copiado: pégalo con Ctrl+V.");
            output::notify(&msg);
            notice(h, msg);
        }
    }
}

/// Error recuperable: aviso en la UI y, si no hay ventana a la vista, notificación.
fn fail(h: &AppHandle, msg: String, background: bool) {
    diag_log::log(&format!("aviso: {msg}"));
    if background {
        output::notify(&msg);
    }
    notice(h, msg);
}

fn toggle_inner(h: &AppHandle, app: &App, background: bool) -> Result<RecState, String> {
    let state = app.machine.lock().unwrap().state();
    match state {
        RecState::Recording => stop_and_transcribe(h, app),
        RecState::Transcribing => Err("Transcribiendo; espera o cancela.".into()),
        _ => start_recording(h, app, background),
    }
}

#[tauri::command]
fn toggle_recording(app: State<App>, h: AppHandle) -> Result<RecState, String> {
    toggle_inner(&h, &app, false)
}

fn do_cancel(h: &AppHandle, app: &App) -> RecState {
    diag_log::log("cancelado");
    app.generation.fetch_add(1, Ordering::SeqCst);
    *app.target.lock().unwrap() = None;
    app.cancel.lock().unwrap().store(true, Ordering::SeqCst);
    if let Some(r) = app.recording.lock().unwrap().take() {
        r.cancel();
    }
    let s = app.machine.lock().unwrap().cancel();
    set_state(h, s);
    s
}

#[tauri::command]
fn cancel(app: State<App>, h: AppHandle) -> RecState {
    do_cancel(&h, &app)
}

/// Texto entregado o descartado: vuelve a IDLE.
#[tauri::command]
fn dismiss(app: State<App>, h: AppHandle) -> RecState {
    let s = {
        let mut m = app.machine.lock().unwrap();
        let _ = m.dismiss();
        m.state()
    };
    set_state(&h, s);
    s
}

#[tauri::command]
fn get_settings(app: State<App>) -> Settings {
    app.settings.lock().unwrap().clone()
}

/// Registra el atajo global (solo Windows). En Linux el atajo lo gestiona el escritorio.
#[cfg(windows)]
fn apply_hotkey(h: &AppHandle, accel: &str) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
    let gs = h.global_shortcut();
    let _ = gs.unregister_all();
    gs.on_shortcut(accel, |app, _shortcut, ev| {
        if ev.state() == ShortcutState::Pressed {
            background_toggle(app);
        }
    })
    .map_err(|e| format!("Atajo no válido o en uso por otra aplicación ({accel}): {e}"))
}

#[tauri::command]
fn save_settings(app: State<App>, h: AppHandle, settings: Settings) -> Result<Settings, String> {
    settings.validate()?;
    #[cfg(windows)]
    if settings.hotkey != app.settings.lock().unwrap().hotkey {
        apply_hotkey(&h, &settings.hotkey)?;
    }
    #[cfg(not(windows))]
    let _ = &h;
    settings.save(&app.settings_path)?;
    *app.settings.lock().unwrap() = settings.clone();
    Ok(settings)
}

#[tauri::command]
fn copy_text(text: String) -> Result<(), String> {
    output::copy(&text).map(|_| ())
}

/// Botón "Insertar" de la ventana: el foco está en VoxFlow, así que no hay destino seguro;
/// se copia el texto (la inserción directa se hace con el modo "Insertar" y el atajo).
#[tauri::command]
fn insert_text(text: String) -> Result<(), String> {
    output::copy(&text).map(|_| ())
}

#[tauri::command]
fn get_diagnostics() -> output::Diagnostics {
    output::diagnostics()
}

#[tauri::command]
fn model_status(app: State<App>) -> Vec<models::ModelStatus> {
    models::status(&app.models_dir)
}

#[tauri::command]
fn download_model(app: State<App>, h: AppHandle, id: String) -> Result<(), String> {
    models::find(&id).ok_or("modelo desconocido")?;
    if app.downloading.swap(true, Ordering::SeqCst) {
        return Err("Ya hay una descarga en curso.".into());
    }
    app.dl_cancel.store(false, Ordering::SeqCst);
    std::thread::spawn(move || {
        let app = h.state::<App>();
        let h2 = h.clone();
        let r = models::download(&app.models_dir, &id, &app.dl_cancel, |d, t| {
            let _ = h2.emit(
                "download-progress",
                Progress {
                    downloaded: d,
                    total: t,
                },
            );
        });
        app.downloading.store(false, Ordering::SeqCst);
        let _ = h.emit(
            "download-done",
            DownloadDone {
                ok: r.is_ok(),
                error: r.err(),
            },
        );
    });
    Ok(())
}

#[tauri::command]
fn cancel_download(app: State<App>) {
    app.dl_cancel.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn delete_model(app: State<App>, id: String) -> Result<(), String> {
    if app.machine.lock().unwrap().state() != RecState::Idle {
        return Err("No se puede eliminar el modelo durante una operación.".into());
    }
    *app.engine.lock().unwrap() = None;
    models::delete(&app.models_dir, &id)
}

/// Alterna la grabación desde el atajo (`voxflow --toggle`) o la bandeja, sin robar el foco.
fn background_toggle(h: &AppHandle) {
    let app = h.state::<App>();
    if let Err(msg) = toggle_inner(h, &app, true) {
        output::notify(&msg);
        notice(h, msg);
    }
}

fn show_main(h: &AppHandle) {
    if let Some(w) = h.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn run() {
    diag_log::init();
    diag_log::log_startup();
    // whisper.cpp imprime tokens/segmentos en stderr: se redirige al crate `log`,
    // que no tiene logger instalado, así que nada de la transcripción se registra.
    whisper_rs::install_logging_hooks();
    let builder = tauri::Builder::default()
        // Debe ser el primer plugin: una 2.ª ejecución (`voxflow --toggle`) reenvía sus
        // argumentos a la instancia en marcha y termina.
        .plugin(tauri_plugin_single_instance::init(|h, args, _cwd| {
            if output::wants_toggle(&args) {
                background_toggle(h);
            } else {
                show_main(h);
            }
        }));
    #[cfg(windows)]
    let builder = builder
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init());
    builder
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let settings_path = dir.join("settings.json");
            let settings = Settings::load(&settings_path);
            let start_minimized = settings.start_minimized;
            #[cfg(windows)]
            {
                output::init(app.handle().clone());
                if let Err(e) = apply_hotkey(app.handle(), &settings.hotkey) {
                    output::notify(&e);
                }
            }
            let models_dir = std::env::var_os("VOXFLOW_MODELS_DIR")
                .map(PathBuf::from)
                .unwrap_or(app.path().app_data_dir()?.join("models"));
            app.manage(App {
                machine: Mutex::new(Machine::default()),
                settings: Mutex::new(settings),
                settings_path,
                models_dir,
                recording: Mutex::new(None),
                engine: Mutex::new(None),
                cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
                generation: AtomicU64::new(0),
                target: Mutex::new(None),
                background: AtomicBool::new(false),
                downloading: AtomicBool::new(false),
                dl_cancel: Arc::new(AtomicBool::new(false)),
            });

            let open = MenuItem::with_id(app, "open", "Abrir VoxFlow", true, None::<&str>)?;
            let toggle = MenuItem::with_id(
                app,
                "toggle",
                "Iniciar/Detener grabación",
                true,
                None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &toggle, &quit])?;
            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().cloned().ok_or("sin icono")?)
                .tooltip("VoxFlow")
                .menu(&menu)
                .on_menu_event(|h, ev| match ev.id.as_ref() {
                    "open" => show_main(h),
                    "toggle" => {
                        background_toggle(h);
                    }
                    "quit" => {
                        let app = h.state::<App>();
                        do_cancel(h, &app);
                        h.exit(0)
                    }
                    _ => {}
                })
                .build(app)?;

            let cli_toggle = output::wants_toggle(&std::env::args().collect::<Vec<_>>());
            if !start_minimized && !cli_toggle {
                show_main(app.handle());
            }
            if cli_toggle {
                // Arranque en frío con --toggle: iniciar a dictar sin mostrar la ventana.
                background_toggle(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_status,
            list_devices,
            toggle_recording,
            cancel,
            dismiss,
            get_settings,
            save_settings,
            model_status,
            download_model,
            cancel_download,
            delete_model,
            copy_text,
            insert_text,
            get_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar VoxFlow");
}
