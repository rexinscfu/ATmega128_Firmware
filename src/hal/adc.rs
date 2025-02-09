use avr_device::atmega128::ADC;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AdcChannel {
    Adc0 = 0,
    Adc1 = 1,
    Adc2 = 2,
    Adc3 = 3,
    Adc4 = 4,
    Adc5 = 5,
    Adc6 = 6,
    Adc7 = 7,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AdcReference {
    Aref = 0,            // External AREF
    Avcc = 1,            // AVCC with external cap at AREF
    Internal2_56V = 3,   // Internal 2.56V with external cap at AREF
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AdcPrescaler {
    Div2 = 0,
    Div4 = 2,
    Div8 = 3,
    Div16 = 4,
    Div32 = 5,
    Div64 = 6,
    Div128 = 7,
}

pub struct Adc {
    _private: (),
}

impl Adc {
    pub fn new() -> Self {
        unsafe {
            let p = ADC::ptr();
            // Enable ADC, prescaler div128 (125kHz @ 16MHz)
            (*p).adcsra.write(|w| w.bits(0x87));
            // Reference voltage = AVCC
            (*p).admux.write(|w| w.bits(0x40));
        }
        Self { _private: () }
    }

    pub fn set_reference(&mut self, reference: AdcReference) {
        unsafe {
            let p = ADC::ptr();
            (*p).admux.modify(|r, w| {
                w.bits((r.bits() & 0x3F) | ((reference as u8) << 6))
            });
        }
    }

    pub fn set_prescaler(&mut self, prescaler: AdcPrescaler) {
        unsafe {
            let p = ADC::ptr();
            (*p).adcsra.modify(|r, w| {
                w.bits((r.bits() & 0xF8) | (prescaler as u8))
            });
        }
    }

    pub fn read_channel(&mut self, channel: AdcChannel) -> u16 {
        unsafe {
            let p = ADC::ptr();
            
            // Select channel
            (*p).admux.modify(|r, w| {
                w.bits((r.bits() & 0xE0) | (channel as u8))
            });
            
            // Start conversion
            (*p).adcsra.modify(|r, w| w.bits(r.bits() | 0x40));
            
            // Wait for completion
            while (*p).adcsra.read().bits() & 0x40 != 0 {}
            
            // Read result (ADCL must be read first)
            let low = (*p).adcl.read().bits() as u16;
            let high = (*p).adch.read().bits() as u16;
            
            (high << 8) | low
        }
    }

    pub fn read_voltage(&mut self, channel: AdcChannel) -> f32 {
        let raw = self.read_channel(channel);
        // Convert to voltage (assuming AVCC = 5V)
        (raw as f32) * 5.0 / 1024.0
    }

    pub fn enable_interrupt(&mut self) {
        unsafe {
            let p = ADC::ptr();
            (*p).adcsra.modify(|r, w| w.bits(r.bits() | 0x08));
        }
    }

    pub fn disable_interrupt(&mut self) {
        unsafe {
            let p = ADC::ptr();
            (*p).adcsra.modify(|r, w| w.bits(r.bits() & !0x08));
        }
    }
}

impl Default for Adc {
    fn default() -> Self {
        Self::new()
    }
} 