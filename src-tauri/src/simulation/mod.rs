use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::app_state::{DtcInfo, ModuleInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationSession {
  pub vin: String,
  pub vehicle: VehicleInfo,
  pub modules: Vec<SimulationModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleInfo {
  pub make: String,
  pub model: String,
  pub year: String,
  pub trim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationModule {
  pub id: String,
  pub name: String,
  pub bus: String,
  pub category: String,
  pub tx_id: u32,
  pub rx_id: u32,
  pub dtcs: Vec<DtcInfo>,
}

impl SimulationSession {
  pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
    let contents = fs::read_to_string(path.as_ref())
      .map_err(|err| format!("Failed to read simulation file: {err}"))?;
    serde_json::from_str(&contents)
      .map_err(|err| format!("Invalid simulation file: {err}"))
  }

  pub fn module_infos(&self) -> Vec<ModuleInfo> {
    self.modules
      .iter()
      .map(|module| ModuleInfo {
        id: module.id.clone(),
        name: module.name.clone(),
        bus: module.bus.clone(),
        category: module.category.clone(),
        tx_id: module.tx_id,
        rx_id: module.rx_id,
        status: crate::app_state::ModuleStatus::Ok,
        dtc_count: module.dtcs.len(),
      })
      .collect()
  }
}
