use directories::ProjectDirs;
use std::backtrace::Backtrace;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn best_effort_dir() -> PathBuf {
    // Prefer app data dir; fallback to %TEMP%\TAS
    if let Some(p) = ProjectDirs::from("com", "TitanArchitects", "TAS") {
        p.data_dir().to_path_buf()
    } else {
        std::env::temp_dir().join("TAS")
    }
}

fn panic_log_path() -> PathBuf {
    best_effort_dir().join("tas_panic.log")
}

fn append_line(tag: &str, msg: &str) {
    let path = panic_log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{tag}] {msg}");
        let _ = f.flush();
    }
}

pub fn note(tag: &str, msg: impl AsRef<str>) {
    append_line(tag, msg.as_ref());
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        // Force capture works without env vars (avoids std::env::set_var, which is unsafe in your toolchain)
        let bt = Backtrace::force_capture();
        append_line("PANIC", &format!("{info}"));
        append_line("BACKTRACE", &format!("{bt}"));
    }));
}
