pub mod gpio;

// Re-export commonly used types
pub use gpio::{Pin, Input, Output};
pub use gpio::board;

// TODO: Add other HAL modules (UART, SPI, TWI, etc)
#[allow(dead_code)]
pub(crate) struct Hal {
    // Will contain HAL instances
} 