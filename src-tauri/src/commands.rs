use std::fs;
use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, State};

use crate::app_state::{AppSnapshot, ErrorInfo, ProgressInfo, TransportMode};
use crate::logger::{LogEvent, LogKind};
use crate::protocol::uds::UdsClient;
use crate::runtime::AppRuntime;
use crate::scanner::run_scan;
use crate::transport::VLinkerFsJ2534Transport;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterStatus {
  pub available: bool,
  pub message: String,
  pub dll_path: Option<String>,
}

#[tauri::command]
pub fn get_snapshot(state: State<Arc<AppRuntime>>) -> AppSnapshot {
  state.inner().snapshot()
}

#[tauri::command]
pub fn get_adapter_status() -> AdapterStatus {
  match VLinkerFsJ2534Transport::probe() {
    Ok(path) => AdapterStatus {
      available: true,
      message: "Adapter driver detected".to_string(),
      dll_path: Some(path.to_string_lossy().to_string()),
    },
    Err(err) => AdapterStatus {
      available: false,
      message: err,
      dll_path: None,
    },
  }
}

#[tauri::command]
pub fn start_scan(
  app: AppHandle,
  state: State<Arc<AppRuntime>>,
  mode: TransportMode,
  simulation_path: Option<String>,
) -> Result<(), String> {
  let runtime = state.inner().clone();
  tauri::async_runtime::spawn(async move {
    if let Err(err) = run_scan(app.clone(), runtime.clone(), mode, simulation_path, vec![]).await {
      runtime.update_state(&app, |state| {
        state.phase = crate::app_state::AppPhase::Error;
        state.last_error = Some(ErrorInfo {
          summary: "Scan failed".to_string(),
          details: err.clone(),
        });
        state.progress = None;
      });
      runtime.log_event(LogEvent {
        timestamp: Utc::now(),
        level: "error".to_string(),
        kind: LogKind::System,
        message: "Scan failed".to_string(),
        payload: json!({ "error": err }),
      });
    }
  });
  Ok(())
}

#[tauri::command]
pub async fn clear_dtcs(
  app: AppHandle,
  state: State<Arc<AppRuntime>>,
  module_id: Option<String>,
) -> Result<(), String> {
  let runtime = state.inner().clone();
  let snapshot = runtime.snapshot();

  runtime.update_state(&app, |state| {
    state.progress = Some(ProgressInfo {
      stage: "clearing".to_string(),
      percent: 5,
      message: "Clearing DTCs".to_string(),
    });
  });

  if snapshot.transport == TransportMode::Simulation {
    runtime.update_state(&app, |state| {
      if let Some(ref module_id) = module_id {
        state.dtcs.insert(module_id.clone(), Vec::new());
      } else {
        for dtcs in state.dtcs.values_mut() {
          dtcs.clear();
        }
      }
      for info in state.modules.iter_mut() {
        if let Some(dtcs) = state.dtcs.get(&info.id) {
          info.dtc_count = dtcs.len();
        }
      }
      state.progress = None;
    });

    runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: "info".to_string(),
      kind: LogKind::Protocol,
      message: "Cleared DTCs (simulation)".to_string(),
      payload: json!({ "module": module_id }),
    });

    return Ok(());
  }

  let modules_to_clear = {
    let state = runtime.state.lock();
    state
      .modules
      .iter()
      .filter(|module| module_id.as_ref().map(|id| id == &module.id).unwrap_or(true))
      .cloned()
      .collect::<Vec<_>>()
  };

  let mut transport_guard = runtime.transport.lock();
  let transport = transport_guard.take().ok_or_else(|| "No active transport".to_string())?;
  let mut uds = UdsClient::new(transport, 500, 1);

  for (index, module) in modules_to_clear.iter().enumerate() {
    let result = uds.clear_dtcs(module.tx_id, module.rx_id);
    let error = result.as_ref().err().cloned();
    runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: if result.is_ok() { "info" } else { "warn" }.to_string(),
      kind: LogKind::Protocol,
      message: "Clear DTCs".to_string(),
      payload: json!({ "module": module.id, "error": error }),
    });

    let percent = ((index + 1) * 100 / modules_to_clear.len().max(1)) as u8;
    runtime.update_state(&app, |state| {
      state.progress = Some(ProgressInfo {
        stage: "clearing".to_string(),
        percent,
        message: format!("Cleared {}", module.name),
      });
      if result.is_ok() {
        state.dtcs.insert(module.id.clone(), Vec::new());
      }
      for info in state.modules.iter_mut() {
        if let Some(dtcs) = state.dtcs.get(&info.id) {
          info.dtc_count = dtcs.len();
        }
      }
    });
  }

  runtime.update_state(&app, |state| {
    state.progress = None;
  });

  *transport_guard = Some(Box::new(uds.into_transport()));

  Ok(())
}

#[tauri::command]
pub fn export_logs(state: State<Arc<AppRuntime>>, destination: String) -> Result<(), String> {
  let logger = state
    .inner()
    .logger
    .lock()
    .as_ref()
    .ok_or_else(|| "No active log session".to_string())?;
  logger.copy_to(destination)
}

#[tauri::command]
pub fn read_log_tail(state: State<Arc<AppRuntime>>, lines: usize) -> Result<String, String> {
  let logger = state
    .inner()
    .logger
    .lock()
    .as_ref()
    .ok_or_else(|| "No active log session".to_string())?;
  let path = logger.path_str();
  let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
  let mut buffer = contents.lines().rev().take(lines).collect::<Vec<_>>();
  buffer.reverse();
  Ok(buffer.join("\n"))
}
