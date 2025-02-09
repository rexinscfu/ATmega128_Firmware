//! Communication protocol stack implementation
#![no_std]

pub mod packet;
pub mod transport;
pub mod crc;

use crate::hal::uart::Uart;

#[derive(Debug)]
pub enum ProtocolError {
    BufferOverflow,
    InvalidChecksum,
    InvalidPacket,
    InvalidCommand,
    Timeout,
    TransportError,
}

pub type Result<T> = core::result::Result<T, ProtocolError>;

#[derive(Clone, Copy)]
pub enum Command {
    Ping = 0x01,
    GetStatus = 0x02,
    SetConfig = 0x03,
    GetData = 0x04,
    Reset = 0x05,
    UpdateFirmware = 0x06,
    Debug = 0x07,
}

pub struct Protocol {
    uart: Uart,
    rx_buffer: [u8; 256],
    tx_buffer: [u8; 256],
    rx_index: usize,
    packet_handler: Option<fn(&[u8]) -> Result<()>>,
}

/*
struct ProtocolStats {
    packets_received: u32,
    packets_sent: u32,
    bytes_received: u32,
    bytes_sent: u32,
    checksum_errors: u32,
    buffer_overflows: u32,
    timeouts: u32,
}

struct ProtocolConfig {
    baudrate: u32,
    parity: bool,
    stop_bits: u8,
    flow_control: bool,
    timeout_ms: u16,
    retry_count: u8,
}
*/

impl Protocol {
    pub fn new(uart: Uart) -> Self {
        Self {
            uart,
            rx_buffer: [0; 256],
            tx_buffer: [0; 256],
            rx_index: 0,
            packet_handler: None,
        }
    }

    pub fn set_packet_handler(&mut self, handler: fn(&[u8]) -> Result<()>) {
        self.packet_handler = Some(handler);
    }

    pub fn process(&mut self) -> Result<()> {
        while let Some(byte) = self.uart.read_byte() {
            if self.rx_index >= self.rx_buffer.len() {
                self.rx_index = 0;
                return Err(ProtocolError::BufferOverflow);
            }
            
            self.rx_buffer[self.rx_index] = byte;
            self.rx_index += 1;
            
            if byte == 0x0A {
                if let Some(handler) = self.packet_handler {
                    handler(&self.rx_buffer[..self.rx_index])?;
                }
                self.rx_index = 0;
            }
        }
        Ok(())
    }

    pub fn send_packet(&mut self, command: Command, data: &[u8]) -> Result<()> {
        if data.len() > 250 {
            return Err(ProtocolError::BufferOverflow);
        }
        
        self.tx_buffer[0] = 0x55;  // Start byte
        self.tx_buffer[1] = 0xAA;  // Start byte
        self.tx_buffer[2] = command as u8;
        self.tx_buffer[3] = data.len() as u8;
        
        self.tx_buffer[4..4+data.len()].copy_from_slice(data);
        
        let checksum = self.calculate_checksum(&self.tx_buffer[..4+data.len()]);
        self.tx_buffer[4+data.len()] = checksum;
        self.tx_buffer[5+data.len()] = 0x0A;  // End byte
        
        for &byte in &self.tx_buffer[..6+data.len()] {
            self.uart.write_byte(byte);
        }
        
        Ok(())
    }

    pub fn send_ping(&mut self) -> Result<()> {
        self.send_packet(Command::Ping, &[])
    }

    pub fn send_status(&mut self, status: u8) -> Result<()> {
        self.send_packet(Command::GetStatus, &[status])
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.send_packet(Command::GetData, data)
    }

    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        let mut sum: u8 = 0;
        for &byte in data {
            sum = sum.wrapping_add(byte);
        }
        !sum
    }

    fn verify_checksum(&self, data: &[u8], checksum: u8) -> bool {
        self.calculate_checksum(data) == checksum
    }
}
