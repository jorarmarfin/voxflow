//! AudioCapture: captura del micrófono vía `pw-record` (PipeWire) como proceso hijo,
//! con argumentos fijos (sin shell). El audio PCM 16 kHz mono s16 se guarda solo en memoria.
use serde::Serialize;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub const SAMPLE_RATE: usize = 16_000;
/// Tope de seguridad de memoria: 5 minutos de audio.
const MAX_SAMPLES: usize = SAMPLE_RATE * 300;

#[derive(Debug, Clone, Serialize)]
pub struct Device {
    pub id: String,
    pub label: String,
}

pub fn available() -> bool {
    Command::new("pw-record")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn valid_device_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && id.len() < 200
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-:".contains(c))
}

/// Parsea `pactl list short sources` (id \t nombre \t driver \t formato \t estado), sin monitores.
pub fn parse_sources(out: &str) -> Vec<Device> {
    out.lines()
        .filter_map(|l| l.split('\t').nth(1))
        .filter(|n| !n.ends_with(".monitor") && valid_device_id(n))
        .map(|n| Device {
            id: n.to_string(),
            label: n.trim_start_matches("alsa_input.").replace(['_', '.'], " "),
        })
        .collect()
}

pub fn list_devices() -> Result<Vec<Device>, String> {
    let out = Command::new("pactl")
        .args(["list", "short", "sources"])
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("No se pudo listar micrófonos (¿falta pactl?): {e}"))?;
    if !out.status.success() {
        return Err("pactl no pudo listar los micrófonos".into());
    }
    Ok(parse_sources(&String::from_utf8_lossy(&out.stdout)))
}

pub struct Recording {
    child: Child,
    buf: Arc<Mutex<Vec<i16>>>,
    reader: Option<JoinHandle<()>>,
}

pub fn start(device: Option<&str>) -> Result<Recording, String> {
    let mut cmd = Command::new("pw-record");
    cmd.args(["--rate", "16000", "--channels", "1", "--format", "s16"]);
    if let Some(d) = device {
        if !valid_device_id(d) {
            return Err("Identificador de micrófono no válido".into());
        }
        cmd.args(["--target", d]);
    }
    cmd.arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null());
    let mut child = cmd.spawn().map_err(|e| {
        format!("No se pudo iniciar la captura (¿falta pw-record / pipewire-bin?): {e}")
    })?;
    let mut out = child.stdout.take().ok_or("sin salida de audio")?;
    let buf = Arc::new(Mutex::new(Vec::<i16>::new()));
    let b2 = buf.clone();
    let reader = std::thread::spawn(move || {
        let mut chunk = [0u8; 8192];
        let mut carry: Option<u8> = None;
        while let Ok(n) = out.read(&mut chunk) {
            if n == 0 {
                break;
            }
            let mut bytes = Vec::with_capacity(n + 1);
            bytes.extend(carry.take());
            bytes.extend_from_slice(&chunk[..n]);
            if bytes.len() % 2 == 1 {
                carry = bytes.pop();
            }
            let mut v = b2.lock().unwrap();
            if v.len() < MAX_SAMPLES {
                v.extend(
                    bytes
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|b| i16::from_le_bytes(*b)),
                );
            }
        }
    });
    Ok(Recording {
        child,
        buf,
        reader: Some(reader),
    })
}

impl Recording {
    fn halt(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(r) = self.reader.take() {
            let _ = r.join();
        }
    }

    /// Detiene, libera el dispositivo y devuelve el audio en f32 [-1, 1].
    pub fn stop(mut self) -> Result<Vec<f32>, String> {
        self.halt();
        let samples = std::mem::take(&mut *self.buf.lock().unwrap());
        if samples.is_empty() {
            return Err(
                "No se recibió audio del micrófono (¿desconectado o sin permisos?).".into(),
            );
        }
        Ok(samples
            .into_iter()
            .map(|s| s as f32 / i16::MAX as f32)
            .collect())
    }

    /// Cancela y descarta el audio.
    pub fn cancel(mut self) {
        self.halt();
        self.buf.lock().unwrap().clear();
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        self.halt();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_fuentes_sin_monitores() {
        let out = "55\talsa_output.pci-0000_00_1b.0.analog-stereo.monitor\tPipeWire\ts32le 2ch 48000Hz\tRUNNING\n\
                   56\talsa_input.pci-0000_00_1b.0.analog-stereo\tPipeWire\ts32le 2ch 48000Hz\tSUSPENDED\n";
        let d = parse_sources(out);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].id, "alsa_input.pci-0000_00_1b.0.analog-stereo");
    }

    #[test]
    fn rechaza_ids_peligrosos() {
        assert!(!valid_device_id("--help"));
        assert!(!valid_device_id("a b"));
        assert!(!valid_device_id("a;rm -rf"));
        assert!(!valid_device_id(""));
        assert!(valid_device_id("alsa_input.usb-X_Y-00.mono-fallback"));
    }

    #[test]
    fn start_rechaza_id_invalido() {
        assert!(start(Some("--target")).is_err());
    }
}

#[cfg(test)]
mod hw_tests {
    use super::*;

    /// Requiere micrófono real: `cargo test -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn captura_real_de_2s() {
        let rec = start(None).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let s = rec.stop().unwrap();
        let rms = (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt();
        println!(
            "muestras={} (~{:.1}s) rms={rms:.4}",
            s.len(),
            s.len() as f32 / 16000.0
        );
        assert!(s.len() > 16_000);
    }
}
