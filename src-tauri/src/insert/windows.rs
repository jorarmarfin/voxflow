//! Inserción en Windows con `SendInput` (Ctrl+V). Mismas garantías que en X11:
//! se guarda la ventana en primer plano al empezar a grabar y solo se pega si sigue siéndolo
//! al terminar y no es la propia VoxFlow. Si no, no se escribe nada (el texto queda copiado).
//! Limitación: Windows (UIPI) bloquea enviar teclas a ventanas con permisos de administrador.
use std::time::{Duration, Instant};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

const VK_V: u16 = 0x56;

pub fn available() -> bool {
    true
}

pub fn tool() -> Option<&'static str> {
    Some("SendInput")
}

fn foreground() -> Option<usize> {
    // SAFETY: llamada sin precondiciones; devuelve 0/NULL si no hay ventana.
    let h = unsafe { GetForegroundWindow() } as usize;
    (h != 0).then_some(h)
}

fn is_own_window(h: usize) -> bool {
    let mut pid: u32 = 0;
    // SAFETY: `pid` es un u32 válido para escritura durante la llamada.
    unsafe { GetWindowThreadProcessId(h as _, &mut pid) };
    pid == std::process::id()
}

/// Ventana en primer plano al empezar a grabar; `None` si es la propia VoxFlow o no hay.
pub fn capture_target() -> Option<String> {
    let h = foreground()?;
    (!is_own_window(h)).then(|| h.to_string())
}

/// Decisión pura (testeable): ¿es seguro pegar?
pub fn check(target: Option<&str>, current: Option<&str>) -> Result<(), &'static str> {
    match (target, current) {
        (None, _) => Err("no hay ventana de destino segura (inicia la grabación con el atajo)"),
        (_, None) => Err("no se pudo comprobar la ventana activa"),
        (Some(t), Some(c)) if t != c => Err("la ventana activa cambió durante la grabación"),
        _ => Ok(()),
    }
}

fn modifier_down(vk: u16) -> bool {
    // SAFETY: llamada sin precondiciones. El bit alto indica tecla pulsada.
    unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 }
}

fn key(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Pega con Ctrl+V si la guarda lo permite. El texto ya debe estar en el portapapeles.
pub fn paste(target: Option<&str>) -> Result<(), String> {
    let current = foreground();
    if let Some(c) = current {
        if is_own_window(c) {
            return Err("la ventana activa es VoxFlow".into());
        }
    }
    check(target, current.map(|c| c.to_string()).as_deref()).map_err(str::to_owned)?;

    // Esperar a que se suelten los modificadores del atajo (si no, Ctrl+Alt+Win+V ≠ pegar).
    let deadline = Instant::now() + Duration::from_millis(1500);
    while [VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
        .iter()
        .any(|&k| modifier_down(k))
    {
        if Instant::now() > deadline {
            return Err("sigues pulsando teclas modificadoras del atajo".into());
        }
        std::thread::sleep(Duration::from_millis(30));
    }

    let inputs = [
        key(VK_CONTROL, false),
        key(VK_V, false),
        key(VK_V, true),
        key(VK_CONTROL, true),
    ];
    // SAFETY: `inputs` es un arreglo válido de INPUT y el tamaño pasado es el de INPUT.
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    if sent as usize != inputs.len() {
        return Err(
            "Windows bloqueó el envío de teclas (¿ventana con permisos de administrador?)".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guarda_de_ventana() {
        assert!(check(Some("1"), Some("1")).is_ok());
        assert!(check(Some("1"), Some("2")).is_err());
        assert!(check(None, Some("1")).is_err());
        assert!(check(Some("1"), None).is_err());
    }
}
