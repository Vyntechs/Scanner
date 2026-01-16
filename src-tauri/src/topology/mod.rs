use std::collections::HashMap;

use crate::app_state::{BusInfo, ModuleInfo, TopologyGraph};

pub fn build_topology(modules: &[ModuleInfo]) -> TopologyGraph {
  let mut buses: HashMap<String, Vec<String>> = HashMap::new();
  for module in modules {
    buses
      .entry(module.bus.clone())
      .or_default()
      .push(module.id.clone());
  }

  let mut bus_list = Vec::new();
  for (name, modules) in buses {
    bus_list.push(BusInfo { name, modules });
  }

  bus_list.sort_by(|a, b| a.name.cmp(&b.name));

  TopologyGraph { buses: bus_list }
}
