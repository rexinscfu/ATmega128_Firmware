//! TWI (I2C) HAL implementation
#![no_std]

use avr_device::atmega128::TWI;
use core::marker::PhantomData;

/// TWI speed modes
#[derive(Clone, Copy)]
pub enum TwiSpeed {
    Standard100k,
    Fast400k,
}

/// TWI status codes
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TwiStatus {
    StartTransmitted = 0x08,
    RepStartTransmitted = 0x10,
    AddrWriteAck = 0x18,
    AddrWriteNack = 0x20,
    DataWriteAck = 0x28,
    DataWriteNack = 0x30,
    ArbitrationLost = 0x38,
    AddrReadAck = 0x40,
    AddrReadNack = 0x48,
    DataReadAck = 0x50,
    DataReadNack = 0x58,
}

/// TWI peripheral driver
pub struct Twi {
    _twi: PhantomData<TWI>,
}

impl Twi {
    /// Create new TWI instance
    pub fn new() -> Self {
        unsafe {
            let p = TWI::ptr();
            
            // Enable TWI with internal pullups
            (*p).twcr.write(|w| w.bits(0x44));
            
            // Default to 100kHz with 16MHz CPU clock
            (*p).twbr.write(|w| w.bits(72));
            (*p).twsr.write(|w| w.bits(0));
        }
        
        Self { _twi: PhantomData }
    }

    /// Set TWI speed
    pub fn set_speed(&mut self, speed: TwiSpeed) {
        unsafe {
            let p = TWI::ptr();
            match speed {
                TwiSpeed::Standard100k => {
                    (*p).twbr.write(|w| w.bits(72)); // 100kHz @ 16MHz
                    (*p).twsr.write(|w| w.bits(0));
                }
                TwiSpeed::Fast400k => {
                    (*p).twbr.write(|w| w.bits(12)); // 400kHz @ 16MHz
                    (*p).twsr.write(|w| w.bits(0));
                }
            }
        }
    }

    /// Start TWI transmission
    pub fn start(&mut self) -> Result<(), ()> {
        unsafe {
            let p = TWI::ptr();
            
            // Send START condition
            (*p).twcr.write(|w| w.bits(0xA4));
            
            // Wait for completion
            while (*p).twcr.read().bits() & 0x80 == 0 {}
            
            // Check status
            if ((*p).twsr.read().bits() & 0xF8) == TwiStatus::StartTransmitted as u8 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    /// Stop TWI transmission
    pub fn stop(&mut self) {
        unsafe {
            let p = TWI::ptr();
            (*p).twcr.write(|w| w.bits(0x94));
            while (*p).twcr.read().bits() & 0x10 != 0 {}
        }
    }

    /// Write address + R/W bit
    pub fn write_address(&mut self, addr: u8, read: bool) -> Result<(), ()> {
        let addr = (addr << 1) | (read as u8);
        self.write_byte(addr)
    }

    /// Write a single byte
    pub fn write_byte(&mut self, byte: u8) -> Result<(), ()> {
        unsafe {
            let p = TWI::ptr();
            
            // Load data and start transmission
            (*p).twdr.write(|w| w.bits(byte));
            (*p).twcr.write(|w| w.bits(0x84));
            
            // Wait for completion
            while (*p).twcr.read().bits() & 0x80 == 0 {}
            
            // Check status
            let status = (*p).twsr.read().bits() & 0xF8;
            if status == TwiStatus::DataWriteAck as u8 || 
               status == TwiStatus::AddrWriteAck as u8 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    /// Read a byte and send ACK/NACK
    pub fn read_byte(&mut self, ack: bool) -> Result<u8, ()> {
        unsafe {
            let p = TWI::ptr();
            
            // Start read with ACK/NACK
            (*p).twcr.write(|w| w.bits(if ack { 0xC4 } else { 0x84 }));
            
            // Wait for completion
            while (*p).twcr.read().bits() & 0x80 == 0 {}
            
            // Check status
            let status = (*p).twsr.read().bits() & 0xF8;
            if status == TwiStatus::DataReadAck as u8 || 
               status == TwiStatus::DataReadNack as u8 {
                Ok((*p).twdr.read().bits())
            } else {
                Err(())
            }
        }
    }
}

impl Default for Twi {
    fn default() -> Self {
        Self::new()
    }
}
