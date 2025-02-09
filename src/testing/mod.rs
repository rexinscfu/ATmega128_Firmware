#![no_std]

use crate::drivers::SerialConsole;
use core::fmt::Write;

pub struct TestRunner {
    console: SerialConsole,
    total_tests: u32,
    passed_tests: u32,
    current_suite: &'static str,
}

pub trait TestCase {
    fn run(&self) -> TestResult;
    fn name(&self) -> &'static str;
}

#[derive(PartialEq)]
pub enum TestResult {
    Pass,
    Fail(TestError),
}

#[derive(PartialEq)]
pub enum TestError {
    AssertionFailed(&'static str),
    Timeout,
    HardwareFault,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            console: SerialConsole::new(),
            total_tests: 0,
            passed_tests: 0,
            current_suite: "",
        }
    }

    pub fn run_suite(&mut self, name: &'static str, tests: &[&dyn TestCase]) {
        self.current_suite = name;
        self.console.write_fmt(format_args!("\n=== Test Suite: {} ===\n", name)).ok();

        for test in tests {
            self.total_tests += 1;
            self.console.write_fmt(format_args!("Running {}: ", test.name())).ok();
            
            match test.run() {
                TestResult::Pass => {
                    self.passed_tests += 1;
                    self.console.write_str("PASS\n").ok();
                }
                TestResult::Fail(err) => {
                    self.console.write_fmt(format_args!("FAIL - {:?}\n", err)).ok();
                }
            }
        }

        self.print_summary();
    }

    fn print_summary(&mut self) {
        self.console.write_fmt(format_args!(
            "\nTest Summary for {}:\n", 
            self.current_suite
        )).ok();
        
        self.console.write_fmt(format_args!(
            "Passed: {}/{} ({}%)\n",
            self.passed_tests,
            self.total_tests,
            (self.passed_tests * 100) / self.total_tests
        )).ok();
    }
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return TestResult::Fail(TestError::AssertionFailed(
                concat!(
                    "assertion failed: `left == right`\n",
                    "  left: `", stringify!($left), "`\n",
                    "  right: `", stringify!($right), "`"
                )
            ));
        }
    };
}

#[macro_export]
macro_rules! assert_within {
    ($value:expr, $target:expr, $tolerance:expr) => {
        if ($value - $target).abs() > $tolerance {
            return TestResult::Fail(TestError::AssertionFailed(
                concat!(
                    "assertion failed: `value within tolerance of target`\n",
                    "  value: `", stringify!($value), "`\n",
                    "  target: `", stringify!($target), "`\n",
                    "  tolerance: `", stringify!($tolerance), "`"
                )
            ));
        }
    };
}

#[macro_export]
macro_rules! assert_timeout {
    ($cond:expr, $timeout:expr) => {
        let mut timeout = $timeout;
        while !$cond {
            if timeout == 0 {
                return TestResult::Fail(TestError::Timeout);
            }
            timeout -= 1;
            crate::hal::delay_ms(1);
        }
    };
}

pub struct UartTest;
impl TestCase for UartTest {
    fn name(&self) -> &'static str {
        "UART Communication"
    }

    fn run(&self) -> TestResult {
        let mut uart = crate::hal::uart::Uart::new();
        let test_byte = 0x55;

        uart.write_byte(test_byte);
        assert_timeout!(uart.is_rx_ready(), 1000);
        
        match uart.read_byte() {
            Some(byte) => assert_eq!(byte, test_byte),
            None => return TestResult::Fail(TestError::HardwareFault),
        }

        TestResult::Pass
    }
}

pub struct AdcTest;
impl TestCase for AdcTest {
    fn name(&self) -> &'static str {
        "ADC Functionality"
    }

    fn run(&self) -> TestResult {
        unsafe {
            let adc = &(*avr_device::atmega128::ADC::ptr());
            
            adc.admux.write(|w| w.bits(0x40));
            adc.adcsra.write(|w| w.bits(0x87));
            
            assert_timeout!(adc.adcsra.read().bits() & 0x10 != 0, 1000);
            
            let value = adc.adcl.read().bits() as u16 | 
                      ((adc.adch.read().bits() as u16) << 8);
            
            assert_within!(value, 512, 100);
        }

        TestResult::Pass
    }
}

pub struct TimerTest;
impl TestCase for TimerTest {
    fn name(&self) -> &'static str {
        "Timer Operation"
    }

    fn run(&self) -> TestResult {
        unsafe {
            let timer = &(*avr_device::atmega128::TC0::ptr());
            
            timer.tccr0.write(|w| w.bits(0x01));
            timer.tcnt0.write(|w| w.bits(0x00));
            
            let start = timer.tcnt0.read().bits();
            crate::hal::delay_ms(1);
            let end = timer.tcnt0.read().bits();
            
            assert_eq!(end > start, true);
        }

        TestResult::Pass
    }
}

pub struct SpiTest;
impl TestCase for SpiTest {
    fn name(&self) -> &'static str {
        "SPI Transfer"
    }

    fn run(&self) -> TestResult {
        let mut spi = crate::hal::spi::Spi::new();
        let test_byte = 0xA5;

        spi.transfer(test_byte);
        assert_timeout!(spi.is_rx_ready(), 1000);
        
        match spi.read() {
            Ok(byte) => assert_eq!(byte, test_byte),
            Err(_) => return TestResult::Fail(TestError::HardwareFault),
        }

        TestResult::Pass
    }
}
