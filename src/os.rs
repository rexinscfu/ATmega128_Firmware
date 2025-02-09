//! Minimal RTOS-like functionality for ATmega128
#![no_std]

use crate::config::CPU_FREQ_HZ;
use crate::hal::Power;
use core::cell::Cell;

/// Simple task scheduler and system time tracking
pub struct Scheduler {
    tick_count: Cell<u32>,
}

impl Scheduler {
    /// Create new scheduler instance
    pub const fn new() -> Self {
        Self {
            tick_count: Cell::new(0),
        }
    }

    /// Increment system tick counter
    #[inline]
    pub fn tick(&self) {
        let count = self.tick_count.get();
        self.tick_count.set(count.wrapping_add(1));
    }

    /// Get current system tick count
    #[inline]
    pub fn get_ticks(&self) -> u32 {
        self.tick_count.get()
    }

    /// Enter sleep mode until next interrupt
    #[inline]
    pub fn sleep(&self, power: &mut Power) {
        power.sleep();
    }
}

/// Global scheduler instance
pub static SCHEDULER: Scheduler = Scheduler::new();
