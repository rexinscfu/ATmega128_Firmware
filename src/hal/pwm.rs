//! PWM (Pulse Width Modulation) HAL implementation
//! 
//! This module provides hardware PWM support using Timer1/3 in various modes.
//! 
//! Note to self: We might want to add support for Timer2 as well, but it's 8-bit
//! and the prescaler options are different. Left some old code commented out below
//! that shows how we could implement it.

#![no_std]

use avr_device::atmega128::{TC1, TC3};
use core::marker::PhantomData;

/// PWM frequency presets
#[derive(Clone, Copy)]
pub enum PwmFreq {
    Hz50 = 50,      // Typical for servos
    Hz200 = 200,    // Good for motors
    Hz400 = 400,    // Fast mode
    Hz1000 = 1000,  // Ultra fast (careful with this one)
}

/// PWM channel configuration
#[derive(Clone, Copy)]
pub enum PwmChannel {
    Timer1A,
    Timer1B,
    Timer1C,
    Timer3A,
    Timer3B,
    Timer3C,
}

/// PWM mode configuration
#[derive(Clone, Copy)]
pub enum PwmMode {
    Fast,           // Fast PWM mode
    PhaseCorrect,   // Phase correct PWM mode
    PhaseFreq,      // Phase and frequency correct PWM mode
}

/// PWM peripheral driver
pub struct Pwm<T> {
    _timer: PhantomData<T>,
    freq: PwmFreq,
    mode: PwmMode,
    
    // Cache configured values for dynamic updates
    period: u16,
    prescaler: u8,
    
    /* Commenting out experimental features
    // Advanced PWM features I was testing:
    #[allow(dead_code)]
    complementary_output: bool,  // Complementary output mode
    #[allow(dead_code)]
    dead_time_ns: u16,          // Dead time for complementary mode
    #[allow(dead_code)]
    fault_detection: bool,       // Fault detection enable
    
    // Different synchronization modes:
    #[allow(dead_code)]
    sync_modes: [bool; 4] = [
        false,  // Master mode
        false,  // Slave mode
        false,  // External trigger
        false   // Phase shift
    ];
    */
}

// Timer1 implementation
impl Pwm<TC1> {
    /// Create new PWM instance using Timer1
    pub fn new() -> Self {
        // Disable timer interrupts during initialization
        unsafe {
            (*TC1::ptr()).timsk.write(|w| w.bits(0));
        }
        
        Self {
            _timer: PhantomData,
            freq: PwmFreq::Hz50,
            mode: PwmMode::Fast,
            period: 0,
            prescaler: 0,
        }
    }

    /// Configure PWM frequency and mode
    pub fn configure(&mut self, freq: PwmFreq, mode: PwmMode) {
        self.freq = freq;
        self.mode = mode;
        
        // Calculate timer parameters for 16MHz clock
        let (period, prescaler) = match freq {
            PwmFreq::Hz50 => (40000, 8),   // 16MHz / (50Hz * 8) = 40000
            PwmFreq::Hz200 => (10000, 8),  // 16MHz / (200Hz * 8) = 10000
            PwmFreq::Hz400 => (5000, 8),   // 16MHz / (400Hz * 8) = 5000
            PwmFreq::Hz1000 => (2000, 8),  // 16MHz / (1000Hz * 8) = 2000
        };
        self.period = period;
        self.prescaler = prescaler;
        
        unsafe {
            let p = TC1::ptr();
            
            // Set PWM mode and prescaler
            match mode {
                PwmMode::Fast => {
                    (*p).tccr1a.write(|w| w.bits(0x02));  // Fast PWM, ICR1 top
                    (*p).tccr1b.write(|w| w.bits(0x18 | prescaler));
                }
                PwmMode::PhaseCorrect => {
                    (*p).tccr1a.write(|w| w.bits(0x02));  // Phase correct PWM, ICR1 top
                    (*p).tccr1b.write(|w| w.bits(0x10 | prescaler));
                }
                PwmMode::PhaseFreq => {
                    (*p).tccr1a.write(|w| w.bits(0x02));  // Phase & freq correct PWM
                    (*p).tccr1b.write(|w| w.bits(0x10 | prescaler));
                }
            }
            
            // Set period
            (*p).icr1.write(|w| w.bits(period));
        }
    }

    /// Set duty cycle for a channel (0-100%)
    pub fn set_duty(&mut self, channel: PwmChannel, duty: f32) {
        let duty = (duty.max(0.0).min(100.0) / 100.0) * self.period as f32;
        let duty = duty as u16;
        
        unsafe {
            let p = TC1::ptr();
            match channel {
                PwmChannel::Timer1A => {
                    (*p).tccr1a.modify(|r, w| w.bits(r.bits() | 0x80));
                    (*p).ocr1a.write(|w| w.bits(duty));
                }
                PwmChannel::Timer1B => {
                    (*p).tccr1a.modify(|r, w| w.bits(r.bits() | 0x20));
                    (*p).ocr1b.write(|w| w.bits(duty));
                }
                PwmChannel::Timer1C => {
                    (*p).tccr1a.modify(|r, w| w.bits(r.bits() | 0x08));
                    (*p).ocr1c.write(|w| w.bits(duty));
                }
                _ => {} // Invalid channel for Timer1
            }
        }
    }
}

// Timer3 implementation (similar to Timer1)
impl Pwm<TC3> {
    /// Create new PWM instance using Timer3
    pub fn new() -> Self {
        unsafe {
            (*TC3::ptr()).timsk.write(|w| w.bits(0));
        }
        
        Self {
            _timer: PhantomData,
            freq: PwmFreq::Hz50,
            mode: PwmMode::Fast,
            period: 0,
            prescaler: 0,
        }
    }

    // Implementation similar to Timer1...
    // TODO: Implement Timer3 specific functions
}

/* Old Timer2 8-bit PWM implementation - keeping for reference
#[allow(dead_code)]
impl Pwm<TC2> {
    const PRESCALER_VALUES: [u16; 7] = [1, 8, 32, 64, 128, 256, 1024];
    
    pub fn new() -> Self {
        unsafe {
            (*TC2::ptr()).timsk.write(|w| w.bits(0));
        }
        
        Self {
            _timer: PhantomData,
            freq: PwmFreq::Hz50,
            mode: PwmMode::Fast,
            period: 0,
            prescaler: 0,
        }
    }
    
    fn calculate_8bit_params(freq: u16) -> (u8, u8) {
        let clock = 16_000_000u32;
        let target = clock / freq as u32;
        
        for (i, prescaler) in Self::PRESCALER_VALUES.iter().enumerate() {
            let period = target / *prescaler as u32;
            if period <= 256 {
                return (i as u8, period as u8);
            }
        }
        
        (6, 255) // Default to max prescaler and period
    }
}
*/

impl<T> Default for Pwm<T> {
    fn default() -> Self {
        Self::new()
    }
}
