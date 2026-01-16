use super::{BusType, Filter, Frame, TimingConfig, Transport};
use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::path::{Path, PathBuf};

const STATUS_NOERROR: u32 = 0x00;
const ERR_TIMEOUT: u32 = 0x0A;
const PROTOCOL_CAN: u32 = 0x00000005;
const PASS_FILTER: u32 = 0x00000001;
const CAN_29BIT_ID: u32 = 0x00000100;

#[repr(C)]
#[derive(Clone, Copy)]
struct PassThruMsg {
  protocol_id: u32,
  rx_status: u32,
  tx_flags: u32,
  timestamp: u32,
  data_size: u32,
  extra_data_index: u32,
  data: [u8; 4128],
}

impl Default for PassThruMsg {
  fn default() -> Self {
    Self {
      protocol_id: 0,
      rx_status: 0,
      tx_flags: 0,
      timestamp: 0,
      data_size: 0,
      extra_data_index: 0,
      data: [0u8; 4128],
    }
  }
}

type PassThruOpen = unsafe extern "C" fn(*mut c_void, *mut u32) -> u32;
type PassThruClose = unsafe extern "C" fn(u32) -> u32;
type PassThruConnect = unsafe extern "C" fn(u32, u32, u32, u32, *mut u32) -> u32;
type PassThruDisconnect = unsafe extern "C" fn(u32) -> u32;
type PassThruReadMsgs = unsafe extern "C" fn(u32, *mut PassThruMsg, *mut u32, u32) -> u32;
type PassThruWriteMsgs = unsafe extern "C" fn(u32, *mut PassThruMsg, *mut u32, u32) -> u32;
type PassThruStartMsgFilter =
  unsafe extern "C" fn(u32, u32, *mut PassThruMsg, *mut PassThruMsg, *mut PassThruMsg, *mut u32) -> u32;

pub struct J2534Library {
  _lib: Library,
  open: PassThruOpen,
  close: PassThruClose,
  connect: PassThruConnect,
  disconnect: PassThruDisconnect,
  read_msgs: PassThruReadMsgs,
  write_msgs: PassThruWriteMsgs,
  start_filter: Option<PassThruStartMsgFilter>,
}

impl J2534Library {
  unsafe fn new(path: &Path) -> Result<Self, String> {
    let lib = Library::new(path).map_err(|err| format!("Failed to load J2534 DLL: {err}"))?;

    let open: Symbol<PassThruOpen> = lib.get(b"PassThruOpen").map_err(|err| err.to_string())?;
    let close: Symbol<PassThruClose> = lib.get(b"PassThruClose").map_err(|err| err.to_string())?;
    let connect: Symbol<PassThruConnect> = lib.get(b"PassThruConnect").map_err(|err| err.to_string())?;
    let disconnect: Symbol<PassThruDisconnect> =
      lib.get(b"PassThruDisconnect").map_err(|err| err.to_string())?;
    let read_msgs: Symbol<PassThruReadMsgs> = lib.get(b"PassThruReadMsgs").map_err(|err| err.to_string())?;
    let write_msgs: Symbol<PassThruWriteMsgs> = lib.get(b"PassThruWriteMsgs").map_err(|err| err.to_string())?;
    let start_filter = lib.get(b"PassThruStartMsgFilter").ok().map(|symbol: Symbol<PassThruStartMsgFilter>| *symbol);

    Ok(Self {
      _lib: lib,
      open: *open,
      close: *close,
      connect: *connect,
      disconnect: *disconnect,
      read_msgs: *read_msgs,
      write_msgs: *write_msgs,
      start_filter,
    })
  }
}

pub struct VLinkerFsJ2534Transport {
  dll_path: Option<PathBuf>,
  lib: Option<J2534Library>,
  device_id: u32,
  channel_id: u32,
  baud: u32,
  is_open: bool,
}

impl VLinkerFsJ2534Transport {
  pub fn new(dll_path: Option<PathBuf>) -> Self {
    Self {
      dll_path,
      lib: None,
      device_id: 0,
      channel_id: 0,
      baud: 500_000,
      is_open: false,
    }
  }

  pub fn probe() -> Result<PathBuf, String> {
    find_j2534_dll()
  }

  fn load_library(&mut self) -> Result<(), String> {
    let dll_path = match self.dll_path.clone() {
      Some(path) => path,
      None => find_j2534_dll()?,
    };
    let lib = unsafe { J2534Library::new(&dll_path)? };
    self.lib = Some(lib);
    Ok(())
  }

  fn ensure_open(&self) -> Result<(), String> {
    if !self.is_open {
      return Err("Transport not open".to_string());
    }
    Ok(())
  }
}

impl Transport for VLinkerFsJ2534Transport {
  fn open(&mut self) -> Result<(), String> {
    if self.is_open {
      return Ok(());
    }
    self.load_library()?;
    let lib = self.lib.as_ref().ok_or_else(|| "J2534 library not loaded".to_string())?;

    let mut device_id = 0u32;
    let status = unsafe { (lib.open)(std::ptr::null_mut(), &mut device_id) };
    if status != STATUS_NOERROR {
      return Err(format!("PassThruOpen failed: {status}"));
    }

    let mut channel_id = 0u32;
    let status = unsafe { (lib.connect)(device_id, PROTOCOL_CAN, 0, self.baud, &mut channel_id) };
    if status != STATUS_NOERROR {
      unsafe { (lib.close)(device_id) };
      return Err(format!("PassThruConnect failed: {status}"));
    }

    self.device_id = device_id;
    self.channel_id = channel_id;
    self.is_open = true;
    Ok(())
  }

  fn close(&mut self) {
    if let Some(lib) = &self.lib {
      if self.is_open {
        unsafe {
          let _ = (lib.disconnect)(self.channel_id);
          let _ = (lib.close)(self.device_id);
        }
      }
    }
    self.is_open = false;
  }

  fn send(&mut self, frame: &Frame) -> Result<(), String> {
    self.ensure_open()?;
    let lib = self.lib.as_ref().ok_or_else(|| "J2534 library not loaded".to_string())?;

    let mut msg = PassThruMsg::default();
    msg.protocol_id = PROTOCOL_CAN;
    msg.tx_flags = if frame.is_extended { CAN_29BIT_ID } else { 0 };
    msg.data_size = (4 + frame.data.len()) as u32;
    msg.data[0..4].copy_from_slice(&frame.id.to_be_bytes());
    msg.data[4..4 + frame.data.len()].copy_from_slice(&frame.data);

    let mut num = 1u32;
    let status = unsafe { (lib.write_msgs)(self.channel_id, &mut msg, &mut num, 100) };
    if status != STATUS_NOERROR {
      return Err(format!("PassThruWriteMsgs failed: {status}"));
    }
    Ok(())
  }

  fn recv(&mut self, timeout_ms: u64) -> Result<Option<Frame>, String> {
    self.ensure_open()?;
    let lib = self.lib.as_ref().ok_or_else(|| "J2534 library not loaded".to_string())?;

    let mut msg = PassThruMsg::default();
    let mut num = 1u32;
    let status = unsafe { (lib.read_msgs)(self.channel_id, &mut msg, &mut num, timeout_ms as u32) };
    if status == ERR_TIMEOUT || num == 0 {
      return Ok(None);
    }
    if status != STATUS_NOERROR {
      return Err(format!("PassThruReadMsgs failed: {status}"));
    }
    if msg.data_size < 4 {
      return Ok(None);
    }
    let id = u32::from_be_bytes([msg.data[0], msg.data[1], msg.data[2], msg.data[3]]);
    let payload = msg.data[4..msg.data_size as usize].to_vec();
    Ok(Some(Frame {
      id,
      data: payload,
      timestamp_ms: msg.timestamp as u128,
      is_extended: (msg.rx_status & CAN_29BIT_ID) == CAN_29BIT_ID,
    }))
  }

  fn set_filters(&mut self, filters: Vec<Filter>) -> Result<(), String> {
    self.ensure_open()?;
    let lib = self.lib.as_ref().ok_or_else(|| "J2534 library not loaded".to_string())?;

    let Some(start_filter) = lib.start_filter else {
      return Ok(());
    };

    for filter in filters {
      let mut mask = PassThruMsg::default();
      mask.protocol_id = PROTOCOL_CAN;
      mask.data_size = 4;
      mask.data[0..4].copy_from_slice(&filter.mask.to_be_bytes());

      let mut pattern = PassThruMsg::default();
      pattern.protocol_id = PROTOCOL_CAN;
      pattern.data_size = 4;
      pattern.data[0..4].copy_from_slice(&filter.id.to_be_bytes());

      let mut flow = PassThruMsg::default();
      let mut filter_id = 0u32;
      let status = unsafe {
        start_filter(self.channel_id, PASS_FILTER, &mut mask, &mut pattern, &mut flow, &mut filter_id)
      };
      if status != STATUS_NOERROR {
        return Err(format!("PassThruStartMsgFilter failed: {status}"));
      }
    }

    Ok(())
  }

  fn set_baud(&mut self, baud: u32) -> Result<(), String> {
    self.baud = baud;
    Ok(())
  }

  fn set_bus(&mut self, _bus: BusType) -> Result<(), String> {
    Ok(())
  }

  fn set_timing(&mut self, _timing: TimingConfig) -> Result<(), String> {
    Ok(())
  }
}

fn find_j2534_dll() -> Result<PathBuf, String> {
  if let Ok(path) = std::env::var("J2534_DLL") {
    let candidate = PathBuf::from(path);
    if candidate.exists() {
      return Ok(candidate);
    }
  }

  let candidates = vec![
    PathBuf::from("C:\\Program Files (x86)\\vLinker\\J2534.dll"),
    PathBuf::from("C:\\Program Files\\vLinker\\J2534.dll"),
    PathBuf::from("C:\\Windows\\System32\\J2534.dll"),
  ];

  for candidate in candidates {
    if candidate.exists() {
      return Ok(candidate);
    }
  }

  Err("J2534 DLL not found. Install vLinker FS drivers or set J2534_DLL.".to_string())
}
