//! Application layer implementation for ATmega128 firmware
//! This module contains the high-level application logic

#![no_std]

use crate::drivers::{LedMatrix, SerialConsole, ButtonHandler, ButtonEvent};
use crate::hal::{Adc, AdcChannel};

/// Main application state and logic
pub struct Application {
    led_pattern: u8,
    adc_value: u16,
}

impl Application {
    /// Create new application instance
    pub fn new() -> Self {
        Self {
            led_pattern: 0,
            adc_value: 0,
        }
    }

    /// Update application state
    pub fn update(&mut self, 
        leds: &mut LedMatrix,
        console: &mut SerialConsole,
        buttons: &mut ButtonHandler,
        adc: &mut Adc
    ) {
        // Handle button events
        if let Some(event) = buttons.get_event() {
            match event {
                ButtonEvent::Pressed(button) => {
                    self.handle_button_press(button, console);
                }
                ButtonEvent::Released(_) => {}
            }
        }

        // Update LED pattern
        leds.set_pattern(self.led_pattern);
        self.led_pattern = self.led_pattern.wrapping_add(1);

        // Read ADC periodically
        self.adc_value = adc.read_channel(AdcChannel::ADC0);
    }

    fn handle_button_press(&mut self, button: crate::drivers::Button, console: &mut SerialConsole) {
        match button {
            crate::drivers::Button::Button1 => {
                console.write_line("Button 1 pressed!");
            }
            crate::drivers::Button::Button2 => {
                console.write_line("Button 2 pressed!");
            }
            crate::drivers::Button::Button3 => {
                console.write_line("Button 3 pressed!");
            }
            crate::drivers::Button::Button4 => {
                console.write_line("Button 4 pressed!");
            }
        }
    }
}
