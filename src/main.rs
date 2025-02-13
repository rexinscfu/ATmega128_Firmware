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

use drivers::{LedMatrix, SerialConsole, ButtonHandler, ButtonEvent, Button};
use hal::{Power, SleepMode, Watchdog, WatchdogTimeout, Adc, AdcChannel};
use application::Application;
use os::Scheduler;

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
    let mut buttons = ButtonHandler::new();
    let mut power = Power::new();
    let mut watchdog = Watchdog::new();
    let mut adc = Adc::new();
    let mut scheduler = Scheduler::new();

    // Enable watchdog with 1s timeout
    watchdog.start(WatchdogTimeout::Ms1000);

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    // Print startup message
    console.write_line("ATmega128 Firmware v0.1.0");
    console.write_line("Ready...");

    // Main application loop
    let mut app = Application::new();
    
    loop {
        let ticks = scheduler.get_ticks();
        
        // Update application state
        app.update(&mut leds, &mut console, &mut buttons, &mut adc, ticks);
        
        // Pet watchdog
        watchdog.feed();
        
        // Enter sleep mode until next tick
        scheduler.sleep(&mut power);
    }
} 