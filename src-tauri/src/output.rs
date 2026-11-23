//! TextOutput: copiar al portapapeles, notificaciones y (no) inserción.
//!
//! - Portapapeles por herramienta del sistema, con el texto por stdin (nunca por argv):
//!   Wayland → `wl-copy` (funciona en GNOME, que no soporta wlr-data-control);
//!   X11 → `xclip` o `xsel`.
//! - La inserción en la ventana activa vive en `insert.rs` (solo X11 + xdotool, con guarda).
//! - Las notificaciones (`notify-send`) nunca incluyen texto dictado.
use serde::Serialize;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Session {
    Wayland,
    X11,
    Unknown,
}

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

pub fn session() -> Session {
    detect_session(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
    )
}

pub(crate) fn have(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(cmd).is_file()))
        .unwrap_or(false)
}

/// Herramientas de portapapeles candidatas, en orden de preferencia, para la sesión.
pub fn clipboard_candidates(s: Session) -> Vec<(&'static str, &'static [&'static str])> {
    let wl: (&str, &[&str]) = ("wl-copy", &[]);
    let xclip: (&str, &[&str]) = ("xclip", &["-selection", "clipboard"]);
    let xsel: (&str, &[&str]) = ("xsel", &["--clipboard", "--input"]);
    match s {
        Session::Wayland => vec![wl, xclip, xsel],
        _ => vec![xclip, xsel, wl],
    }
}

pub fn clipboard_tool() -> Option<&'static str> {
    clipboard_candidates(session())
        .into_iter()
        .map(|c| c.0)
        .find(|c| have(c))
}

pub fn copy(text: &str) -> Result<&'static str, String> {
    for (cmd, args) in clipboard_candidates(session()) {
        if !have(cmd) {
            continue;
        }
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("No se pudo ejecutar {cmd}: {e}"))?;
        child
            .stdin
            .take()
            .ok_or("sin stdin")?
            .write_all(text.as_bytes())
            .map_err(|e| format!("No se pudo escribir en {cmd}: {e}"))?;
        // stdin se cierra al soltarse; xclip/wl-copy se quedan sirviendo el portapapeles en segundo plano.
        let st = child.wait().map_err(|e| e.to_string())?;
        if st.success() {
            return Ok(cmd);
        }
    }
    Err("No hay portapapeles disponible: instala wl-clipboard (Wayland) o xclip (X11).".into())
}

/// Lee el texto actual del portapapeles (para restaurarlo tras pegar). `None` si no hay
/// texto o no se puede leer. Nunca se registra.
pub fn read() -> Option<String> {
    let (cmd, args): (&str, &[&str]) = match session() {
        Session::Wayland if have("wl-paste") => ("wl-paste", &["--no-newline"]),
        _ if have("xclip") => ("xclip", &["-selection", "clipboard", "-o"]),
        _ if have("xsel") => ("xsel", &["--clipboard", "--output"]),
        _ => return None,
    };
    let out = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    (out.status.success() && !out.stdout.is_empty())
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn notify(body: &str) {
    if !have("notify-send") {
        return;
    }
    let _ = Command::new("notify-send")
        .args([
            "-a",
            "VoxFlow",
            "-h",
            "string:x-canonical-private-synchronous:voxflow",
            "VoxFlow",
            body,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

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
        notifications: have("notify-send"),
        auto_insert: crate::insert::available(),
        insert_tool: crate::insert::available().then_some("xdotool"),
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
    fn orden_de_herramientas() {
        assert_eq!(clipboard_candidates(Session::Wayland)[0].0, "wl-copy");
        assert_eq!(clipboard_candidates(Session::X11)[0].0, "xclip");
    }

    #[test]
    fn toggle_en_args() {
        let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(wants_toggle(&a(&["voxflow", "--toggle"])));
        assert!(!wants_toggle(&a(&["voxflow"])));
        assert!(!wants_toggle(&a(&["voxflow", "--toggled"])));
    }

    /// Requiere sesión gráfica y xclip/wl-copy: `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn copia_y_lee_de_vuelta_x11() {
        let marca = format!("voxflow-test-{}", std::process::id());
        copy(&marca).unwrap();
        let out = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), marca);
    }
}
