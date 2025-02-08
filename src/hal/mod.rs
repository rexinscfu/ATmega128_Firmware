pub mod gpio;
pub mod uart;

// Re-export commonly used types
pub use gpio::{Pin, Input, Output};
pub use gpio::board;
pub use uart::Uart;

// TODO: Add other HAL modules (SPI, TWI, etc)
#[allow(dead_code)]
pub(crate) struct Hal {
    // Will contain HAL instances
} 