#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    bootloader::Bootloader,
    hal::{Flash, Uart, Spi},
    drivers::SerialConsole,
};

#[avr_device::entry]
fn main() -> ! {
    let mut console = SerialConsole::new();
    console.write_line("Starting bootloader...");
    
    let uart = Uart::new();
    let spi = Spi::new();
    
    let flash = match Flash::new(spi, 0, 1, 2) {
        Ok(flash) => flash,
        Err(_) => {
            console.write_line("Failed to initialize Flash!");
            loop {}
        }
    };
    
    let mut bootloader = Bootloader::new(flash, uart);
    bootloader.enter_bootloader();
    
    console.write_line("Bootloader ready");
    console.write_line("Waiting for firmware update...");
    
    loop {
        if let Err(_) = bootloader.process() {
            console.write_line("Bootloader error occurred");
        }
    }
}
