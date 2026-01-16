use super::{BusType, Filter, Frame, TimingConfig, Transport};

pub struct VLinkerDirectTransport;

impl VLinkerDirectTransport {
  pub fn new() -> Self {
    Self
  }
}

impl Transport for VLinkerDirectTransport {
  fn open(&mut self) -> Result<(), String> {
    Err("VLinker direct transport not implemented (TODO)".to_string())
  }

  fn close(&mut self) {}

  fn send(&mut self, _frame: &Frame) -> Result<(), String> {
    Err("VLinker direct transport not implemented (TODO)".to_string())
  }

  fn recv(&mut self, _timeout_ms: u64) -> Result<Option<Frame>, String> {
    Err("VLinker direct transport not implemented (TODO)".to_string())
  }

  fn set_filters(&mut self, _filters: Vec<Filter>) -> Result<(), String> {
    Ok(())
  }

  fn set_baud(&mut self, _baud: u32) -> Result<(), String> {
    Ok(())
  }

  fn set_bus(&mut self, _bus: BusType) -> Result<(), String> {
    Ok(())
  }

  fn set_timing(&mut self, _timing: TimingConfig) -> Result<(), String> {
    Ok(())
  }
}
