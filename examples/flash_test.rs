#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    drivers::{Flash, FlashError, SerialConsole},
    hal::{Spi, SpiMode},
};

const FLASH_CS_PIN: u8 = 0;
const FLASH_WP_PIN: u8 = 1;
const FLASH_HOLD_PIN: u8 = 2;

#[avr_device::entry]
fn main() -> ! {
    let mut console = SerialConsole::new();
    let spi = Spi::new();
    
    let mut flash = match Flash::new(spi, FLASH_CS_PIN, FLASH_WP_PIN, FLASH_HOLD_PIN) {
        Ok(flash) => flash,
        Err(_) => {
            console.write_line("Failed to initialize flash!");
            loop {}
        }
    };
    
    console.write_line("Flash initialized successfully");
    
    let test_data = [0x55; 1024];
    let mut read_buffer = [0u8; 1024];
    
    match flash.erase_sector(0) {
        Ok(_) => console.write_line("Sector erased"),
        Err(_) => console.write_line("Erase failed!"),
    }
    
    match flash.write(0, &test_data) {
        Ok(_) => console.write_line("Data written"),
        Err(_) => console.write_line("Write failed!"),
    }
    
    match flash.read(0, &mut read_buffer) {
        Ok(_) => {
            console.write_line("Data read:");
            for i in 0..16 {
                console.write_hex(read_buffer[i]);
                console.write_str(" ");
            }
            console.write_line("");
        }
        Err(_) => console.write_line("Read failed!"),
    }
    
    loop {
        flash.power_down().ok();
    }
}
