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

// Global state for interrupt handling
static GLOBAL_PERIPHERALS: Mutex<RefCell<Option<Peripherals>>> = 
    Mutex::new(RefCell::new(None));

#[avr_device::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    
    interrupt::free(|cs| {
        GLOBAL_PERIPHERALS.borrow(cs).replace(Some(dp));
    });

    // TODO: Initialize system clock and peripherals
    // TODO: Setup watchdog timer
    // TODO: Configure power management
    
    #[allow(clippy::empty_loop)]
    loop {
        // Main event loop will go here
        // For now, just enter sleep mode
        unsafe {
            avr_device::asm::sleep()
        }
    }
} 