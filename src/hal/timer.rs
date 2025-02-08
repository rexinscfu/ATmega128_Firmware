use avr_device::atmega128::{TC0, TC1};
use core::marker::PhantomData;

pub trait TimerRegisterBlock {
    fn ptr() -> *mut avr_device::atmega128::tc0::RegisterBlock;
    const PRESCALER_MASK: u8;
}

impl TimerRegisterBlock for TC0 {
    fn ptr() -> *mut avr_device::atmega128::tc0::RegisterBlock {
        TC0::ptr()
    }
    const PRESCALER_MASK: u8 = 0x07;
}

impl TimerRegisterBlock for TC1 {
    fn ptr() -> *mut avr_device::atmega128::tc0::RegisterBlock {
        TC1::ptr() as *mut _
    }
    const PRESCALER_MASK: u8 = 0x07;
}

#[derive(Clone, Copy)]
pub enum Prescaler {
    Stop = 0,
    Direct = 1,
    Div8 = 2,
    Div64 = 3,
    Div256 = 4,
    Div1024 = 5,
}

pub struct Timer<T> {
    _timer: PhantomData<T>,
}

impl<T: TimerRegisterBlock> Timer<T> {
    pub fn new() -> Self {
        unsafe {
            // Initialize timer in normal mode
            let p = T::ptr();
            (*p).tccr.write(|w| w.bits(0));
            (*p).tcnt.write(|w| w.bits(0));
        }
        Self { _timer: PhantomData }
    }

    pub fn start(&mut self, prescaler: Prescaler) {
        unsafe {
            let p = T::ptr();
            (*p).tccr.modify(|r, w| {
                w.bits((r.bits() & !T::PRESCALER_MASK) | (prescaler as u8 & T::PRESCALER_MASK))
            });
        }
    }

    pub fn stop(&mut self) {
        unsafe {
            let p = T::ptr();
            (*p).tccr.modify(|r, w| w.bits(r.bits() & !T::PRESCALER_MASK));
        }
    }

    pub fn set_counter(&mut self, value: u8) {
        unsafe {
            let p = T::ptr();
            (*p).tcnt.write(|w| w.bits(value));
        }
    }

    pub fn get_counter(&self) -> u8 {
        unsafe {
            let p = T::ptr();
            (*p).tcnt.read().bits()
        }
    }

    pub fn enable_overflow_interrupt(&mut self) {
        unsafe {
            let p = T::ptr();
            (*p).timsk.modify(|r, w| w.bits(r.bits() | 1));
        }
    }

    pub fn disable_overflow_interrupt(&mut self) {
        unsafe {
            let p = T::ptr();
            (*p).timsk.modify(|r, w| w.bits(r.bits() & !1));
        }
    }

    pub fn enable_pwm(&mut self, duty: u8) {
        unsafe {
            let p = T::ptr();
            // Fast PWM mode, non-inverting
            (*p).tccr.modify(|r, w| w.bits(r.bits() | (3 << 6) | (2 << 4)));
            (*p).ocr.write(|w| w.bits(duty));
        }
    }

    pub fn set_pwm_duty(&mut self, duty: u8) {
        unsafe {
            let p = T::ptr();
            (*p).ocr.write(|w| w.bits(duty));
        }
    }
}

// Millisecond delay using Timer0
pub fn delay_ms(ms: u16) {
    let mut timer = Timer::<TC0>::new();
    
    // Configure for 1ms ticks (16MHz/64 = 250kHz, 250 ticks = 1ms)
    timer.set_counter(0);
    timer.start(Prescaler::Div64);

    for _ in 0..ms {
        while timer.get_counter() < 250 {}
        timer.set_counter(0);
    }

    timer.stop();
} 