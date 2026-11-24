//! Inserción de texto en la ventana activa, por plataforma (X11 + xdotool / Windows + SendInput).
//! API común: `available()`, `tool()`, `capture_target()`, `paste(target)`.
//! Wayland no está soportado (no hay API general para escribir en otras ventanas).

#[cfg(unix)]
mod x11;
#[cfg(unix)]
pub use x11::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;
