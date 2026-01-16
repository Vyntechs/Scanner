use crate::app_state::DtcInfo;
use crate::protocol::isotp::IsoTpLink;
use crate::transport::Transport;

pub struct UdsClient<T: Transport> {
  transport: T,
  timeout_ms: u64,
  retries: u8,
}

impl<T: Transport> UdsClient<T> {
  pub fn new(transport: T, timeout_ms: u64, retries: u8) -> Self {
    Self {
      transport,
      timeout_ms,
      retries,
    }
  }

  pub fn open(&mut self) -> Result<(), String> {
    self.transport.open()
  }

  pub fn close(&mut self) {
    self.transport.close();
  }

  pub fn read_vin(&mut self, tx_id: u32, rx_id: u32) -> Result<String, String> {
    let payload = [0x22, 0xF1, 0x90];
    let response = self.request(tx_id, rx_id, &payload)?;
    if response.len() < 3 || response[0] != 0x62 {
      return Err("Unexpected VIN response".to_string());
    }
    let vin_bytes = &response[3..];
    let vin = String::from_utf8_lossy(vin_bytes).trim().to_string();
    Ok(vin)
  }

  pub fn tester_present(&mut self, tx_id: u32, rx_id: u32) -> Result<(), String> {
    let payload = [0x3E, 0x00];
    let response = self.request(tx_id, rx_id, &payload)?;
    if response.is_empty() {
      return Err("No response to tester present".to_string());
    }
    Ok(())
  }

  pub fn read_dtcs(&mut self, tx_id: u32, rx_id: u32) -> Result<Vec<DtcInfo>, String> {
    let payload = [0x19, 0x02, 0xFF];
    let response = self.request(tx_id, rx_id, &payload)?;
    if response.len() < 2 || response[0] != 0x59 {
      return Err("Unexpected DTC response".to_string());
    }
    let mut dtcs = Vec::new();
    let mut index = 2;
    while index + 3 <= response.len() {
      let bytes = [response[index], response[index + 1], response[index + 2]];
      index += 3;
      if bytes == [0x00, 0x00, 0x00] {
        continue;
      }
      let code = decode_dtc(bytes);
      dtcs.push(DtcInfo {
        code,
        description: "DTC description unavailable".to_string(),
        status: "active".to_string(),
      });
    }
    Ok(dtcs)
  }

  pub fn clear_dtcs(&mut self, tx_id: u32, rx_id: u32) -> Result<(), String> {
    let payload = [0x14, 0xFF, 0xFF, 0xFF];
    let response = self.request(tx_id, rx_id, &payload)?;
    if response.is_empty() || response[0] != 0x54 {
      return Err("Clear DTCs failed".to_string());
    }
    Ok(())
  }

  fn request(&mut self, tx_id: u32, rx_id: u32, payload: &[u8]) -> Result<Vec<u8>, String> {
    let mut last_err = None;
    for _ in 0..=self.retries {
      let mut link = IsoTpLink::new(&mut self.transport, tx_id, rx_id, false);
      match link.request(payload, self.timeout_ms) {
        Ok(response) => return Ok(response),
        Err(err) => last_err = Some(err),
      }
    }
    Err(last_err.unwrap_or_else(|| "UDS request failed".to_string()))
  }

  pub fn into_transport(self) -> T {
    self.transport
  }
}

fn decode_dtc(bytes: [u8; 3]) -> String {
  let raw = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | bytes[2] as u32;
  let letter = match (raw >> 22) & 0x3 {
    0 => 'P',
    1 => 'C',
    2 => 'B',
    _ => 'U',
  };
  let code_value = raw & 0x3F_FFFF;
  format!("{}{:06X}", letter, code_value)
}
