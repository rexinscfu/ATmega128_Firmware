#![no_std]
#![no_main]

use panic_halt as _;
use atmega128_firmware::{
    rtos::{Scheduler, TaskBuilder, TaskPriority, TaskState},
    drivers::SerialConsole,
    hal::delay_ms,
};

static mut CONSOLE: Option<SerialConsole> = None;

#[avr_device::entry]
fn main() -> ! {
    unsafe {
        CONSOLE = Some(SerialConsole::new());
    }
    
    let mut scheduler = Scheduler::new();
    scheduler.init();
    
    TaskBuilder::new()
        .function(task_led)
        .priority(TaskPriority::Low)
        .period(500)
        .build(&mut scheduler);
        
    TaskBuilder::new()
        .function(task_sensor)
        .priority(TaskPriority::Normal)
        .period(100)
        .build(&mut scheduler);
        
    TaskBuilder::new()
        .function(task_monitor)
        .priority(TaskPriority::High)
        .period(1000)
        .build(&mut scheduler);
    
    scheduler.run()
}

fn task_led() -> TaskState {
    static mut LED_STATE: bool = false;
    
    unsafe {
        LED_STATE = !LED_STATE;
        if let Some(console) = &mut CONSOLE {
            console.write_str("LED: ");
            console.write_line(if LED_STATE { "ON" } else { "OFF" });
        }
    }
    
    TaskState::Ready
}

fn task_sensor() -> TaskState {
    unsafe {
        if let Some(console) = &mut CONSOLE {
            console.write_line("Reading sensor...");
        }
    }
    delay_ms(50);
    TaskState::Ready
}

fn task_monitor() -> TaskState {
    unsafe {
        if let Some(console) = &mut CONSOLE {
            console.write_line("System monitoring...");
        }
    }
    TaskState::Ready
}
