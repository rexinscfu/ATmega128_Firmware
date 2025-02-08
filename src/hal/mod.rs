pub mod gpio;
pub mod uart;
pub mod timer;
pub mod power;

// Re-export commonly used types
pub use gpio::{Pin, Input, Output};
pub use gpio::board;
pub use uart::Uart;
pub use timer::{Timer, Prescaler, delay_ms};
pub use power::{Power, SleepMode};

// TODO: Add other HAL modules (SPI, TWI, etc)
#[allow(dead_code)]
pub(crate) struct Hal {
    // Will contain HAL instances
} 