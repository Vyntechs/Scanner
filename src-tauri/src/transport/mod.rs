mod j2534;
mod direct;
mod sim;
mod types;

pub use direct::VLinkerDirectTransport;
pub use j2534::VLinkerFsJ2534Transport;
pub use sim::SimTransport;
pub use types::{BusType, Filter, Frame, TimingConfig};

pub trait Transport: Send {
  fn open(&mut self) -> Result<(), String>;
  fn close(&mut self);
  fn send(&mut self, frame: &Frame) -> Result<(), String>;
  fn recv(&mut self, timeout_ms: u64) -> Result<Option<Frame>, String>;
  fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), String>;
  fn set_baud(&mut self, baud: u32) -> Result<(), String>;
  fn set_bus(&mut self, bus: BusType) -> Result<(), String>;
  fn set_timing(&mut self, timing: TimingConfig) -> Result<(), String>;
}

impl Transport for Box<dyn Transport> {
  fn open(&mut self) -> Result<(), String> {
    self.as_mut().open()
  }

  fn close(&mut self) {
    self.as_mut().close();
  }

  fn send(&mut self, frame: &Frame) -> Result<(), String> {
    self.as_mut().send(frame)
  }

  fn recv(&mut self, timeout_ms: u64) -> Result<Option<Frame>, String> {
    self.as_mut().recv(timeout_ms)
  }

  fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), String> {
    self.as_mut().set_filters(filters)
  }

  fn set_baud(&mut self, baud: u32) -> Result<(), String> {
    self.as_mut().set_baud(baud)
  }

  fn set_bus(&mut self, bus: BusType) -> Result<(), String> {
    self.as_mut().set_bus(bus)
  }

  fn set_timing(&mut self, timing: TimingConfig) -> Result<(), String> {
    self.as_mut().set_timing(timing)
  }
}
