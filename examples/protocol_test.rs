#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    protocol::{Protocol, Command, Result, ProtocolError},
    hal::uart::Uart,
    drivers::SerialConsole,
};

static mut CONSOLE: Option<SerialConsole> = None;

#[avr_device::entry]
fn main() -> ! {
    unsafe {
        CONSOLE = Some(SerialConsole::new());
    }
    
    let uart = Uart::new();
    let mut protocol = Protocol::new(uart);
    protocol.set_packet_handler(handle_packet);
    
    loop {
        if let Err(err) = protocol.process() {
            unsafe {
                if let Some(console) = &mut CONSOLE {
                    console.write_str("Protocol error: ");
                    match err {
                        ProtocolError::BufferOverflow => console.write_line("Buffer overflow"),
                        ProtocolError::InvalidChecksum => console.write_line("Invalid checksum"),
                        ProtocolError::InvalidPacket => console.write_line("Invalid packet"),
                        ProtocolError::InvalidCommand => console.write_line("Invalid command"),
                        ProtocolError::Timeout => console.write_line("Timeout"),
                        ProtocolError::TransportError => console.write_line("Transport error"),
                    }
                }
            }
        }
        
        protocol.send_ping().ok();
        protocol.send_status(0x42).ok();
        
        let sensor_data = [0x01, 0x02, 0x03, 0x04];
        protocol.send_data(&sensor_data).ok();
    }
}

fn handle_packet(data: &[u8]) -> Result<()> {
    unsafe {
        if let Some(console) = &mut CONSOLE {
            console.write_str("Received packet: ");
            for &byte in data {
                console.write_hex(byte);
                console.write_str(" ");
            }
            console.write_line("");
        }
    }
    Ok(())
}
