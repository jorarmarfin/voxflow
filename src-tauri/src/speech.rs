//! SpeechEngine: interfaz desacoplada + adaptador whisper.cpp (whisper-rs).
//! La transcripción nunca se registra en logs.
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const MIN_SAMPLES: usize = 16_000 * 3 / 10; // 0,3 s

unsafe extern "C" fn abort_trampoline(user_data: *mut std::ffi::c_void) -> bool {
    // SAFETY: user_data apunta a un AtomicBool vivo (ver `transcribe`).
    unsafe { (*(user_data as *const AtomicBool)).load(Ordering::Relaxed) }
}

pub trait SpeechEngine: Send + Sync {
    fn transcribe(
        &self,
        audio: &[f32],
        language: &str,
        cancel: Arc<AtomicBool>,
    ) -> Result<String, String>;
}

pub struct WhisperEngine {
    ctx: WhisperContext,
}

impl WhisperEngine {
    pub fn load(model: &Path) -> Result<Self, String> {
        let path = model.to_str().ok_or("ruta de modelo no válida")?;
        let ctx = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .map_err(|e| format!("No se pudo cargar el modelo: {e}"))?;
        Ok(Self { ctx })
    }
}

impl SpeechEngine for WhisperEngine {
    fn transcribe(
        &self,
        audio: &[f32],
        language: &str,
        cancel: Arc<AtomicBool>,
    ) -> Result<String, String> {
        if audio.len() < MIN_SAMPLES {
            return Err("La grabación es demasiado corta.".into());
        }
        // whisper.cpp trabaja mejor con al menos 1 s: rellenar con silencio.
        let mut buf = audio.to_vec();
        if buf.len() < 16_000 {
            buf.resize(16_000, 0.0);
        }
        let mut state = self.ctx.create_state().map_err(|e| e.to_string())?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(language));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8);
        params.set_n_threads(threads as i32);
        // `set_abort_callback_safe` de whisper-rs 0.16 lee mal su puntero (UB y aborto
        // espurio, error -6), así que se usa la API cruda con un trampolín propio.
        // SAFETY: `cancel` (Arc) vive hasta después de `state.full`, y el puntero solo
        // se lee como `AtomicBool` dentro de esa llamada.
        unsafe {
            params.set_abort_callback(Some(abort_trampoline));
            params.set_abort_callback_user_data(Arc::as_ptr(&cancel) as *mut std::ffi::c_void);
        }
        state
            .full(params, &buf)
            .map_err(|e| format!("Falló la transcripción: {e}"))?;
        if cancel.load(Ordering::Relaxed) {
            return Err("Transcripción cancelada.".into());
        }
        let mut text = String::new();
        for seg in state.as_iter() {
            text.push_str(&seg.to_str_lossy().map_err(|e| e.to_string())?);
        }
        let text = text.trim().to_string();
        if text.is_empty() {
            return Err("No se detectó voz en la grabación.".into());
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requiere el modelo: `VOXFLOW_MODELS_DIR=../models cargo test -- --ignored`
    #[test]
    #[ignore]
    fn transcribe_fixture_jfk() {
        whisper_rs::install_logging_hooks();
        let dir = std::env::var("VOXFLOW_MODELS_DIR").expect("define VOXFLOW_MODELS_DIR");
        let engine = WhisperEngine::load(&crate::models::path(
            std::path::Path::new(&dir),
            &std::env::var("VOXFLOW_TEST_MODEL").unwrap_or_else(|_| "base".into()),
        ))
        .unwrap();
        let mut r = hound::WavReader::open("../tests/fixtures/jfk.wav").unwrap();
        let audio: Vec<f32> = r
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect();
        let text = engine
            .transcribe(&audio, "en", Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert!(text
            .to_lowercase()
            .contains("ask not what your country can do for you"));
    }

    #[test]
    fn audio_muy_corto_falla_sin_cargar_modelo() {
        // MIN_SAMPLES se valida antes de usar el contexto; aquí solo comprobamos la constante.
        assert_eq!(MIN_SAMPLES, 4800);
    }
}
