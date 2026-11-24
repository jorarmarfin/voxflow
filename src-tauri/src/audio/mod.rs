//! AudioCapture por plataforma. API común: `available()`, `list_devices()`,
//! `start(device) -> Recording`, `Recording::{stop, cancel}`.
//!
//! - Linux: `pw-record` (PipeWire) como proceso hijo.
//! - Windows: WASAPI vía `cpal`, con remuestreo a 16 kHz mono.
//!
//! El audio solo vive en memoria.
use serde::Serialize;

#[cfg_attr(unix, allow(dead_code))]
pub mod resample;

/// Frecuencia que espera whisper.cpp.
pub const SAMPLE_RATE: usize = 16_000;
/// Tope de seguridad de memoria: 5 minutos de audio.
pub const MAX_SAMPLES: usize = SAMPLE_RATE * 300;

#[derive(Debug, Clone, Serialize)]
pub struct Device {
    pub id: String,
    pub label: String,
}

#[cfg(unix)]
mod pipewire;
#[cfg(unix)]
pub use pipewire::*;

#[cfg(windows)]
mod wasapi;
#[cfg(windows)]
pub use wasapi::*;
