use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AppPhase {
  Disconnected,
  Connecting,
  Identifying,
  Discovering,
  ScanningDtc,
  Ready,
  Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransportMode {
  Simulation,
  J2534,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ModuleStatus {
  Ok,
  NoResponse,
  Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInfo {
  pub id: String,
  pub name: String,
  pub bus: String,
  pub category: String,
  pub tx_id: u32,
  pub rx_id: u32,
  pub status: ModuleStatus,
  pub dtc_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtcInfo {
  pub code: String,
  pub description: String,
  pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusInfo {
  pub name: String,
  pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyGraph {
  pub buses: Vec<BusInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressInfo {
  pub stage: String,
  pub percent: u8,
  pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorInfo {
  pub summary: String,
  pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
  pub session_id: String,
  pub timestamp: DateTime<Utc>,
  pub vin: Option<String>,
  pub module_count: usize,
  pub dtc_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
  pub phase: AppPhase,
  pub transport: TransportMode,
  pub adapter_connected: bool,
  pub vin: Option<String>,
  pub modules: Vec<ModuleInfo>,
  pub dtcs: HashMap<String, Vec<DtcInfo>>,
  pub topology: TopologyGraph,
  pub progress: Option<ProgressInfo>,
  pub last_error: Option<ErrorInfo>,
  pub session_id: Option<String>,
  pub logs_path: Option<String>,
  pub last_session: Option<SessionSummary>,
}

#[derive(Debug, Clone)]
pub struct AppState {
  pub phase: AppPhase,
  pub transport: TransportMode,
  pub adapter_connected: bool,
  pub vin: Option<String>,
  pub modules: Vec<ModuleInfo>,
  pub dtcs: HashMap<String, Vec<DtcInfo>>,
  pub topology: TopologyGraph,
  pub progress: Option<ProgressInfo>,
  pub last_error: Option<ErrorInfo>,
  pub session_id: Option<String>,
  pub logs_path: Option<String>,
  pub last_session: Option<SessionSummary>,
}

impl Default for AppState {
  fn default() -> Self {
    Self {
      phase: AppPhase::Disconnected,
      transport: TransportMode::Simulation,
      adapter_connected: false,
      vin: None,
      modules: Vec::new(),
      dtcs: HashMap::new(),
      topology: TopologyGraph { buses: Vec::new() },
      progress: None,
      last_error: None,
      session_id: None,
      logs_path: None,
      last_session: None,
    }
  }
}

impl AppState {
  pub fn snapshot(&self) -> AppSnapshot {
    AppSnapshot {
      phase: self.phase.clone(),
      transport: self.transport.clone(),
      adapter_connected: self.adapter_connected,
      vin: self.vin.clone(),
      modules: self.modules.clone(),
      dtcs: self.dtcs.clone(),
      topology: self.topology.clone(),
      progress: self.progress.clone(),
      last_error: self.last_error.clone(),
      session_id: self.session_id.clone(),
      logs_path: self.logs_path.clone(),
      last_session: self.last_session.clone(),
    }
  }
}
