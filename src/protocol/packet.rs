//! Packet handling implementation
#![no_std]

use super::{Command, Result, ProtocolError};

const MAX_PACKET_SIZE: usize = 256;
const HEADER_SIZE: usize = 4;
const FOOTER_SIZE: usize = 2;

pub struct Packet {
    buffer: [u8; MAX_PACKET_SIZE],
    length: usize,
}

/*
struct PacketHeader {
    start_bytes: [u8; 2],
    command: u8,
    length: u8,
}

struct PacketFooter {
    checksum: u8,
    end_byte: u8,
}
*/

impl Packet {
    pub fn new() -> Self {
        Self {
            buffer: [0; MAX_PACKET_SIZE],
            length: 0,
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<Command> {
        if data.len() < HEADER_SIZE + FOOTER_SIZE {
            return Err(ProtocolError::InvalidPacket);
        }

        if data[0] != 0x55 || data[1] != 0xAA {
            return Err(ProtocolError::InvalidPacket);
        }

        let command = data[2];
        let length = data[3] as usize;
        
        if data.len() != HEADER_SIZE + length + FOOTER_SIZE {
            return Err(ProtocolError::InvalidPacket);
        }

        let checksum = data[HEADER_SIZE + length];
        let end_byte = data[HEADER_SIZE + length + 1];

        if end_byte != 0x0A {
            return Err(ProtocolError::InvalidPacket);
        }

        let calc_checksum = self.calculate_checksum(&data[..HEADER_SIZE + length]);
        if checksum != calc_checksum {
            return Err(ProtocolError::InvalidChecksum);
        }

        self.buffer[..data.len()].copy_from_slice(data);
        self.length = data.len();

        match command {
            0x01 => Ok(Command::Ping),
            0x02 => Ok(Command::GetStatus),
            0x03 => Ok(Command::SetConfig),
            0x04 => Ok(Command::GetData),
            0x05 => Ok(Command::Reset),
            0x06 => Ok(Command::UpdateFirmware),
            0x07 => Ok(Command::Debug),
            _ => Err(ProtocolError::InvalidCommand),
        }
    }

    pub fn get_data(&self) -> &[u8] {
        if self.length < HEADER_SIZE + FOOTER_SIZE {
            return &[];
        }
        &self.buffer[HEADER_SIZE..self.length - FOOTER_SIZE]
    }

    pub fn create(&mut self, command: Command, data: &[u8]) -> Result<&[u8]> {
        if data.len() > MAX_PACKET_SIZE - HEADER_SIZE - FOOTER_SIZE {
            return Err(ProtocolError::BufferOverflow);
        }

        self.buffer[0] = 0x55;
        self.buffer[1] = 0xAA;
        self.buffer[2] = command as u8;
        self.buffer[3] = data.len() as u8;

        self.buffer[HEADER_SIZE..HEADER_SIZE + data.len()].copy_from_slice(data);

        let checksum = self.calculate_checksum(&self.buffer[..HEADER_SIZE + data.len()]);
        self.buffer[HEADER_SIZE + data.len()] = checksum;
        self.buffer[HEADER_SIZE + data.len() + 1] = 0x0A;

        self.length = HEADER_SIZE + data.len() + FOOTER_SIZE;
        Ok(&self.buffer[..self.length])
    }

    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        let mut sum: u8 = 0;
        for &byte in data {
            sum = sum.wrapping_add(byte);
        }
        !sum
    }
}
