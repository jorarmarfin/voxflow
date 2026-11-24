//! Preferencias persistentes (JSON en el directorio de configuración de la app).
//! No contiene audio ni transcripciones.
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Preview,
    Clipboard,
    /// Pega en la ventana activa al terminar (X11 + xdotool); si no es seguro, copia.
    Insert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub device_id: Option<String>,
    pub model: String,
    pub language: String,
    pub output_mode: OutputMode,
    pub start_minimized: bool,
    /// Atajo global (solo Windows; en Linux se usa `voxflow --toggle` desde el escritorio).
    pub hotkey: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            device_id: None,
            model: "base".into(),
            language: "es".into(),
            output_mode: OutputMode::Preview,
            start_minimized: false,
            hotkey: "Ctrl+Alt+V".into(),
        }
    }
}

const LANGUAGES: &[&str] = &["es", "en"];

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if crate::models::find(&self.model).is_none() {
            return Err(format!("modelo no soportado: {}", self.model));
        }
        if !LANGUAGES.contains(&self.language.as_str()) {
            return Err(format!("idioma no soportado: {}", self.language));
        }
        let ok_chars = self
            .hotkey
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+ _".contains(c));
        if self.hotkey.trim().is_empty() || self.hotkey.len() > 40 || !ok_chars {
            return Err("atajo no válido (ejemplo: Ctrl+Alt+V)".into());
        }
        Ok(())
    }

    /// Carga ajustes; si falta o está corrupto, devuelve los valores por defecto.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
            .filter(|s| s.validate().is_ok())
            .unwrap_or_default()
    }

    /// Escritura atómica (archivo temporal + rename).
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("voxflow-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("settings.json")
    }

    #[test]
    fn roundtrip() {
        let p = tmp("rt");
        let s = Settings {
            language: "en".into(),
            start_minimized: true,
            ..Default::default()
        };
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p), s);
    }

    #[test]
    fn archivo_corrupto_da_defaults() {
        let p = tmp("bad");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "{no es json").unwrap();
        assert_eq!(Settings::load(&p), Settings::default());
    }

    #[test]
    fn rechaza_atajo_invalido() {
        let s = Settings {
            hotkey: "Ctrl+Alt+V; rm".into(),
            ..Default::default()
        };
        assert!(s.validate().is_err());
        let s = Settings {
            hotkey: "".into(),
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn rechaza_valores_invalidos() {
        let s = Settings {
            model: "../../etc/passwd".into(),
            ..Default::default()
        };
        assert!(s.save(&tmp("inv")).is_err());
    }
}
