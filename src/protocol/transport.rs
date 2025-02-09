//! Transport layer implementation
#![no_std]

use super::{Result, ProtocolError};
use crate::hal::uart::Uart;

const RX_BUFFER_SIZE: usize = 512;
const TX_BUFFER_SIZE: usize = 512;

pub struct Transport {
    uart: Uart,
    rx_buffer: [u8; RX_BUFFER_SIZE],
    tx_buffer: [u8; TX_BUFFER_SIZE],
    rx_head: usize,
    rx_tail: usize,
    tx_head: usize,
    tx_tail: usize,
}

/*
struct TransportConfig {
    flow_control: bool,
    rts_threshold: u16,
    cts_threshold: u16,
    timeout_ms: u16,
}

struct TransportStats {
    rx_overruns: u32,
    tx_overruns: u32,
    frame_errors: u32,
    parity_errors: u32,
}
*/

impl Transport {
    pub fn new(uart: Uart) -> Self {
        Self {
            uart,
            rx_buffer: [0; RX_BUFFER_SIZE],
            tx_buffer: [0; TX_BUFFER_SIZE],
            rx_head: 0,
            rx_tail: 0,
            tx_head: 0,
            tx_tail: 0,
        }
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut count = 0;
        while count < buffer.len() && self.rx_head != self.rx_tail {
            buffer[count] = self.rx_buffer[self.rx_tail];
            self.rx_tail = (self.rx_tail + 1) % RX_BUFFER_SIZE;
            count += 1;
        }
        Ok(count)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        let mut count = 0;
        for &byte in data {
            let next_head = (self.tx_head + 1) % TX_BUFFER_SIZE;
            if next_head == self.tx_tail {
                return Err(ProtocolError::BufferOverflow);
            }
            self.tx_buffer[self.tx_head] = byte;
            self.tx_head = next_head;
            count += 1;
        }
        self.flush_tx()?;
        Ok(count)
    }

    pub fn process(&mut self) -> Result<()> {
        self.process_rx()?;
        self.process_tx()?;
        Ok(())
    }

    fn process_rx(&mut self) -> Result<()> {
        while let Some(byte) = self.uart.read_byte() {
            let next_head = (self.rx_head + 1) % RX_BUFFER_SIZE;
            if next_head == self.rx_tail {
                return Err(ProtocolError::BufferOverflow);
            }
            self.rx_buffer[self.rx_head] = byte;
            self.rx_head = next_head;
        }
        Ok(())
    }

    fn process_tx(&mut self) -> Result<()> {
        self.flush_tx()
    }

    fn flush_tx(&mut self) -> Result<()> {
        while self.tx_tail != self.tx_head {
            if !self.uart.is_tx_ready() {
                break;
            }
            self.uart.write_byte(self.tx_buffer[self.tx_tail]);
            self.tx_tail = (self.tx_tail + 1) % TX_BUFFER_SIZE;
        }
        Ok(())
    }

    pub fn bytes_available(&self) -> usize {
        if self.rx_head >= self.rx_tail {
            self.rx_head - self.rx_tail
        } else {
            RX_BUFFER_SIZE - (self.rx_tail - self.rx_head)
        }
    }

    pub fn space_available(&self) -> usize {
        if self.tx_tail > self.tx_head {
            self.tx_tail - self.tx_head - 1
        } else {
            TX_BUFFER_SIZE - (self.tx_head - self.tx_tail) - 1
        }
    }
}
