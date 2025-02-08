use crate::hal::gpio::board::{BTN0, BTN1, BTN2, BTN3};
use crate::hal::gpio::{Input, Pin};
use avr_device::atmega128::PORTB;

const DEBOUNCE_TICKS: u8 = 5; // ~5ms debounce time

pub struct ButtonHandler {
    buttons: [Pin<PORTB, u8, Input>; 4],
    states: [bool; 4],
    debounce_counters: [u8; 4],
}

#[derive(Copy, Clone, Debug)]
pub enum Button {
    Button0,
    Button1,
    Button2,
    Button3,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonEvent {
    Pressed(Button),
    Released(Button),
}

impl ButtonHandler {
    pub fn new() -> Self {
        Self {
            buttons: [
                BTN0::default().into_input(),
                BTN1::default().into_input(),
                BTN2::default().into_input(),
                BTN3::default().into_input(),
            ],
            states: [false; 4],
            debounce_counters: [0; 4],
        }
    }

    pub fn poll(&mut self) -> Option<ButtonEvent> {
        for (idx, button) in self.buttons.iter().enumerate() {
            let raw_state = button.is_low(); // Buttons are active low
            
            if raw_state != self.states[idx] {
                self.debounce_counters[idx] = self.debounce_counters[idx].saturating_add(1);
                if self.debounce_counters[idx] >= DEBOUNCE_TICKS {
                    self.states[idx] = raw_state;
                    self.debounce_counters[idx] = 0;
                    
                    let btn = match idx {
                        0 => Button::Button0,
                        1 => Button::Button1,
                        2 => Button::Button2,
                        3 => Button::Button3,
                        _ => unreachable!(),
                    };
                    
                    return Some(if raw_state {
                        ButtonEvent::Pressed(btn)
                    } else {
                        ButtonEvent::Released(btn)
                    });
                }
            } else {
                self.debounce_counters[idx] = 0;
            }
        }
        None
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        let idx = match button {
            Button::Button0 => 0,
            Button::Button1 => 1,
            Button::Button2 => 2,
            Button::Button3 => 3,
        };
        self.states[idx]
    }
}

impl Default for ButtonHandler {
    fn default() -> Self {
        Self::new()
    }
} 