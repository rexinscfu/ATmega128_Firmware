#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    hal::{Power, PowerConfig, SleepMode, ClockDivider, Peripheral},
    drivers::SerialConsole,
};

#[avr_device::entry]
fn main() -> ! {
    let mut console = SerialConsole::new();
    let mut power = Power::new();

    let config = PowerConfig {
        sleep_mode: SleepMode::PowerSave,
        clock_divider: ClockDivider::Div8,
        brown_out_enabled: true,
        watchdog_enabled: true,
    };
    power.configure(config);

    power.disable_peripheral(Peripheral::Spi);
    power.disable_peripheral(Peripheral::Timer2);
    power.disable_peripheral(Peripheral::Timer3);

    loop {
        let voltage = power.get_voltage();
        console.write_str("Battery voltage: ");
        console.write_float(voltage);
        console.write_line("V");

        if power.is_low_battery() {
            console.write_line("Warning: Low battery!");
            power.set_clock_divider(ClockDivider::Div16);
        }

        power.enter_sleep_mode();
    }
}
