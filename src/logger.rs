use chrono::Local;
use directories::ProjectDirs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

fn data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "TitanArchitects", "TAS").map(|p| p.data_dir().to_path_buf())
}

fn log_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("tas.log"))
}

/// Initialize file logging.
///
/// Works even when running with `#![windows_subsystem="windows"]` where stdout/stderr are invisible.
pub fn init() -> Result<(), String> {
    let Some(dir) = data_dir() else { return Err("No data dir (ProjectDirs)".to_string()); };
    std::fs::create_dir_all(&dir).map_err(|e| format!("create_dir_all failed: {e}"))?;

    let Some(path) = log_path() else { return Err("No log path".to_string()); };

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open log file failed ({path:?}): {e}"))?;

    LOG_FILE
        .set(Mutex::new(file))
        .map_err(|_| "logger already initialized".to_string())?;

    info(&format!("--- TAS log started at {} ---", Local::now().format("%Y-%m-%d %H:%M:%S")));
    info(&format!("Log file: {:?}", path));
    Ok(())
}

fn write_line(level: &str, msg: &str) {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{ts}] [{level}] {msg}\n");

    if let Some(lock) = LOG_FILE.get() {
        if let Ok(mut file) = lock.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

pub fn info(msg: &str) { write_line("INFO", msg); }
#[allow(dead_code)] // Agregado para silenciar warning
pub fn warn(msg: &str) { write_line("WARN", msg); }
#[allow(dead_code)] // Agregado para silenciar warning
pub fn error(msg: &str) { write_line("ERROR", msg); }