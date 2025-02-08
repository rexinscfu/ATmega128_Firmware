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

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    // Print startup message
    console.write_line("ATmega128 Firmware v0.1.0");
    console.write_line("Ready...");

    let mut led_pattern = 0u8;

    #[allow(clippy::empty_loop)]
    loop {
        // Handle button events
        if let Some(event) = buttons.poll() {
            match event {
                ButtonEvent::Pressed(btn) => {
                    match btn {
                        Button::Button0 => {
                            led_pattern = led_pattern.wrapping_add(1);
                            leds.set_pattern(led_pattern);
                            console.debug("Pattern", led_pattern);
                        },
                        Button::Button1 => {
                            led_pattern = led_pattern.wrapping_sub(1);
                            leds.set_pattern(led_pattern);
                            console.debug("Pattern", led_pattern);
                        },
                        Button::Button2 => {
                            leds.toggle_all();
                            console.write_line("Toggle all LEDs");
                        },
                        Button::Button3 => {
                            leds.knight_rider(100);
                            console.write_line("Knight Rider mode");
                        }
                    }
                },
                ButtonEvent::Released(_) => {
                    // Ignore release events for now
                }
            }
        }

        // Echo any received characters
        if let Some(byte) = console.read_byte() {
            console.write_byte(byte);
        }

        // Enter sleep mode to save power
        unsafe {
            avr_device::asm::sleep()
        }
    }
} 