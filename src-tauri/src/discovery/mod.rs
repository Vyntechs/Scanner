use std::collections::HashSet;

use crate::app_state::{ModuleInfo, ModuleStatus};
use crate::protocol::uds::UdsClient;
use crate::transport::Transport;

#[derive(Debug, Clone)]
pub struct ModuleCandidate {
  pub tx_id: u32,
  pub rx_id: u32,
  pub name: String,
  pub bus: String,
  pub category: String,
}

#[derive(Debug, Clone)]
pub struct VinInfo {
  pub wmi: String,
  pub year: String,
}

pub fn decode_vin(vin: &str) -> Option<VinInfo> {
  if vin.len() < 3 {
    return None;
  }
  let wmi = vin[0..3].to_string();
  let year_code = vin.chars().nth(9)?;
  let year = match year_code {
    'K' => "2019",
    'L' => "2020",
    'M' => "2021",
    'N' => "2022",
    'P' => "2023",
    'R' => "2024",
    _ => "Unknown",
  };
  Some(VinInfo {
    wmi,
    year: year.to_string(),
  })
}

pub fn gateway_inventory<T: Transport>(_uds: &mut UdsClient<T>) -> Vec<ModuleInfo> {
  // Placeholder for OEM-specific gateway inventory queries.
  Vec::new()
}

pub fn default_candidates() -> Vec<ModuleCandidate> {
  vec![
    ModuleCandidate {
      tx_id: 0x7E0,
      rx_id: 0x7E8,
      name: "PCM".to_string(),
      bus: "HS-CAN".to_string(),
      category: "Powertrain".to_string(),
    },
    ModuleCandidate {
      tx_id: 0x7E1,
      rx_id: 0x7E9,
      name: "TCM".to_string(),
      bus: "HS-CAN".to_string(),
      category: "Powertrain".to_string(),
    },
    ModuleCandidate {
      tx_id: 0x726,
      rx_id: 0x72E,
      name: "ABS".to_string(),
      bus: "HS-CAN".to_string(),
      category: "Chassis".to_string(),
    },
    ModuleCandidate {
      tx_id: 0x727,
      rx_id: 0x72F,
      name: "BCM".to_string(),
      bus: "MS-CAN".to_string(),
      category: "Body".to_string(),
    },
  ]
}

pub fn discover_modules<T: Transport>(
  uds: &mut UdsClient<T>,
  extra_candidates: &[ModuleCandidate],
) -> Vec<ModuleInfo> {
  let mut modules = Vec::new();
  let mut candidates = default_candidates();
  candidates.extend_from_slice(extra_candidates);
  let mut seen: HashSet<u32> = HashSet::new();

  for candidate in candidates {
    let response = uds.tester_present(candidate.tx_id, candidate.rx_id);
    if response.is_ok() {
      let id = format!("0x{:03X}", candidate.tx_id);
      modules.push(ModuleInfo {
        id,
        name: candidate.name.clone(),
        bus: candidate.bus.clone(),
        category: candidate.category.clone(),
        tx_id: candidate.tx_id,
        rx_id: candidate.rx_id,
        status: ModuleStatus::Ok,
        dtc_count: 0,
      });
      seen.insert(candidate.tx_id);
    }
  }

  for tx_id in 0x700u32..=0x7E7u32 {
    if seen.contains(&tx_id) {
      continue;
    }
    let rx_id = tx_id + 0x8;
    if uds.tester_present(tx_id, rx_id).is_ok() {
      let id = format!("0x{:03X}", tx_id);
      modules.push(ModuleInfo {
        id: id.clone(),
        name: format!("ECU {id}"),
        bus: "Unknown".to_string(),
        category: "Unknown".to_string(),
        tx_id,
        rx_id,
        status: ModuleStatus::Ok,
        dtc_count: 0,
      });
    }
  }

  modules
}
