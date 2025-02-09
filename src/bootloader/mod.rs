#![no_std]

use crate::hal::{flash::Flash, uart::Uart};

const BOOTLOADER_START: u32 = 0x1E000;
const PAGE_SIZE: usize = 256;
const MAGIC_WORD: u32 = 0xB007F11E;

pub struct Bootloader {
    flash: Flash,
    uart: Uart,
    state: BootloaderState,
}

#[derive(PartialEq)]
enum BootloaderState {
    Idle,
    Receiving,
    Programming,
    Verifying,
}

pub struct FirmwareHeader {
    magic: u32,
    version: u32,
    size: u32,
    crc: u32,
}

impl Bootloader {
    pub fn new(flash: Flash, uart: Uart) -> Self {
        Self {
            flash,
            uart,
            state: BootloaderState::Idle,
        }
    }

    pub fn enter_bootloader(&mut self) {
        unsafe {
            let mcucr = &(*avr_device::atmega128::CPU::ptr()).mcucr;
            mcucr.write(|w| w.bits(0x01));
            mcucr.write(|w| w.bits(0x00));
        }
        self.state = BootloaderState::Idle;
    }

    pub fn process(&mut self) -> Result<(), ()> {
        match self.state {
            BootloaderState::Idle => self.handle_idle(),
            BootloaderState::Receiving => self.handle_receiving(),
            BootloaderState::Programming => self.handle_programming(),
            BootloaderState::Verifying => self.handle_verifying(),
        }
    }

    fn handle_idle(&mut self) -> Result<(), ()> {
        if let Some(byte) = self.uart.read_byte() {
            if byte == 0x7F {
                self.uart.write_byte(0x55);
                self.state = BootloaderState::Receiving;
            }
        }
        Ok(())
    }

    fn handle_receiving(&mut self) -> Result<(), ()> {
        let mut header = [0u8; core::mem::size_of::<FirmwareHeader>()];
        let mut received = 0;

        while received < header.len() {
            if let Some(byte) = self.uart.read_byte() {
                header[received] = byte;
                received += 1;
            }
        }

        let header = unsafe { core::ptr::read(header.as_ptr() as *const FirmwareHeader) };
        
        if header.magic != MAGIC_WORD {
            self.uart.write_byte(0x45);
            self.state = BootloaderState::Idle;
            return Err(());
        }

        if header.size > (BOOTLOADER_START - 0x1000) {
            self.uart.write_byte(0x46);
            self.state = BootloaderState::Idle;
            return Err(());
        }

        self.uart.write_byte(0xAA);
        self.state = BootloaderState::Programming;
        Ok(())
    }

    fn handle_programming(&mut self) -> Result<(), ()> {
        let mut page_buffer = [0u8; PAGE_SIZE];
        let mut address = 0;

        while address < BOOTLOADER_START {
            let mut received = 0;
            while received < PAGE_SIZE {
                if let Some(byte) = self.uart.read_byte() {
                    page_buffer[received] = byte;
                    received += 1;
                }
            }

            self.flash.erase_page(address)?;
            self.flash.write_page(address, &page_buffer)?;

            let mut verify_buffer = [0u8; PAGE_SIZE];
            self.flash.read(address, &mut verify_buffer)?;

            if verify_buffer != page_buffer {
                self.uart.write_byte(0x47);
                self.state = BootloaderState::Idle;
                return Err(());
            }

            self.uart.write_byte(0xAC);
            address += PAGE_SIZE as u32;
        }

        self.state = BootloaderState::Verifying;
        Ok(())
    }

    fn handle_verifying(&mut self) -> Result<(), ()> {
        let mut crc = 0u32;
        let mut buffer = [0u8; PAGE_SIZE];
        let mut address = 0;

        while address < BOOTLOADER_START {
            self.flash.read(address, &mut buffer)?;
            crc = self.calculate_crc32(&buffer, crc);
            address += PAGE_SIZE as u32;
        }

        self.uart.write_byte(0xCC);
        self.uart.write_byte((crc >> 24) as u8);
        self.uart.write_byte((crc >> 16) as u8);
        self.uart.write_byte((crc >> 8) as u8);
        self.uart.write_byte(crc as u8);

        self.state = BootloaderState::Idle;
        Ok(())
    }

    fn calculate_crc32(&self, data: &[u8], mut crc: u32) -> u32 {
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 == 1 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }

    pub fn jump_to_application(&mut self) {
        unsafe {
            core::arch::asm!(
                "jmp 0",
                options(noreturn)
            );
        }
    }
}
