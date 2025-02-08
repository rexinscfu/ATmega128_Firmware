use crate::hal::gpio::board::{LED0, LED1, LED2, LED3};
use crate::hal::gpio::Output;
use crate::hal::gpio::Pin;
use avr_device::atmega128::PORTA;

pub struct LedMatrix {
    leds: [Pin<PORTA, u8, Output>; 4],
}

impl LedMatrix {
    pub fn new() -> Self {
        // Create LED instances - note the unsafe block is needed for raw register access
        // but we ensure safety through our type system
        LedMatrix {
            leds: [
                LED0::default().into_output(),
                LED1::default().into_output(),
                LED2::default().into_output(),
                LED3::default().into_output(),
            ],
        }
    }

    pub fn set_pattern(&mut self, pattern: u8) {
        for (i, led) in self.leds.iter_mut().enumerate() {
            if (pattern & (1 << i)) != 0 {
                led.set_high();
            } else {
                led.set_low();
            }
        }
    }

    pub fn toggle_all(&mut self) {
        for led in self.leds.iter_mut() {
            led.toggle();
        }
    }

    pub fn set_all(&mut self, state: bool) {
        for led in self.leds.iter_mut() {
            if state {
                led.set_high();
            } else {
                led.set_low();
            }
        }
    }

    // Knight Rider effect 🚗 (because why not?)
    pub fn knight_rider(&mut self, delay_ms: u16) {
        for i in 0..self.leds.len() {
            self.set_pattern(1 << i);
            self.delay_ms(delay_ms);
        }
        for i in (0..self.leds.len()).rev() {
            self.set_pattern(1 << i);
            self.delay_ms(delay_ms);
        }
    }

    // TODO: Replace with proper timer-based delay
    #[inline(always)]
    fn delay_ms(&self, ms: u16) {
        for _ in 0..ms {
            for _ in 0..1600 { // Approximately 1ms at 16MHz
                unsafe { core::arch::asm!("nop") }
            }
        }
    }
}

impl Default for LedMatrix {
    fn default() -> Self {
        Self::new()
    }
} 