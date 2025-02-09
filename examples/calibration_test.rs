#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    drivers::{Calibration, Mpu6050, SerialConsole},
    hal::{Spi, Twi, TwiSpeed, delay_ms},
};

#[avr_device::entry]
fn main() -> ! {
    let mut console = SerialConsole::new();
    console.write_line("Starting sensor calibration...");
    
    let mut twi = Twi::new();
    twi.set_speed(TwiSpeed::Fast400k);
    
    let mut imu = match Mpu6050::new(twi) {
        Ok(imu) => imu,
        Err(_) => {
            console.write_line("Failed to initialize IMU!");
            loop {}
        }
    };
    
    let spi = Spi::new();
    let flash = match Flash::new(spi, 0, 1, 2) {
        Ok(flash) => flash,
        Err(_) => {
            console.write_line("Failed to initialize Flash!");
            loop {}
        }
    };
    
    let mut calibration = Calibration::new(flash);
    
    console.write_line("Place the sensor on a level surface");
    delay_ms(5000);
    
    console.write_line("Calibrating gyroscope...");
    if let Err(_) = calibration.calibrate_gyro(&mut imu) {
        console.write_line("Gyro calibration failed!");
    }
    
    console.write_line("Calibrating accelerometer...");
    console.write_line("Rotate the sensor through all orientations");
    delay_ms(5000);
    
    if let Err(_) = calibration.calibrate_accel(&mut imu) {
        console.write_line("Accelerometer calibration failed!");
    }
    
    console.write_line("Saving calibration data...");
    if let Err(_) = calibration.save_calibration() {
        console.write_line("Failed to save calibration!");
    }
    
    console.write_line("Calibration complete!");
    
    loop {
        if let Ok(raw_gyro) = imu.read_gyro() {
            let cal_gyro = calibration.apply_gyro_calibration(raw_gyro);
            console.write_str("Gyro: ");
            console.write_float(cal_gyro.x);
            console.write_str(", ");
            console.write_float(cal_gyro.y);
            console.write_str(", ");
            console.write_float(cal_gyro.z);
            console.write_line("");
        }
        
        if let Ok(raw_accel) = imu.read_accel() {
            let cal_accel = calibration.apply_accel_calibration(raw_accel);
            console.write_str("Accel: ");
            console.write_float(cal_accel.x);
            console.write_str(", ");
            console.write_float(cal_accel.y);
            console.write_str(", ");
            console.write_float(cal_accel.z);
            console.write_line("");
        }
        
        delay_ms(100);
    }
}
