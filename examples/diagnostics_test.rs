#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    diagnostics::{Diagnostics, ErrorCode},
    logger::Logger,
    drivers::{Flash, SerialConsole},
    hal::{Spi, delay_ms},
};

#[avr_device::entry]
fn main() -> ! {
    let mut console = SerialConsole::new();
    console.write_line("Starting diagnostics test...");
    
    let spi = Spi::new();
    let flash = match Flash::new(spi, 0, 1, 2) {
        Ok(flash) => flash,
        Err(_) => {
            console.write_line("Failed to initialize Flash!");
            loop {}
        }
    };
    
    let logger = Logger::new(flash);
    let mut diagnostics = Diagnostics::new(logger);
    
    console.write_line("Running system diagnostics...");
    match diagnostics.run_diagnostics() {
        Ok(_) => console.write_line("All diagnostics passed!"),
        Err(error) => {
            console.write_str("Diagnostic error: ");
            console.write_hex(error.code as u32);
            console.write_str(" subcode: ");
            console.write_hex(error.subcode as u32);
            console.write_line("");
        }
    }
    
    diagnostics.enable_watchdog();
    console.write_line("Watchdog enabled");
    
    let mut test_count = 0;
    loop {
        diagnostics.reset_watchdog();
        
        test_count += 1;
        if test_count % 100 == 0 {
            console.write_str("Error count: ");
            console.write_u32(diagnostics.get_error_count());
            console.write_line("");
            
            if let Some(error) = diagnostics.get_last_error() {
                console.write_str("Last error: ");
                console.write_hex(error.code as u32);
                console.write_line("");
            }
        }
        
        if test_count % 500 == 0 {
            diagnostics.report_error(
                ErrorCode::SensorError,
                0x0001,
                test_count as u32
            );
        }
        
        delay_ms(10);
    }
}
