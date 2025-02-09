//! MPU6050 IMU demo application
#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    drivers::{Mpu6050, SerialConsole, AccelScale, GyroScale},
    hal::{Twi, TwiSpeed},
};

#[avr_device::entry]
fn main() -> ! {
    // Initialize peripherals
    let mut console = SerialConsole::new();
    let mut twi = Twi::new();
    
    // Configure TWI for fast mode
    twi.set_speed(TwiSpeed::Fast400k);
    
    // Initialize MPU6050
    let mut mpu = match Mpu6050::new(twi) {
        Ok(mpu) => {
            console.write_line("MPU6050 initialized successfully");
            mpu
        }
        Err(_) => {
            console.write_line("Failed to initialize MPU6050!");
            loop {}
        }
    };
    
    // Configure sensor ranges
    if let Err(_) = mpu.set_accel_scale(AccelScale::G8) {
        console.write_line("Failed to set accelerometer scale!");
    }
    if let Err(_) = mpu.set_gyro_scale(GyroScale::Dps1000) {
        console.write_line("Failed to set gyroscope scale!");
    }
    
    loop {
        // Read accelerometer data
        if let Ok(accel) = mpu.read_accel() {
            console.write_str("Accel (g): ");
            console.write_str("X=");
            console.write_float(accel.x);
            console.write_str(" Y=");
            console.write_float(accel.y);
            console.write_str(" Z=");
            console.write_float(accel.z);
            console.write_line("");
        }
        
        // Read gyroscope data
        if let Ok(gyro) = mpu.read_gyro() {
            console.write_str("Gyro (deg/s): ");
            console.write_str("X=");
            console.write_float(gyro.x);
            console.write_str(" Y=");
            console.write_float(gyro.y);
            console.write_str(" Z=");
            console.write_float(gyro.z);
            console.write_line("");
        }
        
        // Delay between readings
        atmega128_firmware::hal::delay_ms(100);
    }
}
