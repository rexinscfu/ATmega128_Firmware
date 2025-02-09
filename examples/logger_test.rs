#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    logger::{Logger, LogEntry},
    drivers::{Flash, SerialConsole, Mpu6050},
    hal::{Spi, Twi, TwiSpeed, delay_ms},
};

#[avr_device::entry]
fn main() -> ! {
    let mut console = SerialConsole::new();
    console.write_line("Initializing data logger...");
    
    let spi = Spi::new();
    let flash = match Flash::new(spi, 0, 1, 2) {
        Ok(flash) => flash,
        Err(_) => {
            console.write_line("Failed to initialize Flash!");
            loop {}
        }
    };
    
    let mut logger = Logger::new(flash);
    if let Err(_) = logger.init() {
        console.write_line("Failed to initialize logger!");
        loop {}
    }
    
    let mut twi = Twi::new();
    twi.set_speed(TwiSpeed::Fast400k);
    
    let mut imu = match Mpu6050::new(twi) {
        Ok(imu) => imu,
        Err(_) => {
            console.write_line("Failed to initialize IMU!");
            loop {}
        }
    };
    
    logger.log_system(b"System initialized").ok();
    
    console.write_line("Reading previous logs...");
    logger.read_logs(|entry| {
        console.write_str("Log entry: Type=");
        console.write_u8(entry.log_type as u8);
        console.write_str(" Data=");
        for i in 0..entry.length {
            console.write_hex(entry.data[i as usize]);
            console.write_str(" ");
        }
        console.write_line("");
        Ok(())
    }).ok();
    
    console.write_line("Starting sensor logging...");
    
    let mut count = 0;
    loop {
        if let Ok(accel) = imu.read_accel() {
            let mut data = [0u8; 12];
            data[0..4].copy_from_slice(&accel.x.to_le_bytes());
            data[4..8].copy_from_slice(&accel.y.to_le_bytes());
            data[8..12].copy_from_slice(&accel.z.to_le_bytes());
            logger.log_sensor(&data).ok();
        }
        
        count += 1;
        if count >= 100 {
            logger.flush().ok();
            count = 0;
        }
        
        delay_ms(10);
    }
}
