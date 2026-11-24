//! AudioCapture en Windows (WASAPI vía `cpal`). El stream vive en un hilo propio
//! (los streams de cpal no son `Send` en todas las plataformas) y se detiene con una bandera.
use super::resample::Resampler;
use super::{Device, MAX_SAMPLES};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

pub fn available() -> bool {
    cpal::default_host().default_input_device().is_some()
}

pub fn list_devices() -> Result<Vec<Device>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("No se pudieron listar los micrófonos: {e}"))?;
    Ok(devices
        .filter_map(|d| {
            let id = d.id().ok()?.to_string();
            let label = d
                .description()
                .map(|x| x.name().to_string())
                .unwrap_or_else(|_| id.clone());
            Some(Device { id, label })
        })
        .collect())
}

pub struct Recording {
    stop: Arc<AtomicBool>,
    buf: Arc<Mutex<Vec<f32>>>,
    worker: Option<JoinHandle<()>>,
}

fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    buf: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let mut rs = Resampler::new(config.sample_rate, config.channels);
    let mut scratch: Vec<f32> = Vec::new();
    device
        .build_input_stream::<T, _, _>(
            config,
            move |data: &[T], _| {
                scratch.clear();
                scratch.extend(data.iter().map(|s| f32::from_sample(*s)));
                let mut out = buf.lock().unwrap();
                if out.len() < MAX_SAMPLES {
                    rs.push(&scratch, &mut out);
                }
            },
            |_e| {},
            None,
        )
        .map_err(|e| format!("No se pudo abrir el micrófono: {e}"))
}

pub fn start(device: Option<&str>) -> Result<Recording, String> {
    let device_id = device.map(str::to_owned);
    let stop = Arc::new(AtomicBool::new(false));
    let buf = Arc::new(Mutex::new(Vec::<f32>::new()));
    let (tx, rx) = mpsc::channel::<Result<(), String>>();
    let (stop2, buf2) = (stop.clone(), buf.clone());

    let worker = std::thread::spawn(move || {
        let setup = (|| -> Result<cpal::Stream, String> {
            let host = cpal::default_host();
            let dev = match device_id.as_deref() {
                Some(id) => {
                    let did = cpal::DeviceId::from_str(id)
                        .map_err(|_| "Identificador de micrófono no válido")?;
                    host.device_by_id(&did)
                        .ok_or("El micrófono elegido no está disponible")?
                }
                None => host.default_input_device().ok_or(
                    "No hay micrófono disponible (revisa Configuración → Privacidad → Micrófono)",
                )?,
            };
            let cfg = dev
                .default_input_config()
                .map_err(|e| format!("No se pudo leer la configuración del micrófono: {e}"))?;
            let fmt = cfg.sample_format();
            crate::diag_log::log(&format!(
                "audio: {} Hz, {} canales, formato {fmt}",
                cfg.sample_rate(),
                cfg.channels()
            ));
            let config = cfg.config();
            let stream = match fmt {
                SampleFormat::F32 => build::<f32>(&dev, config, buf2.clone())?,
                SampleFormat::I16 => build::<i16>(&dev, config, buf2.clone())?,
                SampleFormat::I32 => build::<i32>(&dev, config, buf2.clone())?,
                SampleFormat::U16 => build::<u16>(&dev, config, buf2.clone())?,
                other => return Err(format!("Formato de audio no soportado: {other}")),
            };
            stream
                .play()
                .map_err(|e| format!("No se pudo iniciar la captura: {e}"))?;
            Ok(stream)
        })();
        match setup {
            Ok(stream) => {
                let _ = tx.send(Ok(()));
                while !stop2.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(30));
                }
                drop(stream); // libera el dispositivo
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });

    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(Recording {
            stop,
            buf,
            worker: Some(worker),
        }),
        Ok(Err(e)) => {
            let _ = worker.join();
            Err(e)
        }
        Err(_) => {
            stop.store(true, Ordering::SeqCst);
            Err("El micrófono no respondió.".into())
        }
    }
}

impl Recording {
    fn halt(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }

    /// Detiene, libera el dispositivo y devuelve el audio mono f32 a 16 kHz.
    pub fn stop(mut self) -> Result<Vec<f32>, String> {
        self.halt();
        let samples = std::mem::take(&mut *self.buf.lock().unwrap());
        if samples.is_empty() {
            return Err(
                "No se recibió audio del micrófono (¿desconectado o sin permiso de micrófono?)."
                    .into(),
            );
        }
        Ok(samples)
    }

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
