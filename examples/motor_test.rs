#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    drivers::{MotorController, PidConfig},
    hal::{PwmChannel, delay_ms},
};

#[avr_device::entry]
fn main() -> ! {
    // Initialize motor controller
    let mut motor = MotorController::new(PwmChannel::Timer1A);
    
    // Configure PID
    let config = PidConfig {
        kp: 2.0,
        ki: 0.5,
        kd: 0.1,
        output_min: 0.0,
        output_max: 100.0,
        iterm_min: -25.0,
        iterm_max: 25.0,
        sample_time_ms: 10,
    };
    motor.configure(config);
    
    // Enable motor
    motor.set_enabled(true);
    
    /*
    // Motion profile parameters
    const STEPS: &[(f32, u16)] = &[
        (25.0, 1000),  // 25% speed for 1s
        (50.0, 2000),  // 50% speed for 2s
        (75.0, 1000),  // 75% speed for 1s
        (100.0, 500),  // 100% speed for 0.5s
        (0.0, 1000),   // Stop for 1s
    ];
    */
    
    // Simple speed ramp test
    let mut speed = 0.0f32;
    let mut increasing = true;
    
    loop {
        // Update speed
        if increasing {
            speed += 0.1;
            if speed >= 100.0 {
                speed = 100.0;
                increasing = false;
            }
        } else {
            speed -= 0.1;
            if speed <= 0.0 {
                speed = 0.0;
                increasing = true;
            }
        }
        
        // Set target speed
        motor.set_target(speed);
        
        // Simulate feedback 
        let feedback = speed * 0.95;
        motor.update(feedback);
        
        delay_ms(10);
    }
}
