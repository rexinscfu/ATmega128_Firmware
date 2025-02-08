#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
use avr_device::atmega128::Peripherals;
use core::cell::RefCell;
use avr_device::interrupt::{self, Mutex};

mod hal;
mod drivers;
mod application;
mod config;
mod os;

use drivers::{LedMatrix, SerialConsole};

// Global state for interrupt handling
static GLOBAL_PERIPHERALS: Mutex<RefCell<Option<Peripherals>>> = 
    Mutex::new(RefCell::new(None));

#[avr_device::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    
    interrupt::free(|cs| {
        GLOBAL_PERIPHERALS.borrow(cs).replace(Some(dp));
    });

    // Initialize drivers
    let mut console = SerialConsole::new();
    let mut leds = LedMatrix::new();

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    // Print startup message
    console.write_line("ATmega128 Firmware v0.1.0");
    console.write_line("Ready...");

    #[allow(clippy::empty_loop)]
    loop {
        // Echo any received characters and toggle LED pattern
        if let Some(byte) = console.read_byte() {
            console.write_byte(byte);
            leds.set_pattern(byte & 0x0F);
        }

        // Enter sleep mode to save power
        unsafe {
            avr_device::asm::sleep()
        }
    }
} 