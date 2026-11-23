//! ModelManager: estado, descarga verificada (SHA-256), cancelación y borrado.
//! Un modelo solo queda "instalado" tras verificar el checksum (descarga a .part + rename).
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ModelDef {
    pub id: &'static str,
    pub sha256: &'static str,
    pub url: &'static str,
}

pub const MODELS: &[ModelDef] = &[ModelDef {
    id: "base",
    sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
}];

#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub id: String,
    pub installed: bool,
    pub size_bytes: u64,
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
                installed: meta.is_some(),
                size_bytes: meta.map(|x| x.len()).unwrap_or(0),
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
    let resp = ureq::get(def.url)
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
    fn modelo_desconocido_falla() {
        let d = std::env::temp_dir();
        assert!(download(&d, "nope", &AtomicBool::new(false), |_, _| {}).is_err());
    }
}
