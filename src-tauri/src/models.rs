//! ModelManager: estado, descarga verificada (SHA-256), cancelación y borrado.
//! Un modelo solo queda "instalado" tras verificar el checksum (descarga a .part + rename).
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ModelDef {
    pub id: &'static str,
    /// Texto para la UI: nombre y calidad aproximada.
    pub label: &'static str,
    pub sha256: &'static str,
    /// Tamaño de descarga esperado (bytes), solo informativo.
    pub size_bytes: u64,
}

impl ModelDef {
    pub fn url(&self) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            self.id
        )
    }
}

/// Modelos de whisper.cpp (`ggerganov/whisper.cpp`). Los checksums son los SHA-256 publicados
/// por Hugging Face (LFS). Más grande = más preciso y más lento; `q5` = cuantizado (más liviano).
pub const MODELS: &[ModelDef] = &[
    ModelDef {
        id: "base",
        label: "base · rápido, precisión básica",
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        size_bytes: 147_951_465,
    },
    ModelDef {
        id: "small",
        label: "small · equilibrado",
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        size_bytes: 487_601_967,
    },
    ModelDef {
        id: "medium",
        label: "medium · buena precisión",
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        size_bytes: 1_533_763_059,
    },
    ModelDef {
        id: "large-v3-turbo-q5_0",
        label: "large-v3-turbo (q5) · muy buena precisión, recomendado",
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        size_bytes: 574_041_195,
    },
    ModelDef {
        id: "large-v3-turbo",
        label: "large-v3-turbo · muy buena precisión",
        sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
        size_bytes: 1_624_555_275,
    },
    ModelDef {
        id: "large-v3-q5_0",
        label: "large-v3 (q5) · máxima precisión, cuantizado",
        sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1",
        size_bytes: 1_081_140_203,
    },
    ModelDef {
        id: "large-v3",
        label: "large-v3 · máxima precisión (usa ~4-5 GB de RAM)",
        sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
        size_bytes: 3_095_033_483,
    },
];

#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub id: String,
    pub label: String,
    pub installed: bool,
    /// Tamaño en disco si está instalado.
    pub size_bytes: u64,
    /// Tamaño de descarga esperado.
    pub expected_bytes: u64,
}

pub fn find(id: &str) -> Option<&'static ModelDef> {
    MODELS.iter().find(|m| m.id == id)
}

pub fn path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("ggml-{id}.bin"))
}

pub fn status(dir: &Path) -> Vec<ModelStatus> {
    MODELS
        .iter()
        .map(|m| {
            let meta = std::fs::metadata(path(dir, m.id)).ok();
            ModelStatus {
                id: m.id.into(),
                label: m.label.into(),
                installed: meta.is_some(),
                size_bytes: meta.map(|x| x.len()).unwrap_or(0),
                expected_bytes: m.size_bytes,
            }
        })
        .collect()
}

pub fn delete(dir: &Path, id: &str) -> Result<(), String> {
    find(id).ok_or("modelo desconocido")?;
    match std::fs::remove_file(path(dir, id)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("No se pudo eliminar el modelo: {e}")),
    }
}

pub fn download(
    dir: &Path,
    id: &str,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<(), String> {
    let def = find(id).ok_or("modelo desconocido")?;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("No se pudo crear la carpeta de modelos: {e}"))?;
    let dest = path(dir, id);
    let tmp = dest.with_extension("bin.part");
    let result = fetch(def, &tmp, cancel, &mut on_progress);
    match result {
        Ok(()) => {
            std::fs::rename(&tmp, &dest).map_err(|e| format!("No se pudo instalar el modelo: {e}"))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

fn fetch(
    def: &ModelDef,
    tmp: &Path,
    cancel: &AtomicBool,
    on_progress: &mut impl FnMut(u64, u64),
) -> Result<(), String> {
    let resp = ureq::get(&def.url())
        .call()
        .map_err(|e| format!("Descarga fallida (¿sin conexión?): {e}"))?;
    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let mut reader = resp.into_body().into_reader();
    let mut file =
        std::fs::File::create(tmp).map_err(|e| format!("No se pudo crear el archivo: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut done = 0u64;
    let mut last_emit = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Descarga cancelada.".into());
        }
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Descarga interrumpida: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])
            .map_err(|e| format!("No se pudo escribir (¿sin espacio en disco?): {e}"))?;
        done += n as u64;
        if done - last_emit >= 512 * 1024 {
            last_emit = done;
            on_progress(done, total);
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    on_progress(done, total);
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if digest != def.sha256 {
        return Err("Checksum inválido: la descarga se descartó.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_y_delete() {
        let d = std::env::temp_dir().join(format!("voxflow-models-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert!(!status(&d)[0].installed);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(path(&d, "base"), b"x").unwrap();
        assert!(status(&d)[0].installed);
        delete(&d, "base").unwrap();
        assert!(!status(&d)[0].installed);
        assert!(delete(&d, "../../x").is_err());
    }

    #[test]
    fn tabla_de_modelos_valida() {
        let mut ids = std::collections::HashSet::new();
        for m in MODELS {
            assert!(ids.insert(m.id), "id repetido {}", m.id);
            assert!(m
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.'));
            assert_eq!(m.sha256.len(), 64, "{}", m.id);
            assert!(m.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
            assert!(m
                .url()
                .starts_with("https://huggingface.co/ggerganov/whisper.cpp/"));
            assert!(m.size_bytes > 100_000_000);
        }
        assert!(find("base").is_some() && find("../x").is_none());
    }

    #[test]
    fn modelo_desconocido_falla() {
        let d = std::env::temp_dir();
        assert!(download(&d, "nope", &AtomicBool::new(false), |_, _| {}).is_err());
    }
}
