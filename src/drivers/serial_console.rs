use crate::hal::Uart;
use avr_device::atmega128::USART0;

pub struct SerialConsole {
    uart: Uart<USART0>,
}

impl SerialConsole {
    pub fn new() -> Self {
        Self {
            uart: Uart::new(),
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.uart.write_str(s);
    }

    pub fn write_line(&mut self, s: &str) {
        self.write_str(s);
        self.write_str("\r\n");
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        self.uart.read_byte()
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.uart.write_byte(byte);
    }

    // Debug helper - print hex value
    pub fn write_hex(&mut self, val: u8) {
        const HEX_CHARS: [u8; 16] = *b"0123456789ABCDEF";
        self.write_byte(HEX_CHARS[(val >> 4) as usize]);
        self.write_byte(HEX_CHARS[(val & 0xF) as usize]);
    }

    // Print formatted debug info
    pub fn debug(&mut self, msg: &str, val: u8) {
        self.write_str("[DBG] ");
        self.write_str(msg);
        self.write_str(": 0x");
        self.write_hex(val);
        self.write_str("\r\n");
    }
}

impl Default for SerialConsole {
    fn default() -> Self {
        Self::new()
    }
} 