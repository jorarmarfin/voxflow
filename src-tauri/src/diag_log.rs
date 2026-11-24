//! Registro de diagnóstico en archivo (`voxflow.log`) para depurar cierres inesperados.
//!
//! PRIVACIDAD: solo se escriben eventos del programa (inicio, estados, tamaños, errores
//! propios, pánicos con su ubicación). **Nunca** audio ni texto dictado. Los mensajes de
//! pánico solo se registran si son literales estáticos (no pueden contener datos del usuario).
//!
//! Ubicación: Windows → `<carpeta de instalación>\logs` (el instalador da permiso de
//! escritura) y, si no se puede, `%LOCALAPPDATA%\pe.voxflow.app\logs`; Linux →
//! `~/.local/share/pe.voxflow.app/logs`. Rotación simple a 1 MB (`voxflow.log.1`).
//! En Windows además se redirige stderr al archivo (el binario no tiene consola) y se
//! registra cualquier excepción nativa (p. ej. instrucción ilegal, violación de acceso).
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG: OnceLock<Mutex<File>> = OnceLock::new();
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
const MAX_BYTES: u64 = 1_000_000;

/// Carpetas candidatas, en orden de preferencia.
pub fn default_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    #[cfg(windows)]
    {
        if let Some(d) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("logs")))
        {
            v.push(d);
        }
        if let Some(l) = std::env::var_os("LOCALAPPDATA") {
            v.push(PathBuf::from(l).join("pe.voxflow.app").join("logs"));
        }
    }
    #[cfg(unix)]
    {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
            });
        if let Some(b) = base {
            v.push(b.join("pe.voxflow.app").join("logs"));
        }
    }
    v
}

fn try_open(dir: &Path) -> Option<(File, PathBuf)> {
    std::fs::create_dir_all(dir).ok()?;
    let path = dir.join("voxflow.log");
    if std::fs::metadata(&path)
        .map(|m| m.len() > MAX_BYTES)
        .unwrap_or(false)
    {
        let _ = std::fs::rename(&path, dir.join("voxflow.log.1"));
    }
    let f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;
    Some((f, path))
}

/// Abre el log (primera carpeta que funcione), instala el hook de pánicos y, en Windows,
/// la captura de stderr y de excepciones nativas. Devuelve la ruta del log.
pub fn init() -> Option<PathBuf> {
    let (file, path) = default_dirs().iter().find_map(|d| try_open(d))?;
    #[cfg(windows)]
    native::redirect_stderr(&file);
    let _ = LOG.set(Mutex::new(file));
    let _ = LOG_PATH.set(path.clone());
    std::panic::set_hook(Box::new(|info| {
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "?".into());
        let msg = info
            .payload()
            .downcast_ref::<&'static str>()
            .copied()
            .unwrap_or("(mensaje omitido)");
        log(&format!("PANIC en {loc}: {msg}"));
    }));
    #[cfg(windows)]
    native::install_exception_filter();
    Some(path)
}

pub fn path() -> Option<&'static Path> {
    LOG_PATH.get().map(|p| p.as_path())
}

pub fn log(msg: &str) {
    if let Some(m) = LOG.get() {
        if let Ok(mut f) = m.lock() {
            let _ = writeln!(f, "{} {}", timestamp(), msg);
            let _ = f.flush();
        }
    }
}

/// "YYYY-MM-DD HH:MM:SS.mmm" en UTC, sin dependencias.
fn timestamp() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format_utc(d.as_secs(), d.subsec_millis())
}

fn format_utc(secs: u64, millis: u32) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Algoritmo de Howard Hinnant (civil_from_days).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}.{millis:03}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Línea de arranque con datos técnicos útiles (sin rutas ni datos personales).
pub fn log_startup() {
    log(&format!(
        "VoxFlow {} arrancó ({} {})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    #[cfg(target_arch = "x86_64")]
    log(&format!(
        "CPU: avx={} avx2={} fma={} f16c={} avx512f={}",
        std::arch::is_x86_feature_detected!("avx"),
        std::arch::is_x86_feature_detected!("avx2"),
        std::arch::is_x86_feature_detected!("fma"),
        std::arch::is_x86_feature_detected!("f16c"),
        std::arch::is_x86_feature_detected!("avx512f"),
    ));
}

#[cfg(windows)]
mod native {
    use super::log;
    use std::fs::File;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Console::{SetStdHandle, STD_ERROR_HANDLE};
    use windows_sys::Win32::System::Diagnostics::Debug::{
        SetUnhandledExceptionFilter, EXCEPTION_POINTERS,
    };

    extern "C" {
        fn _open_osfhandle(osfhandle: isize, flags: i32) -> i32;
        fn _dup2(fd1: i32, fd2: i32) -> i32;
    }

    /// Redirige stderr (Win32 y CRT de C/C++, usado por whisper.cpp/ggml) al archivo de log.
    pub fn redirect_stderr(file: &File) {
        if let Ok(dup) = file.try_clone() {
            let handle = dup.as_raw_handle();
            // SAFETY: `handle` es un HANDLE válido que mantenemos abierto (se filtra a propósito).
            unsafe {
                SetStdHandle(STD_ERROR_HANDLE, handle as _);
                let fd = _open_osfhandle(handle as isize, 0);
                if fd >= 0 {
                    _dup2(fd, 2);
                }
            }
            std::mem::forget(dup);
        }
    }

    unsafe extern "system" fn filter(info: *const EXCEPTION_POINTERS) -> i32 {
        if !info.is_null() && !(*info).ExceptionRecord.is_null() {
            let r = &*(*info).ExceptionRecord;
            log(&format!(
                "EXCEPCION NATIVA código=0x{:08X} dirección={:p} (0xC000001D = instrucción ilegal; 0xC0000005 = acceso inválido; 0xC00000FD = desbordamiento de pila)",
                r.ExceptionCode as u32, r.ExceptionAddress
            ));
        }
        0 // EXCEPTION_CONTINUE_SEARCH: que Windows termine el proceso como siempre
    }

    pub fn install_exception_filter() {
        // SAFETY: `filter` tiene la firma `extern "system"` requerida y vive todo el proceso.
        unsafe {
            SetUnhandledExceptionFilter(Some(filter));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatea_utc() {
        assert_eq!(format_utc(0, 0), "1970-01-01 00:00:00.000Z");
        assert_eq!(format_utc(1_000_000_000, 5), "2001-09-09 01:46:40.005Z");
        assert_eq!(format_utc(1_795_392_000, 123), "2026-11-23 00:00:00.123Z");
    }

    #[test]
    fn rota_y_escribe() {
        let d = std::env::temp_dir().join(format!("voxflow-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let (mut f, p) = try_open(&d).unwrap();
        writeln!(f, "hola").unwrap();
        assert!(p.exists());
    }
}
