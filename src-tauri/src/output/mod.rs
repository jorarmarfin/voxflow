//! TextOutput: copiar al portapapeles y notificaciones, por plataforma.
//!
//! **Windows:** portapapeles nativo vía `arboard`; notificaciones con el plugin de Tauri.
//! **Linux:**
//! - Portapapeles por herramienta del sistema, con el texto por stdin (nunca por argv):
//!   Wayland → `wl-copy` (funciona en GNOME, que no soporta wlr-data-control);
//!   X11 → `xclip` o `xsel`.
//! - La inserción en la ventana activa vive en `insert.rs` (solo X11 + xdotool, con guarda).
//! - Las notificaciones (`notify-send`) nunca incluyen texto dictado.
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Session {
    Wayland,
    X11,
    #[cfg_attr(unix, allow(dead_code))]
    Windows,
    Unknown,
}

#[cfg_attr(windows, allow(dead_code))]
pub fn detect_session(
    session_type: Option<&str>,
    wayland_display: Option<&str>,
    display: Option<&str>,
) -> Session {
    match session_type {
        Some("wayland") => Session::Wayland,
        Some("x11") => Session::X11,
        _ if wayland_display.is_some_and(|v| !v.is_empty()) => Session::Wayland,
        _ if display.is_some_and(|v| !v.is_empty()) => Session::X11,
        _ => Session::Unknown,
    }
}

#[cfg(windows)]
pub fn session() -> Session {
    Session::Windows
}

#[cfg(unix)]
pub fn session() -> Session {
    detect_session(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
    )
}

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(crate) use unix::have;
#[cfg(unix)]
pub use unix::{clipboard_tool, copy, notifications_available, notify, read};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{clipboard_tool, copy, init, notifications_available, notify, read};

#[derive(Serialize)]
pub struct Diagnostics {
    pub session: Session,
    pub clipboard_tool: Option<&'static str>,
    pub notifications: bool,
    pub auto_insert: bool,
    pub insert_tool: Option<&'static str>,
}

pub fn diagnostics() -> Diagnostics {
    Diagnostics {
        session: session(),
        clipboard_tool: clipboard_tool(),
        notifications: notifications_available(),
        auto_insert: crate::insert::available(),
        insert_tool: crate::insert::tool(),
    }
}

/// ¿Alguno de los argumentos pide alternar la grabación?
pub fn wants_toggle(args: &[String]) -> bool {
    args.iter().any(|a| a == "--toggle")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_sesion() {
        assert_eq!(
            detect_session(Some("wayland"), None, None),
            Session::Wayland
        );
        assert_eq!(
            detect_session(Some("x11"), Some("wayland-0"), Some(":0")),
            Session::X11
        );
        assert_eq!(
            detect_session(None, Some("wayland-0"), None),
            Session::Wayland
        );
        assert_eq!(detect_session(Some("tty"), None, Some(":0")), Session::X11);
        assert_eq!(detect_session(None, Some(""), Some("")), Session::Unknown);
    }

    #[test]
    fn toggle_en_args() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(wants_toggle(&a(&["voxflow", "--toggle"])));
        assert!(!wants_toggle(&a(&["voxflow"])));
        assert!(!wants_toggle(&a(&["voxflow", "--toggled"])));
    }
}
