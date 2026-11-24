//! Mezcla a mono y remuestreo lineal en streaming a 16 kHz (sin dependencias).
//! Suficiente para voz hacia STT; no pretende calidad de audio profesional.
use super::SAMPLE_RATE;

pub struct Resampler {
    channels: usize,
    step: f64,
    /// Posición de la próxima muestra de salida, en índices de entrada relativos al chunk actual.
    pos: f64,
    /// Última muestra mono del chunk anterior (índice -1 del chunk actual).
    prev: f32,
}

impl Resampler {
    pub fn new(in_rate: u32, channels: u16) -> Self {
        Self {
            channels: channels.max(1) as usize,
            step: in_rate as f64 / SAMPLE_RATE as f64,
            pos: 0.0,
            prev: 0.0,
        }
    }

    /// Procesa audio entrelazado f32 y añade muestras mono a 16 kHz en `out`.
    pub fn push(&mut self, interleaved: &[f32], out: &mut Vec<f32>) {
        let mono: Vec<f32> = interleaved
            .chunks_exact(self.channels)
            .map(|f| f.iter().sum::<f32>() / self.channels as f32)
            .collect();
        let len = mono.len() as isize;
        if len == 0 {
            return;
        }
        let at = |i: isize| if i < 0 { self.prev } else { mono[i as usize] };
        while (self.pos.floor() as isize) < len - 1 {
            let j = self.pos.floor() as isize;
            let frac = (self.pos - j as f64) as f32;
            out.push(at(j) * (1.0 - frac) + at(j + 1) * frac);
            self.pos += self.step;
        }
        self.pos -= len as f64;
        self.prev = mono[len as usize - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(rate: u32, ch: u16, input: &[f32], chunk: usize) -> Vec<f32> {
        let mut r = Resampler::new(rate, ch);
        let mut out = Vec::new();
        for c in input.chunks(chunk * ch as usize) {
            r.push(c, &mut out);
        }
        out
    }

    #[test]
    fn misma_frecuencia_conserva_las_muestras() {
        let input: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let out = run(16_000, 1, &input, 10);
        assert!(out.len() >= 98);
        for (a, b) in out.iter().zip(input.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn de_48k_a_16k_reduce_a_un_tercio() {
        let input = vec![0.5f32; 48_000];
        let out = run(48_000, 1, &input, 480);
        assert!((out.len() as i64 - 16_000).abs() <= 2, "len={}", out.len());
        assert!(out.iter().all(|v| (v - 0.5).abs() < 1e-6));
    }

    #[test]
    fn estereo_se_mezcla_a_mono() {
        let mut input = Vec::new();
        for _ in 0..4800 {
            input.extend([1.0f32, 0.0]);
        }
        let out = run(16_000, 2, &input, 100);
        assert!(out.iter().all(|v| (v - 0.5).abs() < 1e-6));
    }

    #[test]
    fn chunking_no_cambia_el_resultado() {
        let input: Vec<f32> = (0..44_100).map(|i| ((i as f32) * 0.01).sin()).collect();
        let a = run(44_100, 1, &input, 441);
        let b = run(44_100, 1, &input, 1000);
        let n = a.len().min(b.len());
        assert!(n > 15_000);
        for i in 0..n {
            assert!((a[i] - b[i]).abs() < 1e-5, "i={i}");
        }
    }
}
