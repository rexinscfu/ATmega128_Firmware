//! SPI (Serial Peripheral Interface) HAL implementation
#![no_std]

use avr_device::atmega128::SPI;
use core::marker::PhantomData;

/// SPI clock prescaler options
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SpiPrescaler {
    Div4 = 0,
    Div16 = 1,
    Div64 = 2,
    Div128 = 3,
}

/// SPI data order
#[derive(Clone, Copy)]
pub enum DataOrder {
    MsbFirst,
    LsbFirst,
}

/// SPI mode configurations
#[derive(Clone, Copy)]
pub enum SpiMode {
    Mode0, // CPOL=0, CPHA=0
    Mode1, // CPOL=0, CPHA=1
    Mode2, // CPOL=1, CPHA=0
    Mode3, // CPOL=1, CPHA=1
}

/// SPI peripheral driver
pub struct Spi {
    _spi: PhantomData<SPI>,
}

impl Spi {
    /// Create new SPI instance
    pub fn new() -> Self {
        unsafe {
            let p = SPI::ptr();
            
            // Enable SPI, Master mode
            (*p).spcr.write(|w| w.bits(0x50));
            
            // Default: Mode0, MSB first, Fosc/4
            (*p).spcr.modify(|_, w| {
                w.mstr().set_bit()
                 .spe().set_bit()
            });
        }
        
        Self { _spi: PhantomData }
    }

    /// Configure SPI mode
    pub fn set_mode(&mut self, mode: SpiMode) {
        unsafe {
            let p = SPI::ptr();
            (*p).spcr.modify(|r, w| {
                let bits = r.bits();
                let bits = match mode {
                    SpiMode::Mode0 => bits & !0x0C,
                    SpiMode::Mode1 => (bits & !0x0C) | 0x04,
                    SpiMode::Mode2 => (bits & !0x0C) | 0x08,
                    SpiMode::Mode3 => bits | 0x0C,
                };
                w.bits(bits)
            });
        }
    }

    /// Set clock prescaler
    pub fn set_clock(&mut self, prescaler: SpiPrescaler) {
        unsafe {
            let p = SPI::ptr();
            (*p).spcr.modify(|r, w| {
                w.bits((r.bits() & !0x03) | (prescaler as u8))
            });
        }
    }

    /// Set data order (MSB/LSB first)
    pub fn set_data_order(&mut self, order: DataOrder) {
        unsafe {
            let p = SPI::ptr();
            match order {
                DataOrder::MsbFirst => (*p).spcr.modify(|r, w| w.bits(r.bits() & !0x20)),
                DataOrder::LsbFirst => (*p).spcr.modify(|r, w| w.bits(r.bits() | 0x20)),
            }
        }
    }

    /// Transfer a single byte
    pub fn transfer(&mut self, byte: u8) -> u8 {
        unsafe {
            let p = SPI::ptr();
            
            // Start transmission
            (*p).spdr.write(|w| w.bits(byte));
            
            // Wait for transmission complete
            while (*p).spsr.read().bits() & 0x80 == 0 {}
            
            // Read received byte
            (*p).spdr.read().bits()
        }
    }

    /// Transfer multiple bytes
    pub fn transfer_bytes(&mut self, data: &[u8], buffer: &mut [u8]) {
        for i in 0..data.len().min(buffer.len()) {
            buffer[i] = self.transfer(data[i]);
        }
    }
}

impl Default for Spi {
    fn default() -> Self {
        Self::new()
    }
}
