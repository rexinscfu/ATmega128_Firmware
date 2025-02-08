pub mod gpio;
pub mod uart;
pub mod timer;

// Re-export commonly used types
pub use gpio::{Pin, Input, Output};
pub use gpio::board;
pub use uart::Uart;
pub use timer::{Timer, Prescaler, delay_ms};

// TODO: Add other HAL modules (SPI, TWI, etc)
#[allow(dead_code)]
pub(crate) struct Hal {
    // Will contain HAL instances
} 