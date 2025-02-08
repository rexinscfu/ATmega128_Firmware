#![allow(clippy::missing_safety_doc)]

use avr_device::atmega128::{USART0, USART1};
use core::marker::PhantomData;
use core::cell::RefCell;
use avr_device::interrupt::Mutex;

// Buffer size must be power of 2 for efficient masking
const BUFFER_SIZE: usize = 32;
const BUFFER_MASK: usize = BUFFER_SIZE - 1;

// Baud rate calculation (16MHz clock)
const UBRR_9600: u16 = 103;  // (16_000_000 / (16 * 9600)) - 1

pub struct Buffer {
    data: [u8; BUFFER_SIZE],
    write_idx: usize,
    read_idx: usize,
}

impl Buffer {
    const fn new() -> Self {
        Self {
            data: [0; BUFFER_SIZE],
            write_idx: 0,
            read_idx: 0,
        }
    }

    fn write(&mut self, byte: u8) -> bool {
        let next_write = (self.write_idx + 1) & BUFFER_MASK;
        if next_write != self.read_idx {
            self.data[self.write_idx] = byte;
            self.write_idx = next_write;
            true
        } else {
            false
        }
    }

    fn read(&mut self) -> Option<u8> {
        if self.read_idx != self.write_idx {
            let byte = self.data[self.read_idx];
            self.read_idx = (self.read_idx + 1) & BUFFER_MASK;
            Some(byte)
        } else {
            None
        }
    }
}

// Global buffers for interrupt handlers
static TX_BUFFER: Mutex<RefCell<Buffer>> = Mutex::new(RefCell::new(Buffer::new()));
static RX_BUFFER: Mutex<RefCell<Buffer>> = Mutex::new(RefCell::new(Buffer::new()));

pub struct Uart<USART> {
    usart: PhantomData<USART>,
}

impl<USART: UartRegisterBlock> Uart<USART> {
    pub fn new() -> Self {
        unsafe {
            let p = USART::ptr();
            
            // Set baud rate
            (*p).ubrr.write(|w| w.bits(UBRR_9600));
            
            // Enable TX, RX and RX interrupt
            (*p).ucsr.modify(|_, w| {
                w.rxen().set_bit()
                 .txen().set_bit()
                 .rxcie().set_bit()
            });
        }
        
        Self {
            usart: PhantomData,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        avr_device::interrupt::free(|cs| {
            if !TX_BUFFER.borrow(cs).borrow_mut().write(byte) {
                // Buffer full - enable TX interrupt to start sending
                unsafe {
                    (*USART::ptr()).ucsr.modify(|_, w| w.udrie().set_bit());
                }
            }
        });
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        avr_device::interrupt::free(|cs| {
            RX_BUFFER.borrow(cs).borrow_mut().read()
        })
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

// Trait for USART register block access
pub trait UartRegisterBlock {
    fn ptr() -> *mut avr_device::atmega128::usart0::RegisterBlock;
}

// Implement for both USART0 and USART1
impl UartRegisterBlock for USART0 {
    fn ptr() -> *mut avr_device::atmega128::usart0::RegisterBlock {
        USART0::ptr()
    }
}

impl UartRegisterBlock for USART1 {
    fn ptr() -> *mut avr_device::atmega128::usart0::RegisterBlock {
        USART1::ptr() as *mut _
    }
}

// Interrupt handlers
#[avr_device::interrupt(atmega128)]
fn USART0_RX() {
    unsafe {
        let byte = (*USART0::ptr()).udr.read().bits();
        avr_device::interrupt::free(|cs| {
            RX_BUFFER.borrow(cs).borrow_mut().write(byte);
        });
    }
}

#[avr_device::interrupt(atmega128)]
fn USART0_UDRE() {
    avr_device::interrupt::free(|cs| {
        if let Some(byte) = TX_BUFFER.borrow(cs).borrow_mut().read() {
            unsafe {
                (*USART0::ptr()).udr.write(|w| w.bits(byte));
            }
        } else {
            // Buffer empty - disable TX interrupt
            unsafe {
                (*USART0::ptr()).ucsr.modify(|_, w| w.udrie().clear_bit());
            }
        }
    });
} 