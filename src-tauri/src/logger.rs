use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::AppHandle;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LogKind {
  Transport,
  Protocol,
  System,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
  pub timestamp: DateTime<Utc>,
  pub level: String,
  pub kind: LogKind,
  pub message: String,
  pub payload: serde_json::Value,
}

pub struct Logger {
  file: Mutex<File>,
  path: PathBuf,
}

impl Logger {
  pub fn new(app_handle: &AppHandle, session_id: &str) -> Result<Self, String> {
    let base = tauri::api::path::app_data_dir(&app_handle.config())
      .ok_or_else(|| "Unable to determine app data directory".to_string())?;
    let logs_dir = base.join("logs");
    create_dir_all(&logs_dir).map_err(|err| format!("Failed to create logs dir: {err}"))?;
    let path = logs_dir.join(format!("session_{session_id}.jsonl"));
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&path)
      .map_err(|err| format!("Failed to open log file: {err}"))?;

    Ok(Self {
      file: Mutex::new(file),
      path,
    })
  }

  pub fn log(&self, event: LogEvent) {
    if let Ok(line) = serde_json::to_string(&event) {
      if let Ok(mut file) = self.file.lock() {
        let _ = writeln!(file, "{line}");
      }
    }
  }

  pub fn path_str(&self) -> String {
    self.path.to_string_lossy().to_string()
  }

  pub fn copy_to<P: AsRef<Path>>(&self, destination: P) -> Result<(), String> {
    std::fs::copy(&self.path, destination)
      .map(|_| ())
      .map_err(|err| format!("Failed to export logs: {err}"))
  }
}
