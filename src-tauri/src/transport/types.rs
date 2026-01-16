use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
  pub id: u32,
  pub data: Vec<u8>,
  pub timestamp_ms: u128,
  pub is_extended: bool,
}

#[derive(Debug, Clone)]
pub struct Filter {
  pub id: u32,
  pub mask: u32,
  pub is_extended: bool,
}

#[derive(Debug, Clone)]
pub enum BusType {
  Can,
}

#[derive(Debug, Clone)]
pub struct TimingConfig {
  pub p2_ms: u64,
  pub p2_star_ms: u64,
}
