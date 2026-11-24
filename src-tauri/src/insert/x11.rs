//! Inserción de texto en la ventana activa (solo X11, vía `xdotool`).
//!
//! Seguridad: se registra la ventana activa al iniciar la grabación (`capture_target`) y
//! solo se pega si, al terminar, sigue siendo la ventana activa y no es la propia VoxFlow.
//! En cualquier otro caso no se escribe nada: el texto queda copiado y se avisa.
//! Wayland no está soportado (no hay API general para escribir en otras ventanas).
use crate::output::{have, session, Session};
use std::process::{Command, Stdio};

fn xdo(args: &[&str]) -> Option<String> {
    let out = Command::new("xdotool")
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn tool() -> Option<&'static str> {
    available().then_some("xdotool")
}

/// ¿Se puede insertar en esta sesión? (X11, `xdotool` y `xprop` instalados)
pub fn available() -> bool {
    session() == Session::X11 && have("xdotool") && have("xprop")
}

/// Clase de la ventana (WM_CLASS) con `xprop`: `WM_CLASS(STRING) = "xterm", "XTerm"`.
fn window_class(id: &str) -> Option<String> {
    let out = Command::new("xprop")
        .args(["-id", id, "WM_CLASS"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let (_, classes) = text.split_once('=')?;
    let c = classes.trim().replace('"', "");
    (out.status.success() && !c.is_empty()).then_some(c)
}

fn valid_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 12 && s.bytes().all(|b| b.is_ascii_digit())
}

fn active_window() -> Option<String> {
    xdo(&["getactivewindow"]).filter(|id| valid_id(id))
}

fn is_own_window(id: &str) -> bool {
    xdo(&["getwindowpid", id]).and_then(|p| p.parse::<u32>().ok()) == Some(std::process::id())
}

/// Ventana activa al empezar a grabar. `None` si no se puede insertar con seguridad
/// (sin xdotool, sesión no X11, o la ventana activa es la propia VoxFlow).
pub fn capture_target() -> Option<String> {
    if !available() {
        return None;
    }
    let id = active_window()?;
    (!is_own_window(&id)).then_some(id)
}

const TERMINALS: &[&str] = &[
    "gnome-terminal",
    "ptyxis",
    "konsole",
    "xterm",
    "uxterm",
    "rxvt",
    "kitty",
    "alacritty",
    "terminator",
    "tilix",
    "wezterm",
    "xfce4-terminal",
    "guake",
    "tilda",
    "yakuake",
    "lxterminal",
    "mate-terminal",
    "foot",
];

pub fn is_terminal(class: &str) -> bool {
    let c = class.to_lowercase();
    TERMINALS.iter().any(|t| c.contains(t))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    Paste,
    PasteTerminal,
    Skip(&'static str),
}

/// Decisión pura (testeable): ¿es seguro pegar y con qué combinación?
pub fn plan(target: Option<&str>, current: Option<&str>, class: Option<&str>) -> Plan {
    match (target, current) {
        (None, _) => {
            Plan::Skip("no hay ventana de destino segura (inicia la grabación con el atajo)")
        }
        (_, None) => Plan::Skip("no se pudo comprobar la ventana activa"),
        (Some(t), Some(c)) if t != c => Plan::Skip("la ventana activa cambió durante la grabación"),
        (_, _) => match class {
            None => Plan::Skip("no se pudo determinar el tipo de ventana"),
            Some(c) if is_terminal(c) => Plan::PasteTerminal,
            Some(_) => Plan::Paste,
        },
    }
}

/// Pega (Ctrl+V / Ctrl+Shift+V en terminales) si la guarda lo permite.
/// El texto ya debe estar en el portapapeles. Devuelve el motivo si no se insertó.
pub fn paste(target: Option<&str>) -> Result<(), String> {
    if !available() {
        return Err("inserción no disponible (requiere X11 y xdotool)".into());
    }
    let current = active_window();
    let class = current.as_deref().and_then(window_class);
    let keys = match plan(target, current.as_deref(), class.as_deref()) {
        Plan::Paste => "ctrl+v",
        Plan::PasteTerminal => "ctrl+shift+v",
        Plan::Skip(why) => return Err(why.into()),
    };
    xdo(&["key", "--clearmodifiers", keys])
        .map(|_| ())
        .ok_or_else(|| "xdotool no pudo enviar las teclas".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_validos() {
        assert!(valid_id("65011715"));
        assert!(!valid_id(""));
        assert!(!valid_id("12 34"));
        assert!(!valid_id("--help"));
        assert!(!valid_id("1234567890123"));
    }

    #[test]
    fn detecta_terminales() {
        assert!(is_terminal("Gnome-terminal"));
        assert!(is_terminal("XTerm"));
        assert!(is_terminal("kitty"));
        assert!(!is_terminal("Google-chrome"));
        assert!(!is_terminal("code"));
    }

    #[test]
    fn guarda_de_ventana() {
        let code = Some("code, Code");
        assert_eq!(plan(Some("1"), Some("1"), code), Plan::Paste);
        assert_eq!(
            plan(Some("1"), Some("1"), Some("xterm, XTerm")),
            Plan::PasteTerminal
        );
        assert!(matches!(plan(Some("1"), Some("2"), code), Plan::Skip(_)));
        assert!(matches!(plan(None, Some("1"), code), Plan::Skip(_)));
        assert!(matches!(plan(Some("1"), None, code), Plan::Skip(_)));
        assert!(matches!(plan(Some("1"), Some("1"), None), Plan::Skip(_)));
    }
}
