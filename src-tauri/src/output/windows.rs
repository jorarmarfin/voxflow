//! Portapapeles y notificaciones en Windows.
//! Portapapeles nativo vía `arboard` (con reintentos: otro proceso puede tenerlo abierto);
//! notificaciones con el plugin de Tauri. Las notificaciones nunca incluyen texto dictado.
use std::sync::OnceLock;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

static APP: OnceLock<AppHandle> = OnceLock::new();

/// Guarda el handle para poder notificar desde cualquier hilo.
pub fn init(h: AppHandle) {
    let _ = APP.set(h);
}

pub fn clipboard_tool() -> Option<&'static str> {
    Some("portapapeles de Windows")
}

pub fn notifications_available() -> bool {
    true
}

pub fn copy(text: &str) -> Result<&'static str, String> {
    let mut last = String::new();
    for _ in 0..8 {
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(text.to_owned())) {
            Ok(()) => return Ok("windows"),
            Err(e) => last = e.to_string(),
        }
        std::thread::sleep(Duration::from_millis(60));
    }
    Err(format!("No se pudo copiar al portapapeles: {last}"))
}

/// Texto actual del portapapeles (para restaurarlo tras pegar). `None` si no es texto.
pub fn read() -> Option<String> {
    for _ in 0..4 {
        if let Ok(mut c) = arboard::Clipboard::new() {
            return c.get_text().ok().filter(|t| !t.is_empty());
        }
        std::thread::sleep(Duration::from_millis(60));
    }
    None
}

pub fn notify(body: &str) {
    if let Some(h) = APP.get() {
        let _ = h
            .notification()
            .builder()
            .title("VoxFlow")
            .body(body)
            .show();
    }
}
