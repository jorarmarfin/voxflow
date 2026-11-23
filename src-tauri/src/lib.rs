mod settings;
mod state;

use settings::Settings;
use state::{Machine, RecState};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};

/// AudioCapture y SpeechEngine se implementan en la Fase 2.
/// Mientras tanto, iniciar una grabación devuelve un error explícito (sin simulaciones).
const ENGINE_AVAILABLE: bool = false;
const ENGINE_MSG: &str =
    "La captura de audio y la transcripción aún no están implementadas (Fase 2).";

struct App {
    machine: Mutex<Machine>,
    settings: Mutex<Settings>,
    settings_path: PathBuf,
}

#[tauri::command]
fn get_state(app: State<App>) -> RecState {
    app.machine.lock().unwrap().state()
}

#[tauri::command]
fn engine_available() -> bool {
    ENGINE_AVAILABLE
}

fn toggle_inner(app: &App) -> Result<RecState, String> {
    if !ENGINE_AVAILABLE {
        return Err(ENGINE_MSG.into());
    }
    let mut m = app.machine.lock().unwrap();
    match m.state() {
        RecState::Idle => m.start(),
        RecState::Recording => m.stop(),
        _ => return Err("Operación en curso; cancela primero.".into()),
    }
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_recording(app: State<App>, handle: AppHandle) -> Result<RecState, String> {
    let s = toggle_inner(&app)?;
    let _ = handle.emit("state-changed", s);
    Ok(s)
}

#[tauri::command]
fn cancel(app: State<App>, handle: AppHandle) -> RecState {
    let s = app.machine.lock().unwrap().cancel();
    let _ = handle.emit("state-changed", s);
    s
}

#[tauri::command]
fn get_settings(app: State<App>) -> Settings {
    app.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(app: State<App>, settings: Settings) -> Result<Settings, String> {
    settings.save(&app.settings_path)?;
    *app.settings.lock().unwrap() = settings.clone();
    Ok(settings)
}

fn show_main(handle: &AppHandle) {
    if let Some(w) = handle.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let settings_path = dir.join("settings.json");
            let settings = Settings::load(&settings_path);
            let start_minimized = settings.start_minimized;
            app.manage(App {
                machine: Mutex::new(Machine::default()),
                settings: Mutex::new(settings),
                settings_path,
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
            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().ok_or("sin icono")?)
                .tooltip("VoxFlow")
                .menu(&menu)
                .on_menu_event(|h, ev| match ev.id.as_ref() {
                    "open" => show_main(h),
                    "toggle" => {
                        let app = h.state::<App>();
                        match toggle_inner(&app) {
                            Ok(s) => {
                                let _ = h.emit("state-changed", s);
                            }
                            Err(msg) => {
                                show_main(h);
                                let _ = h.emit("notice", msg);
                            }
                        }
                    }
                    "quit" => h.exit(0),
                    _ => {}
                })
                .build(app)?;

            if start_minimized {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            engine_available,
            toggle_recording,
            cancel,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar VoxFlow");
}
