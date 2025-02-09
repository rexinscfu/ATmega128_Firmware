pub mod adc;
pub mod gpio;
pub mod power;
pub mod spi;
pub mod timer;
pub mod twi;
pub mod uart;
pub mod watchdog;

// Re-export commonly used types
pub use adc::{Adc, AdcChannel, AdcPrescaler, AdcReference};
pub use gpio::board;
pub use gpio::{Input, Output, Pin};
pub use power::{Power, SleepMode};
pub use spi::{DataOrder, Spi, SpiMode, SpiPrescaler};
pub use timer::{delay_ms, Prescaler, Timer};
pub use twi::{Twi, TwiSpeed};
pub use uart::Uart;
pub use watchdog::{Watchdog, WatchdogTimeout};

// TODO: Add other HAL modules (PWM, etc)
#[allow(dead_code)]
pub(crate) struct Hal {
    // Will contain HAL instances
}