use avr_device::atmega128::CPU;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SleepMode {
    Idle = 0,
    AdcNoiseReduction = 1,
    PowerDown = 2,
    PowerSave = 3,
    Standby = 6,
    ExtendedStandby = 7,
}

pub struct Power {
    _private: (),
}

impl Power {
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[inline]
    pub fn set_sleep_mode(&mut self, mode: SleepMode) {
        unsafe {
            let p = CPU::ptr();
            (*p).mcucr.modify(|r, w| {
                w.bits((r.bits() & !0x70) | ((mode as u8) << 4))
            });
        }
    }

    #[inline]
    pub fn enable_sleep(&mut self) {
        unsafe {
            let p = CPU::ptr();
            (*p).mcucr.modify(|r, w| w.bits(r.bits() | 0x20));
        }
    }

    #[inline]
    pub fn disable_sleep(&mut self) {
        unsafe {
            let p = CPU::ptr();
            (*p).mcucr.modify(|r, w| w.bits(r.bits() & !0x20));
        }
    }

    #[inline]
    pub fn sleep(&mut self) {
        unsafe {
            avr_device::asm::sleep()
        }
    }

    pub fn enter_idle_mode(&mut self) {
        self.set_sleep_mode(SleepMode::Idle);
        self.enable_sleep();
        self.sleep();
        self.disable_sleep();
    }

    pub fn enter_power_down(&mut self) {
        self.set_sleep_mode(SleepMode::PowerDown);
        self.enable_sleep();
        self.sleep();
        self.disable_sleep();
    }

    // Module clock control
    pub fn disable_module_clock(&mut self, module: u8) {
        unsafe {
            let p = CPU::ptr();
            (*p).prr.modify(|r, w| w.bits(r.bits() | (1 << module)));
        }
    }

    pub fn enable_module_clock(&mut self, module: u8) {
        unsafe {
            let p = CPU::ptr();
            (*p).prr.modify(|r, w| w.bits(r.bits() & !(1 << module)));
        }
    }
}

impl Default for Power {
    fn default() -> Self {
        Self::new()
    }
} 