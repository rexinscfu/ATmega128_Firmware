//! External Flash Memory Driver (W25Q128)
#![no_std]

use crate::hal::spi::{Spi, SpiMode};

const WRITE_ENABLE: u8 = 0x06;
const WRITE_DISABLE: u8 = 0x04;
const READ_STATUS: u8 = 0x05;
const WRITE_STATUS: u8 = 0x01;
const READ_DATA: u8 = 0x03;
const PAGE_PROGRAM: u8 = 0x02;
const SECTOR_ERASE: u8 = 0x20;
const BLOCK_ERASE_32K: u8 = 0x52;
const BLOCK_ERASE_64K: u8 = 0xD8;
const CHIP_ERASE: u8 = 0xC7;
const POWER_DOWN: u8 = 0xB9;
const RELEASE_POWER_DOWN: u8 = 0xAB;
const DEVICE_ID: u8 = 0x90;
const JEDEC_ID: u8 = 0x9F;

const PAGE_SIZE: usize = 256;
const SECTOR_SIZE: usize = 4096;
const BLOCK_SIZE_32K: usize = 32768;
const BLOCK_SIZE_64K: usize = 65536;

pub struct Flash {
    spi: Spi,
    cs_pin: u8,
    wp_pin: u8,
    hold_pin: u8,
}

#[derive(Debug)]
pub enum FlashError {
    WriteError,
    ReadError,
    EraseError,
    TimeoutError,
    WrongId,
}

/*
struct FlashConfig {
    quad_enable: bool,
    dummy_cycles: u8,
    address_mode: AddressMode,
    clock_mode: ClockMode,
}

struct FlashStatus {
    busy: bool,
    write_enabled: bool,
    block_protected: bool,
    power_down: bool,
    quad_enabled: bool,
}
*/

impl Flash {
    pub fn new(spi: Spi, cs_pin: u8, wp_pin: u8, hold_pin: u8) -> Result<Self, FlashError> {
        let mut flash = Self {
            spi,
            cs_pin,
            wp_pin,
            hold_pin,
        };
        
        flash.init()?;
        Ok(flash)
    }

    fn init(&mut self) -> Result<(), FlashError> {
        self.spi.set_mode(SpiMode::Mode0);
        self.set_pin_high(self.cs_pin);
        self.set_pin_high(self.wp_pin);
        self.set_pin_high(self.hold_pin);
        
        let id = self.read_jedec_id()?;
        if id[0] != 0xEF || id[1] != 0x40 || id[2] != 0x18 {
            return Err(FlashError::WrongId);
        }
        
        Ok(())
    }

    pub fn read(&mut self, addr: u32, buffer: &mut [u8]) -> Result<(), FlashError> {
        self.wait_busy()?;
        self.set_pin_low(self.cs_pin);
        
        self.spi.transfer(READ_DATA);
        self.spi.transfer((addr >> 16) as u8);
        self.spi.transfer((addr >> 8) as u8);
        self.spi.transfer(addr as u8);
        
        for byte in buffer.iter_mut() {
            *byte = self.spi.transfer(0x00);
        }
        
        self.set_pin_high(self.cs_pin);
        Ok(())
    }

    pub fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), FlashError> {
        for (i, chunk) in data.chunks(PAGE_SIZE).enumerate() {
            let page_addr = addr + (i * PAGE_SIZE) as u32;
            self.write_page(page_addr, chunk)?;
        }
        Ok(())
    }

    pub fn erase_sector(&mut self, addr: u32) -> Result<(), FlashError> {
        self.wait_busy()?;
        self.write_enable()?;
        
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(SECTOR_ERASE);
        self.spi.transfer((addr >> 16) as u8);
        self.spi.transfer((addr >> 8) as u8);
        self.spi.transfer(addr as u8);
        self.set_pin_high(self.cs_pin);
        
        self.wait_busy()?;
        Ok(())
    }

    pub fn erase_block32k(&mut self, addr: u32) -> Result<(), FlashError> {
        self.wait_busy()?;
        self.write_enable()?;
        
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(BLOCK_ERASE_32K);
        self.spi.transfer((addr >> 16) as u8);
        self.spi.transfer((addr >> 8) as u8);
        self.spi.transfer(addr as u8);
        self.set_pin_high(self.cs_pin);
        
        self.wait_busy()?;
        Ok(())
    }

    pub fn erase_block64k(&mut self, addr: u32) -> Result<(), FlashError> {
        self.wait_busy()?;
        self.write_enable()?;
        
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(BLOCK_ERASE_64K);
        self.spi.transfer((addr >> 16) as u8);
        self.spi.transfer((addr >> 8) as u8);
        self.spi.transfer(addr as u8);
        self.set_pin_high(self.cs_pin);
        
        self.wait_busy()?;
        Ok(())
    }

    pub fn erase_chip(&mut self) -> Result<(), FlashError> {
        self.wait_busy()?;
        self.write_enable()?;
        
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(CHIP_ERASE);
        self.set_pin_high(self.cs_pin);
        
        self.wait_busy()?;
        Ok(())
    }

    pub fn power_down(&mut self) -> Result<(), FlashError> {
        self.wait_busy()?;
        
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(POWER_DOWN);
        self.set_pin_high(self.cs_pin);
        
        Ok(())
    }

    pub fn release_power_down(&mut self) -> Result<(), FlashError> {
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(RELEASE_POWER_DOWN);
        self.set_pin_high(self.cs_pin);
        
        Ok(())
    }

    fn write_page(&mut self, addr: u32, data: &[u8]) -> Result<(), FlashError> {
        self.wait_busy()?;
        self.write_enable()?;
        
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(PAGE_PROGRAM);
        self.spi.transfer((addr >> 16) as u8);
        self.spi.transfer((addr >> 8) as u8);
        self.spi.transfer(addr as u8);
        
        for &byte in data {
            self.spi.transfer(byte);
        }
        
        self.set_pin_high(self.cs_pin);
        self.wait_busy()?;
        Ok(())
    }

    fn write_enable(&mut self) -> Result<(), FlashError> {
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(WRITE_ENABLE);
        self.set_pin_high(self.cs_pin);
        Ok(())
    }

    fn read_status(&mut self) -> Result<u8, FlashError> {
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(READ_STATUS);
        let status = self.spi.transfer(0x00);
        self.set_pin_high(self.cs_pin);
        Ok(status)
    }

    fn wait_busy(&mut self) -> Result<(), FlashError> {
        let mut timeout = 0;
        while (self.read_status()? & 0x01) != 0 {
            timeout += 1;
            if timeout > 10000 {
                return Err(FlashError::TimeoutError);
            }
        }
        Ok(())
    }

    fn read_jedec_id(&mut self) -> Result<[u8; 3], FlashError> {
        let mut id = [0u8; 3];
        self.set_pin_low(self.cs_pin);
        self.spi.transfer(JEDEC_ID);
        id[0] = self.spi.transfer(0x00);
        id[1] = self.spi.transfer(0x00);
        id[2] = self.spi.transfer(0x00);
        self.set_pin_high(self.cs_pin);
        Ok(id)
    }

    fn set_pin_high(&mut self, pin: u8) {
        unsafe {
            (*avr_device::atmega128::PORTB::ptr()).portb.modify(|r, w| 
                w.bits(r.bits() | (1 << pin))
            );
        }
    }

    fn set_pin_low(&mut self, pin: u8) {
        unsafe {
            (*avr_device::atmega128::PORTB::ptr()).portb.modify(|r, w| 
                w.bits(r.bits() & !(1 << pin))
            );
        }
    }
}
