//! Configuration constants for ATmega128 firmware
#![no_std]

/// CPU frequency in Hz
pub const CPU_FREQ_HZ: u32 = 16_000_000;

/// UART baud rate
pub const UART_BAUD: u32 = 9600;

/// ADC reference voltage in millivolts
pub const ADC_VREF_MV: u16 = 5000;

/// Watchdog timeout period in milliseconds
pub const WDT_TIMEOUT_MS: u16 = 1000;

/// LED matrix update interval in milliseconds
pub const LED_UPDATE_MS: u16 = 100;

/// Button debounce time in milliseconds
pub const BUTTON_DEBOUNCE_MS: u16 = 50;
