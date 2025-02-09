//! Data logging system implementation
#![no_std]

use crate::drivers::flash::Flash;
use crate::hal::timer::Timer;

pub struct LogEntry {
    timestamp: u32,
    log_type: LogType,
    data: [u8; 16],
    length: u8,
}

#[derive(Clone, Copy)]
pub enum LogType {
    System = 0,
    Sensor = 1,
    Error = 2,
    Debug = 3,
}

pub struct Logger {
    flash: Flash,
    current_sector: u32,
    write_pointer: u32,
    buffer: [LogEntry; 32],
    buffer_index: usize,
}

impl Logger {
    pub fn new(flash: Flash) -> Self {
        Self {
            flash,
            current_sector: 0,
            write_pointer: 0,
            buffer: [LogEntry {
                timestamp: 0,
                log_type: LogType::System,
                data: [0; 16],
                length: 0,
            }; 32],
            buffer_index: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), ()> {
        self.current_sector = self.find_last_sector()?;
        self.write_pointer = self.find_write_pointer()?;
        Ok(())
    }

    pub fn log_system(&mut self, data: &[u8]) -> Result<(), ()> {
        self.log_entry(LogType::System, data)
    }

    pub fn log_sensor(&mut self, data: &[u8]) -> Result<(), ()> {
        self.log_entry(LogType::Sensor, data)
    }

    pub fn log_error(&mut self, data: &[u8]) -> Result<(), ()> {
        self.log_entry(LogType::Error, data)
    }

    pub fn log_debug(&mut self, data: &[u8]) -> Result<(), ()> {
        self.log_entry(LogType::Debug, data)
    }

    fn log_entry(&mut self, log_type: LogType, data: &[u8]) -> Result<(), ()> {
        if data.len() > 16 {
            return Err(());
        }

        let entry = LogEntry {
            timestamp: get_timestamp(),
            log_type,
            data: {
                let mut buf = [0u8; 16];
                buf[..data.len()].copy_from_slice(data);
                buf
            },
            length: data.len() as u8,
        };

        self.buffer[self.buffer_index] = entry;
        self.buffer_index += 1;

        if self.buffer_index >= self.buffer.len() {
            self.flush()?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        if self.buffer_index == 0 {
            return Ok(());
        }

        let data = unsafe {
            core::slice::from_raw_parts(
                self.buffer.as_ptr() as *const u8,
                self.buffer_index * core::mem::size_of::<LogEntry>(),
            )
        };

        if self.write_pointer + data.len() as u32 > 0x1000 {
            self.current_sector += 1;
            if self.current_sector >= 0x100 {
                self.current_sector = 0;
            }
            self.flash.erase_sector(self.current_sector * 0x1000)?;
            self.write_pointer = 0;
        }

        self.flash.write(
            self.current_sector * 0x1000 + self.write_pointer,
            data,
        )?;

        self.write_pointer += data.len() as u32;
        self.buffer_index = 0;

        Ok(())
    }

    pub fn read_logs(&mut self, callback: fn(&LogEntry) -> Result<(), ()>) -> Result<(), ()> {
        let mut sector = 0;
        while sector <= self.current_sector {
            let mut buffer = [0u8; core::mem::size_of::<LogEntry>()];
            let mut offset = 0;

            while offset < 0x1000 {
                self.flash.read(
                    sector * 0x1000 + offset,
                    &mut buffer,
                )?;

                let entry = unsafe {
                    core::ptr::read(buffer.as_ptr() as *const LogEntry)
                };

                if entry.timestamp == 0xFFFFFFFF {
                    break;
                }

                callback(&entry)?;
                offset += core::mem::size_of::<LogEntry>() as u32;
            }
            sector += 1;
        }
        Ok(())
    }

    fn find_last_sector(&mut self) -> Result<u32, ()> {
        for sector in 0..0x100 {
            let mut buffer = [0u8; 4];
            self.flash.read(sector * 0x1000, &mut buffer)?;
            if buffer == [0xFF; 4] {
                return Ok(if sector == 0 { 0 } else { sector - 1 });
            }
        }
        Ok(0)
    }

    fn find_write_pointer(&mut self) -> Result<u32, ()> {
        let mut left = 0;
        let mut right = 0x1000;

        while left < right {
            let mid = left + (right - left) / 2;
            let mid = mid - (mid % core::mem::size_of::<LogEntry>() as u32);

            let mut buffer = [0u8; 4];
            self.flash.read(
                self.current_sector * 0x1000 + mid,
                &mut buffer,
            )?;

            if buffer == [0xFF; 4] {
                right = mid;
            } else {
                left = mid + core::mem::size_of::<LogEntry>() as u32;
            }
        }

        Ok(left)
    }
}

fn get_timestamp() -> u32 {
    // TODO: Implement real timestamp
    0
}
