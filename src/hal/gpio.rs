use avr_device::atmega128::{PORTA, PORTB, PORTC, PORTD, PORTE, PORTF};
use core::marker::PhantomData;

pub trait PinMode {}
pub struct Input;
pub struct Output;
impl PinMode for Input {}
impl PinMode for Output {}

#[derive(Debug)]
pub struct Pin<PORT, const PIN: u8, MODE> {
    _port: PhantomData<PORT>,
    _mode: PhantomData<MODE>,
}

macro_rules! impl_port {
    ($PORT:ident, $port:ident) => {
        impl<const P: u8, MODE: PinMode> Pin<$PORT, P, MODE> {
            pub fn into_output(self) -> Pin<$PORT, P, Output> {
                // Set DDRx bit
                unsafe {
                    (*$PORT::ptr()).$port.ddr.modify(|r, w| w.bits(r.bits() | (1 << P)));
                }
                Pin {
                    _port: PhantomData,
                    _mode: PhantomData,
                }
            }

            pub fn into_input(self) -> Pin<$PORT, P, Input> {
                // Clear DDRx bit and disable pull-up
                unsafe {
                    (*$PORT::ptr()).$port.ddr.modify(|r, w| w.bits(r.bits() & !(1 << P)));
                    (*$PORT::ptr()).$port.port.modify(|r, w| w.bits(r.bits() & !(1 << P)));
                }
                Pin {
                    _port: PhantomData,
                    _mode: PhantomData,
                }
            }
        }
    };
}

// Implement for all ATmega128 ports
impl_port!(PORTA, porta);
impl_port!(PORTB, portb);
impl_port!(PORTC, portc);
impl_port!(PORTD, portd);
impl_port!(PORTE, porte);
impl_port!(PORTF, portf);

// Output pin implementation
impl<PORT, const P: u8> Pin<PORT, P, Output> {
    #[inline]
    pub fn set_high(&mut self) where Self: PinOps {
        unsafe {
            self.port_ptr().port.modify(|r, w| w.bits(r.bits() | (1 << P)));
        }
    }

    #[inline]
    pub fn set_low(&mut self) where Self: PinOps {
        unsafe {
            self.port_ptr().port.modify(|r, w| w.bits(r.bits() & !(1 << P)));
        }
    }

    #[inline]
    pub fn toggle(&mut self) where Self: PinOps {
        unsafe {
            self.port_ptr().pin.modify(|r, w| w.bits(r.bits() ^ (1 << P)));
        }
    }
}

// Input pin implementation
impl<PORT, const P: u8> Pin<PORT, P, Input> {
    #[inline]
    pub fn is_high(&self) -> bool where Self: PinOps {
        unsafe {
            (self.port_ptr().pin.read().bits() & (1 << P)) != 0
        }
    }

    #[inline]
    pub fn is_low(&self) -> bool where Self: PinOps {
        !self.is_high()
    }
}

// Internal trait for port operations
trait PinOps {
    type PORT;
    fn port_ptr(&self) -> &Self::PORT;
}

// Implement PinOps for each port
macro_rules! impl_pin_ops {
    ($PORT:ident) => {
        impl<const P: u8, MODE> PinOps for Pin<$PORT, P, MODE> {
            type PORT = $PORT;
            #[inline]
            fn port_ptr(&self) -> &Self::PORT {
                unsafe { &*$PORT::ptr() }
            }
        }
    };
}

impl_pin_ops!(PORTA);
impl_pin_ops!(PORTB);
impl_pin_ops!(PORTC);
impl_pin_ops!(PORTD);
impl_pin_ops!(PORTE);
impl_pin_ops!(PORTF);

// BigAVR2 board-specific pin definitions
pub mod board {
    use super::*;
    
    // LED definitions (PORTA)
    pub type LED0 = Pin<PORTA, 0, Output>;
    pub type LED1 = Pin<PORTA, 1, Output>;
    pub type LED2 = Pin<PORTA, 2, Output>;
    pub type LED3 = Pin<PORTA, 3, Output>;
    
    // Button definitions (PORTB)
    pub type BTN0 = Pin<PORTB, 0, Input>;
    pub type BTN1 = Pin<PORTB, 1, Input>;
    pub type BTN2 = Pin<PORTB, 2, Input>;
    pub type BTN3 = Pin<PORTB, 3, Input>;
    
    // TODO: Add more board-specific pins (UART, SPI, etc)
} 