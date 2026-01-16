use std::fs;
use std::path::PathBuf;

use parking_lot::Mutex;
use tauri::AppHandle;

use crate::app_state::{AppSnapshot, AppState, SessionSummary};
use crate::logger::{LogEvent, Logger};
use crate::transport::Transport;

pub struct AppRuntime {
  pub state: Mutex<AppState>,
  pub logger: Mutex<Option<Logger>>,
  pub transport: Mutex<Option<Box<dyn Transport>>>,
  pub simulation: Mutex<Option<crate::simulation::SimulationSession>>,
}

impl AppRuntime {
  pub fn new(last_session: Option<SessionSummary>) -> Self {
    let mut state = AppState::default();
    state.last_session = last_session;
    Self {
      state: Mutex::new(state),
      logger: Mutex::new(None),
      transport: Mutex::new(None),
      simulation: Mutex::new(None),
    }
  }

  pub fn snapshot(&self) -> AppSnapshot {
    self.state.lock().snapshot()
  }

  pub fn update_state<F>(&self, app: &AppHandle, update: F) -> AppSnapshot
  where
    F: FnOnce(&mut AppState),
  {
    let snapshot = {
      let mut state = self.state.lock();
      update(&mut state);
      state.snapshot()
    };
    let _ = app.emit_all("app://snapshot", snapshot.clone());
    snapshot
  }

  pub fn set_logger(&self, logger: Option<Logger>) {
    let mut guard = self.logger.lock();
    *guard = logger;
  }

  pub fn log_event(&self, event: LogEvent) {
    if let Some(logger) = self.logger.lock().as_ref() {
      logger.log(event);
    }
  }
}

pub fn last_session_path(app: &AppHandle) -> Option<PathBuf> {
  let base = tauri::api::path::app_data_dir(&app.config())?;
  Some(base.join("last_session.json"))
}

pub fn load_last_session(app: &AppHandle) -> Option<SessionSummary> {
  let path = last_session_path(app)?;
  let contents = fs::read_to_string(path).ok()?;
  serde_json::from_str(&contents).ok()
}

pub fn save_last_session(app: &AppHandle, summary: &SessionSummary) -> Result<(), String> {
  let path = last_session_path(app).ok_or_else(|| "Missing app data dir".to_string())?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("Failed to create app data dir: {err}"))?;
  }
  let contents = serde_json::to_string_pretty(summary).map_err(|err| err.to_string())?;
  fs::write(path, contents).map_err(|err| format!("Failed to save last session: {err}"))
}
