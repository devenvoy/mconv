use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Logger {
    log_path: PathBuf,
    file: Mutex<Option<File>>,
}

impl Logger {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let log_dir = Path::new(&home).join(".mconv").join("logs");
        let _ = create_dir_all(&log_dir);

        let filename = format!("mconv_{}_{}.log", current_timestamp_str(), std::process::id());
        let log_path = log_dir.join(filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        Self {
            log_path,
            file: Mutex::new(file),
        }
    }

    pub fn path(&self) -> &Path {
        &self.log_path
    }

    pub fn log(&self, level: &str, message: &str) {
        let formatted = format!("{} [{:<4}] {}\n", current_time_str(), level, message);

        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut f) = *guard {
                let _ = f.write_all(formatted.as_bytes());
                let _ = f.flush();
            }
        }
    }

    pub fn ok(&self, msg: &str) {
        self.log("OK", msg);
    }

    pub fn skip(&self, msg: &str) {
        self.log("SKIP", msg);
    }

    pub fn fail(&self, msg: &str) {
        self.log("FAIL", msg);
    }

    pub fn del(&self, msg: &str) {
        self.log("DEL", msg);
    }
}

fn current_time_str() -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
    }
}

fn current_timestamp_str() -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        format!(
            "{:04}{:02}{:02}_{:02}{:02}{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    }
}
