#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    testing::{TestRunner, TestCase, TestResult, TestError},
    hal::delay_ms,
};

struct MemoryTest;
impl TestCase for MemoryTest {
    fn name(&self) -> &'static str {
        "Memory Access"
    }

    fn run(&self) -> TestResult {
        let test_patterns = [0x55, 0xAA, 0x33, 0xCC];
        let test_addr = 0x100 as *mut u8;

        for &pattern in test_patterns.iter() {
            unsafe {
                core::ptr::write_volatile(test_addr, pattern);
                let readback = core::ptr::read_volatile(test_addr);
                if readback != pattern {
                    return TestResult::Fail(TestError::HardwareFault);
                }
            }
        }

        TestResult::Pass
    }
}

struct PwmTest;
impl TestCase for PwmTest {
    fn name(&self) -> &'static str {
        "PWM Generation"
    }

    fn run(&self) -> TestResult {
        unsafe {
            let timer = &(*avr_device::atmega128::TC1::ptr());
            
            timer.tccr1a.write(|w| w.bits(0x82));
            timer.tccr1b.write(|w| w.bits(0x19));
            timer.ocr1a.write(|w| w.bits(0x7FFF));
            
            delay_ms(100);
            
            let capture = timer.icr1.read().bits();
            if capture == 0 || capture == 0xFFFF {
                return TestResult::Fail(TestError::HardwareFault);
            }
        }

        TestResult::Pass
    }
}

#[avr_device::entry]
fn main() -> ! {
    let mut runner = TestRunner::new();
    
    let hardware_tests = [
        &atmega128_firmware::testing::UartTest,
        &atmega128_firmware::testing::AdcTest,
        &atmega128_firmware::testing::TimerTest,
        &atmega128_firmware::testing::SpiTest,
    ];
    
    let peripheral_tests = [
        &MemoryTest,
        &PwmTest,
    ];
    
    runner.run_suite("Hardware Tests", &hardware_tests);
    delay_ms(1000);
    
    runner.run_suite("Peripheral Tests", &peripheral_tests);
    
    loop {
        delay_ms(1000);
    }
}
