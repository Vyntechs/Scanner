#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;
mod discovery;
mod logger;
mod protocol;
mod runtime;
mod scanner;
mod simulation;
mod topology;
mod transport;

use std::sync::Arc;

use tauri::Manager;

use crate::runtime::{load_last_session, AppRuntime};

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let last_session = load_last_session(app.app_handle());
      let runtime = Arc::new(AppRuntime::new(last_session));
      app.manage(runtime);
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_snapshot,
      commands::get_adapter_status,
      commands::start_scan,
      commands::clear_dtcs,
      commands::export_logs,
      commands::read_log_tail,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
