//! PoC Fase 0: transcribe un WAV (16 kHz mono, 16-bit PCM) con whisper.cpp local.
//! Uso: whisper-poc <modelo.bin> <audio.wav> [idioma]
//! No registra el texto en logs; solo lo imprime en stdout.
use std::time::Instant;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: {} <modelo.bin> <audio.wav> [idioma=es]", args[0]);
        std::process::exit(2);
    }
    let lang = args.get(3).map(String::as_str).unwrap_or("es");

    let mut reader = hound::WavReader::open(&args[2])?;
    let spec = reader.spec();
    if spec.sample_rate != 16_000 || spec.channels != 1 || spec.bits_per_sample != 16 {
        return Err(format!("WAV debe ser 16 kHz mono 16-bit; recibido {spec:?}").into());
    }
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
        .collect::<Result<_, _>>()?;
    let audio_secs = samples.len() as f32 / 16_000.0;

    let t0 = Instant::now();
    let ctx = WhisperContext::new_with_params(&args[1], WhisperContextParameters::default())?;
    let load = t0.elapsed();

    let mut state = ctx.create_state()?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(lang));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);

    let t1 = Instant::now();
    state.full(params, &samples)?;
    let infer = t1.elapsed();

    let mut text = String::new();
    for seg in state.as_iter() {
        text.push_str(&seg.to_str_lossy()?);
    }
    println!("{}", text.trim());
    eprintln!(
        "[metrics] audio={audio_secs:.1}s carga_modelo={:.2}s inferencia={:.2}s rtf={:.2}",
        load.as_secs_f32(),
        infer.as_secs_f32(),
        infer.as_secs_f32() / audio_secs
    );
    Ok(())
}
