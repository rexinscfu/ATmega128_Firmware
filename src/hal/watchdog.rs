use avr_device::atmega128::WDT;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum WatchdogTimeout {
    Ms16 = 0,
    Ms32 = 1,
    Ms64 = 2,
    Ms125 = 3,
    Ms250 = 4,
    Ms500 = 5,
    Ms1000 = 6,
    Ms2000 = 7,
}

pub struct Watchdog {
    _private: (),
}

impl Watchdog {
    #[inline]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[inline]
    pub fn start(&mut self, timeout: WatchdogTimeout) {
        unsafe {
            let p = WDT::ptr();
            // Enable change bit and system reset mode
            (*p).wdtcr.write(|w| w.bits(0x18));
            // Set timeout and enable watchdog
            (*p).wdtcr.write(|w| w.bits(0x08 | timeout as u8));
        }
    }

    #[inline]
    pub fn feed(&mut self) {
        unsafe {
            avr_device::asm::wdr();
        }
    }

    #[inline]
    pub fn disable(&mut self) {
        unsafe {
            let p = WDT::ptr();
            // Timed sequence to disable watchdog
            (*p).wdtcr.write(|w| w.bits(0x18));
            (*p).wdtcr.write(|w| w.bits(0x00));
        }
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new()
    }
} 