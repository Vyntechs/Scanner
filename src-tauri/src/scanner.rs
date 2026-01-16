use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde_json::json;
use tauri::AppHandle;

use crate::app_state::{
  AppPhase, ErrorInfo, ModuleStatus, ProgressInfo, SessionSummary, TransportMode,
};
use crate::discovery::{decode_vin, discover_modules, ModuleCandidate};
use crate::logger::{LogEvent, LogKind, Logger};
use crate::protocol::uds::UdsClient;
use crate::runtime::{save_last_session, AppRuntime};
use crate::simulation::SimulationSession;
use crate::topology::build_topology;
use crate::transport::{Transport, VLinkerFsJ2534Transport};

struct LoggingTransport<T: Transport> {
  inner: T,
  runtime: Arc<AppRuntime>,
}

impl<T: Transport> LoggingTransport<T> {
  fn new(inner: T, runtime: Arc<AppRuntime>) -> Self {
    Self { inner, runtime }
  }

  fn log_frame(&self, direction: &str, frame: &crate::transport::Frame) {
    self.runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: "debug".to_string(),
      kind: LogKind::Transport,
      message: format!("{direction} CAN frame"),
      payload: json!({
        "id": format!("0x{:03X}", frame.id),
        "data": frame.data,
        "extended": frame.is_extended
      }),
    });
  }
}

impl<T: Transport> Transport for LoggingTransport<T> {
  fn open(&mut self) -> Result<(), String> {
    self.inner.open()
  }

  fn close(&mut self) {
    self.inner.close();
  }

  fn send(&mut self, frame: &crate::transport::Frame) -> Result<(), String> {
    self.log_frame("TX", frame);
    self.inner.send(frame)
  }

  fn recv(&mut self, timeout_ms: u64) -> Result<Option<crate::transport::Frame>, String> {
    let frame = self.inner.recv(timeout_ms)?;
    if let Some(ref frame) = frame {
      self.log_frame("RX", frame);
    }
    Ok(frame)
  }

  fn set_filters(&mut self, filters: Vec<crate::transport::Filter>) -> Result<(), String> {
    self.inner.set_filters(filters)
  }

  fn set_baud(&mut self, baud: u32) -> Result<(), String> {
    self.inner.set_baud(baud)
  }

  fn set_bus(&mut self, bus: crate::transport::BusType) -> Result<(), String> {
    self.inner.set_bus(bus)
  }

  fn set_timing(&mut self, timing: crate::transport::TimingConfig) -> Result<(), String> {
    self.inner.set_timing(timing)
  }
}

pub async fn run_scan(
  app: AppHandle,
  runtime: Arc<AppRuntime>,
  mode: TransportMode,
  simulation_path: Option<String>,
  extra_candidates: Vec<ModuleCandidate>,
) -> Result<(), String> {
  let session_id = uuid::Uuid::new_v4().to_string();
  let logger = Logger::new(&app, &session_id)?;
  let logs_path = logger.path_str();
  runtime.set_logger(Some(logger));

  runtime.update_state(&app, |state| {
    state.phase = AppPhase::Connecting;
    state.transport = mode.clone();
    state.adapter_connected = mode == TransportMode::Simulation;
    state.vin = None;
    state.modules.clear();
    state.dtcs.clear();
    state.topology = crate::app_state::TopologyGraph { buses: Vec::new() };
    state.progress = Some(ProgressInfo {
      stage: "connecting".to_string(),
      percent: 5,
      message: "Starting session".to_string(),
    });
    state.last_error = None;
    state.session_id = Some(session_id.clone());
    state.logs_path = Some(logs_path);
  });

  runtime.log_event(LogEvent {
    timestamp: Utc::now(),
    level: "info".to_string(),
    kind: LogKind::System,
    message: "Session started".to_string(),
    payload: json!({ "sessionId": session_id, "mode": format!("{mode:?}") }),
  });

  match mode {
    TransportMode::Simulation => run_simulation(&app, runtime, simulation_path).await,
    TransportMode::J2534 => run_real_scan(&app, runtime, extra_candidates).await,
  }
}

async fn run_simulation(
  app: &AppHandle,
  runtime: Arc<AppRuntime>,
  simulation_path: Option<String>,
) -> Result<(), String> {
  let path = simulation_path.unwrap_or_else(|| "samples/f250_session.json".to_string());
  let session = resolve_simulation(&path).ok_or_else(|| {
    format!("Simulation file not found: {path}. Provide a valid path or keep /samples in the repo.")
  })?;
  *runtime.simulation.lock() = Some(session.clone());

  runtime.update_state(app, |state| {
    state.adapter_connected = true;
    state.phase = AppPhase::Identifying;
    state.progress = Some(ProgressInfo {
      stage: "identifying".to_string(),
      percent: 15,
      message: "Reading VIN".to_string(),
    });
  });
  runtime.log_event(LogEvent {
    timestamp: Utc::now(),
    level: "info".to_string(),
    kind: LogKind::Protocol,
    message: "VIN read (simulation)".to_string(),
    payload: json!({ "vin": session.vin }),
  });
  if let Some(info) = decode_vin(&session.vin) {
    runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: "info".to_string(),
      kind: LogKind::Protocol,
      message: "VIN decode (simulation)".to_string(),
      payload: json!({ "wmi": info.wmi, "year": info.year }),
    });
  }

  tauri::async_runtime::sleep(Duration::from_millis(350)).await;

  runtime.update_state(app, |state| {
    state.vin = Some(session.vin.clone());
    state.phase = AppPhase::Discovering;
    state.progress = Some(ProgressInfo {
      stage: "discovering".to_string(),
      percent: 30,
      message: "Discovering modules".to_string(),
    });
  });

  let mut modules = Vec::new();
  for (index, module) in session.modules.iter().enumerate() {
    modules.push(crate::app_state::ModuleInfo {
      id: module.id.clone(),
      name: module.name.clone(),
      bus: module.bus.clone(),
      category: module.category.clone(),
      tx_id: module.tx_id,
      rx_id: module.rx_id,
      status: ModuleStatus::Ok,
      dtc_count: module.dtcs.len(),
    });
    let percent = 30 + ((index + 1) * 30 / session.modules.len().max(1));
    runtime.update_state(app, |state| {
      state.modules = modules.clone();
      state.topology = build_topology(&state.modules);
      state.progress = Some(ProgressInfo {
        stage: "discovering".to_string(),
        percent: percent as u8,
        message: format!("Discovered {}", module.name),
      });
    });
    runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: "info".to_string(),
      kind: LogKind::Protocol,
      message: "Module discovered".to_string(),
      payload: json!({ "module": module.id, "name": module.name }),
    });
    tauri::async_runtime::sleep(Duration::from_millis(180)).await;
  }

  runtime.update_state(app, |state| {
    state.phase = AppPhase::ScanningDtc;
    state.progress = Some(ProgressInfo {
      stage: "scanning".to_string(),
      percent: 70,
      message: "Reading DTCs".to_string(),
    });
  });

  let mut dtcs_map = std::collections::HashMap::new();
  for (index, module) in session.modules.iter().enumerate() {
    dtcs_map.insert(module.id.clone(), module.dtcs.clone());
    let percent = 70 + ((index + 1) * 25 / session.modules.len().max(1));
    runtime.update_state(app, |state| {
      state.dtcs = dtcs_map.clone();
      for info in state.modules.iter_mut() {
        if let Some(dtcs) = state.dtcs.get(&info.id) {
          info.dtc_count = dtcs.len();
        }
      }
      state.progress = Some(ProgressInfo {
        stage: "scanning".to_string(),
        percent: percent as u8,
        message: format!("Scanned {}", module.name),
      });
    });
    tauri::async_runtime::sleep(Duration::from_millis(160)).await;
  }

  finish_session(app, runtime).await;
  Ok(())
}

fn resolve_simulation(path: &str) -> Option<SimulationSession> {
  let candidate = std::path::PathBuf::from(path);
  if candidate.exists() {
    return SimulationSession::load_from_file(candidate).ok();
  }
  if let Ok(cwd) = std::env::current_dir() {
    let direct = cwd.join(path);
    if direct.exists() {
      return SimulationSession::load_from_file(direct).ok();
    }
    let parent = cwd.join("..").join(path);
    if parent.exists() {
      return SimulationSession::load_from_file(parent).ok();
    }
  }
  None
}

async fn run_real_scan(
  app: &AppHandle,
  runtime: Arc<AppRuntime>,
  extra_candidates: Vec<ModuleCandidate>,
) -> Result<(), String> {
  let transport = VLinkerFsJ2534Transport::new(None);
  let transport = LoggingTransport::new(transport, runtime.clone());
  let mut uds = UdsClient::new(transport, 500, 1);

  uds.open().map_err(|err| {
    runtime.update_state(app, |state| {
      state.phase = AppPhase::Error;
      state.last_error = Some(ErrorInfo {
        summary: "Adapter connection failed".to_string(),
        details: err.clone(),
      });
      state.progress = None;
    });
    err
  })?;

  runtime.update_state(app, |state| {
    state.adapter_connected = true;
    state.phase = AppPhase::Identifying;
    state.progress = Some(ProgressInfo {
      stage: "identifying".to_string(),
      percent: 18,
      message: "Reading VIN".to_string(),
    });
  });

  let vin = uds
    .read_vin(0x7E0, 0x7E8)
    .or_else(|_| uds.read_vin(0x7DF, 0x7E8))
    .map_err(|err| {
      runtime.update_state(app, |state| {
        state.phase = AppPhase::Error;
        state.last_error = Some(ErrorInfo {
          summary: "VIN read failed".to_string(),
          details: err.clone(),
        });
        state.progress = None;
      });
      err
    })?;

  runtime.log_event(LogEvent {
    timestamp: Utc::now(),
    level: "info".to_string(),
    kind: LogKind::Protocol,
    message: "VIN read".to_string(),
    payload: json!({ "vin": vin }),
  });
  if let Some(info) = decode_vin(&vin) {
    runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: "info".to_string(),
      kind: LogKind::Protocol,
      message: "VIN decode".to_string(),
      payload: json!({ "wmi": info.wmi, "year": info.year }),
    });
  }

  runtime.update_state(app, |state| {
    state.vin = Some(vin.clone());
    state.phase = AppPhase::Discovering;
    state.progress = Some(ProgressInfo {
      stage: "discovering".to_string(),
      percent: 35,
      message: "Discovering modules".to_string(),
    });
  });

  let modules = discover_modules(&mut uds, &extra_candidates);
  runtime.update_state(app, |state| {
    state.modules = modules.clone();
    state.topology = build_topology(&state.modules);
    state.progress = Some(ProgressInfo {
      stage: "discovering".to_string(),
      percent: 55,
      message: format!("Discovered {} modules", state.modules.len()),
    });
  });

  runtime.update_state(app, |state| {
    state.phase = AppPhase::ScanningDtc;
    state.progress = Some(ProgressInfo {
      stage: "scanning".to_string(),
      percent: 65,
      message: "Reading DTCs".to_string(),
    });
  });

  let mut dtcs_map = std::collections::HashMap::new();
  let module_count = modules.len().max(1);
  for (index, module) in modules.iter().enumerate() {
    match uds.read_dtcs(module.tx_id, module.rx_id) {
      Ok(dtcs) => {
        dtcs_map.insert(module.id.clone(), dtcs);
      }
      Err(err) => {
        runtime.log_event(LogEvent {
          timestamp: Utc::now(),
          level: "warn".to_string(),
          kind: LogKind::Protocol,
          message: "DTC read failed".to_string(),
          payload: json!({ "module": module.id, "error": err }),
        });
      }
    }

    let percent = 65 + ((index + 1) * 30 / module_count);
    runtime.update_state(app, |state| {
      state.dtcs = dtcs_map.clone();
      for info in state.modules.iter_mut() {
        if let Some(dtcs) = state.dtcs.get(&info.id) {
          info.dtc_count = dtcs.len();
        }
      }
      state.progress = Some(ProgressInfo {
        stage: "scanning".to_string(),
        percent: percent as u8,
        message: format!("Scanned {}", module.name),
      });
    });
  }

  let transport = uds.into_transport();
  *runtime.transport.lock() = Some(Box::new(transport));

  finish_session(app, runtime).await;
  Ok(())
}

async fn finish_session(app: &AppHandle, runtime: Arc<AppRuntime>) {
  runtime.update_state(app, |state| {
    state.phase = AppPhase::Ready;
    state.progress = None;
  });

  let summary = {
    let state = runtime.state.lock();
    SessionSummary {
      session_id: state.session_id.clone().unwrap_or_default(),
      timestamp: Utc::now(),
      vin: state.vin.clone(),
      module_count: state.modules.len(),
      dtc_count: state
        .dtcs
        .values()
        .map(|items| items.len())
        .sum::<usize>(),
    }
  };

  if let Err(err) = save_last_session(app, &summary) {
    runtime.log_event(LogEvent {
      timestamp: Utc::now(),
      level: "warn".to_string(),
      kind: LogKind::System,
      message: "Failed to save session summary".to_string(),
      payload: json!({ "error": err }),
    });
  }

  runtime.update_state(app, |state| {
    state.last_session = Some(summary);
  });
}
