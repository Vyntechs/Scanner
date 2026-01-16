use std::time::{Duration, Instant};

use crate::transport::{Frame, Transport};

pub struct IsoTpLink<'a, T: Transport> {
  transport: &'a mut T,
  tx_id: u32,
  rx_id: u32,
  is_extended: bool,
}

impl<'a, T: Transport> IsoTpLink<'a, T> {
  pub fn new(transport: &'a mut T, tx_id: u32, rx_id: u32, is_extended: bool) -> Self {
    Self {
      transport,
      tx_id,
      rx_id,
      is_extended,
    }
  }

  pub fn request(&mut self, payload: &[u8], timeout_ms: u64) -> Result<Vec<u8>, String> {
    self.send_payload(payload)?;
    self.recv_payload(timeout_ms)
  }

  fn send_payload(&mut self, payload: &[u8]) -> Result<(), String> {
    if payload.len() <= 7 {
      let mut data = vec![0u8; 8];
      data[0] = payload.len() as u8;
      data[1..1 + payload.len()].copy_from_slice(payload);
      let frame = Frame {
        id: self.tx_id,
        data,
        timestamp_ms: 0,
        is_extended: self.is_extended,
      };
      return self.transport.send(&frame);
    }

    let total_len = payload.len();
    if total_len > 4095 {
      return Err("ISO-TP payload too large".to_string());
    }

    let mut data = vec![0u8; 8];
    data[0] = 0x10 | ((total_len >> 8) as u8 & 0x0F);
    data[1] = (total_len & 0xFF) as u8;
    data[2..8].copy_from_slice(&payload[0..6]);

    let frame = Frame {
      id: self.tx_id,
      data,
      timestamp_ms: 0,
      is_extended: self.is_extended,
    };
    self.transport.send(&frame)?;

    let mut offset = 6;
    let mut seq = 1u8;
    while offset < total_len {
      let chunk_len = usize::min(7, total_len - offset);
      let mut cf = vec![0u8; 8];
      cf[0] = 0x20 | (seq & 0x0F);
      cf[1..1 + chunk_len].copy_from_slice(&payload[offset..offset + chunk_len]);
      let frame = Frame {
        id: self.tx_id,
        data: cf,
        timestamp_ms: 0,
        is_extended: self.is_extended,
      };
      self.transport.send(&frame)?;
      offset += chunk_len;
      seq = seq.wrapping_add(1);
    }

    Ok(())
  }

  fn recv_payload(&mut self, timeout_ms: u64) -> Result<Vec<u8>, String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let mut buffer: Vec<u8> = Vec::new();
    let mut total_len: Option<usize> = None;

    while Instant::now() < deadline {
      let remaining = deadline.saturating_duration_since(Instant::now());
      let timeout = remaining.as_millis().clamp(10, 250) as u64;
      let frame = match self.transport.recv(timeout)? {
        Some(frame) => frame,
        None => continue,
      };

      if frame.id != self.rx_id {
        continue;
      }

      if frame.data.is_empty() {
        continue;
      }

      let pci = frame.data[0] >> 4;
      match pci {
        0x0 => {
          let len = (frame.data[0] & 0x0F) as usize;
          let end = usize::min(1 + len, frame.data.len());
          return Ok(frame.data[1..end].to_vec());
        }
        0x1 => {
          let len = (((frame.data[0] as usize) & 0x0F) << 8) | frame.data[1] as usize;
          total_len = Some(len);
          buffer.extend_from_slice(&frame.data[2..8]);

          let mut fc = vec![0u8; 8];
          fc[0] = 0x30;
          fc[1] = 0x00;
          fc[2] = 0x00;
          let flow = Frame {
            id: self.tx_id,
            data: fc,
            timestamp_ms: 0,
            is_extended: self.is_extended,
          };
          self.transport.send(&flow)?;
        }
        0x2 => {
          if total_len.is_none() {
            continue;
          }
          buffer.extend_from_slice(&frame.data[1..8]);
          if let Some(len) = total_len {
            if buffer.len() >= len {
              buffer.truncate(len);
              return Ok(buffer);
            }
          }
        }
        _ => continue,
      }
    }

    Err("ISO-TP timeout waiting for response".to_string())
  }
}
