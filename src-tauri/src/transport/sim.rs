use std::collections::VecDeque;

use super::{BusType, Filter, Frame, TimingConfig, Transport};

pub struct SimTransport {
  open: bool,
  queue: VecDeque<Frame>,
}

impl SimTransport {
  pub fn new() -> Self {
    Self {
      open: false,
      queue: VecDeque::new(),
    }
  }

  pub fn push_frame(&mut self, frame: Frame) {
    self.queue.push_back(frame);
  }
}

impl Transport for SimTransport {
  fn open(&mut self) -> Result<(), String> {
    self.open = true;
    Ok(())
  }

  fn close(&mut self) {
    self.open = false;
  }

  fn send(&mut self, _frame: &Frame) -> Result<(), String> {
    if !self.open {
      return Err("Sim transport not open".to_string());
    }
    Ok(())
  }

  fn recv(&mut self, _timeout_ms: u64) -> Result<Option<Frame>, String> {
    if !self.open {
      return Err("Sim transport not open".to_string());
    }
    Ok(self.queue.pop_front())
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
