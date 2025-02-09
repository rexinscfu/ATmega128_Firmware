//! Error handling and diagnostics system
#![no_std]

use crate::logger::Logger;
use core::sync::atomic::{AtomicU32, Ordering};

static ERROR_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    HardwareFault = 0x1000,
    SensorError = 0x2000,
    CommunicationError = 0x3000,
    CalibrationError = 0x4000,
    TimingError = 0x5000,
    MemoryError = 0x6000,
    PowerError = 0x7000,
    SystemError = 0x8000,
}

#[derive(Debug, Clone, Copy)]
pub struct Error {
    code: ErrorCode,
    subcode: u16,
    timestamp: u32,
    data: u32,
}

pub struct Diagnostics {
    logger: Logger,
    last_error: Option<Error>,
    watchdog_enabled: bool,
}

impl Diagnostics {
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            last_error: None,
            watchdog_enabled: false,
        }
    }

    pub fn report_error(&mut self, code: ErrorCode, subcode: u16, data: u32) {
        let error = Error {
            code,
            subcode,
            timestamp: self.get_timestamp(),
            data,
        };

        self.last_error = Some(error);
        ERROR_COUNT.fetch_add(1, Ordering::Relaxed);

        let mut error_data = [0u8; 16];
        error_data[0..4].copy_from_slice(&(error.code as u32).to_le_bytes());
        error_data[4..6].copy_from_slice(&error.subcode.to_le_bytes());
        error_data[6..10].copy_from_slice(&error.timestamp.to_le_bytes());
        error_data[10..14].copy_from_slice(&error.data.to_le_bytes());

        self.logger.log_error(&error_data).ok();
        self.handle_error(&error);
    }

    pub fn get_last_error(&self) -> Option<Error> {
        self.last_error
    }

    pub fn get_error_count(&self) -> u32 {
        ERROR_COUNT.load(Ordering::Relaxed)
    }

    pub fn enable_watchdog(&mut self) {
        unsafe {
            let wdt = &(*avr_device::atmega128::WDT::ptr());
            wdt.wdtcr.write(|w| w.bits(0x18));
            wdt.wdtcr.write(|w| w.bits(0x0E));
        }
        self.watchdog_enabled = true;
    }

    pub fn disable_watchdog(&mut self) {
        unsafe {
            let wdt = &(*avr_device::atmega128::WDT::ptr());
            wdt.wdtcr.write(|w| w.bits(0x18));
            wdt.wdtcr.write(|w| w.bits(0x00));
        }
        self.watchdog_enabled = false;
    }

    pub fn reset_watchdog(&self) {
        if self.watchdog_enabled {
            unsafe {
                avr_device::asm::wdr();
            }
        }
    }

    pub fn run_diagnostics(&mut self) -> Result<(), Error> {
        self.check_voltage()?;
        self.check_temperature()?;
        self.check_memory()?;
        self.check_peripherals()?;
        Ok(())
    }

    fn handle_error(&mut self, error: &Error) {
        match error.code {
            ErrorCode::HardwareFault => self.handle_hardware_fault(error),
            ErrorCode::PowerError => self.handle_power_error(error),
            ErrorCode::SystemError => self.handle_system_error(error),
            _ => {}
        }
    }

    fn handle_hardware_fault(&mut self, error: &Error) {
        if error.subcode & 0xFF00 == 0x0100 {
            self.emergency_shutdown();
        }
    }

    fn handle_power_error(&mut self, error: &Error) {
        if error.subcode & 0xFF00 == 0x0100 {
            self.enter_low_power_mode();
        }
    }

    fn handle_system_error(&mut self, _error: &Error) {
        self.reset_system();
    }

    fn check_voltage(&self) -> Result<(), Error> {
        unsafe {
            let adc = &(*avr_device::atmega128::ADC::ptr());
            adc.admux.write(|w| w.bits(0x40));
            adc.adcsra.write(|w| w.bits(0x87));
            while adc.adcsra.read().bits() & 0x10 == 0 {}
            let value = adc.adcl.read().bits() as u16 | ((adc.adch.read().bits() as u16) << 8);
            
            if value < 300 {
                return Err(Error {
                    code: ErrorCode::PowerError,
                    subcode: 0x0101,
                    timestamp: self.get_timestamp(),
                    data: value as u32,
                });
            }
        }
        Ok(())
    }

    fn check_temperature(&self) -> Result<(), Error> {
        let mut lm75_temp: i16 = 0;
        unsafe {
            let twi = &(*avr_device::atmega128::TWI::ptr());
            
            // Start + SLA+W
            twi.twcr.write(|w| w.bits(0xA4));
            while twi.twcr.read().bits() & 0x80 == 0 {}
            if twi.twsr.read().bits() & 0xF8 != 0x18 {
                return Err(Error {
                    code: ErrorCode::SensorError,
                    subcode: 0x0102,
                    timestamp: self.get_timestamp(),
                    data: twi.twsr.read().bits() as u32,
                });
            }
            
            // Read temperature from external LM75
            twi.twdr.write(|w| w.bits(0x00));
            twi.twcr.write(|w| w.bits(0x84));
            while twi.twcr.read().bits() & 0x80 == 0 {}
            
            lm75_temp = (twi.twdr.read().bits() as i16) << 8;
            twi.twcr.write(|w| w.bits(0x84));
            while twi.twcr.read().bits() & 0x80 == 0 {}
            
            lm75_temp |= twi.twdr.read().bits() as i16;
            twi.twcr.write(|w| w.bits(0x94));
            
            if lm75_temp > 85 * 256 { // 85°C max temperature
                return Err(Error {
                    code: ErrorCode::SystemError,
                    subcode: 0x0102,
                    timestamp: self.get_timestamp(),
                    data: lm75_temp as u32,
                });
            }
        }
        Ok(())
    }

    fn check_memory(&self) -> Result<(), Error> {
        extern "C" {
            static _heap_start: u8;
            static _heap_end: u8;
        }
        
        unsafe {
            let heap_start = &_heap_start as *const u8 as usize;
            let heap_end = &_heap_end as *const u8 as usize;
            let test_size = core::cmp::min(32, heap_end - heap_start);
            
            let test_patterns: [u8; 4] = [0x55, 0xAA, 0x33, 0xCC];
            
            for i in 0..test_size {
                for &pattern in test_patterns.iter() {
                    let addr = (heap_start + i) as *mut u8;
                    core::ptr::write_volatile(addr, pattern);
                    let readback = core::ptr::read_volatile(addr);
                    
                    if readback != pattern {
                        return Err(Error {
                            code: ErrorCode::MemoryError,
                            subcode: 0x0103,
                            timestamp: self.get_timestamp(),
                            data: ((i as u32) << 16) | ((pattern as u32) << 8) | readback as u32,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn check_peripherals(&self) -> Result<(), Error> {
        unsafe {
            let timer = &(*avr_device::atmega128::TC0::ptr());
            
            // Stop timer and clear settings
            timer.tccr0.write(|w| w.bits(0x00));
            timer.tcnt0.write(|w| w.bits(0x00));
            
            // Configure for normal mode, no prescaling
            timer.tccr0.write(|w| w.bits(0x01));
            
            // Wait for a few ticks
            let start = timer.tcnt0.read().bits();
            let mut timeout = 1000;
            
            while timer.tcnt0.read().bits() <= start && timeout > 0 {
                timeout -= 1;
            }
            
            if timeout == 0 {
                return Err(Error {
                    code: ErrorCode::HardwareFault,
                    subcode: 0x0104,
                    timestamp: self.get_timestamp(),
                    data: 0x01,
                });
            }
            
            // Stop timer
            timer.tccr0.write(|w| w.bits(0x00));
            
            // Test output compare functionality
            timer.tcnt0.write(|w| w.bits(0x00));
            timer.ocr0.write(|w| w.bits(0x80));
            timer.tccr0.write(|w| w.bits(0x08)); // CTC mode
            
            if timer.tcnt0.read().bits() > 0x80 {
                return Err(Error {
                    code: ErrorCode::HardwareFault,
                    subcode: 0x0104,
                    timestamp: self.get_timestamp(),
                    data: 0x02,
                });
            }
            
            // Cleanup
            timer.tccr0.write(|w| w.bits(0x00));
        }
        Ok(())
    }

    fn emergency_shutdown(&mut self) {
        self.logger.flush().ok();
        unsafe {
            let pmx = &(*avr_device::atmega128::PMX::ptr());
            pmx.mcucr.write(|w| w.bits(0x20));
            loop {
                avr_device::asm::sleep_mode();
            }
        }
    }

    fn enter_low_power_mode(&mut self) {
        unsafe {
            let pmx = &(*avr_device::atmega128::PMX::ptr());
            pmx.mcucr.write(|w| w.bits(0x10));
            avr_device::asm::sleep_mode();
        }
    }

    fn reset_system(&mut self) {
        self.logger.flush().ok();
        unsafe {
            let wdt = &(*avr_device::atmega128::WDT::ptr());
            wdt.wdtcr.write(|w| w.bits(0x18));
            wdt.wdtcr.write(|w| w.bits(0x08));
            loop {}
        }
    }

    fn get_timestamp(&self) -> u32 {
        // TODO: Implement real timestamp
        0
    }
}
