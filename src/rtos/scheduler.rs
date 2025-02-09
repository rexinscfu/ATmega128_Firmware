//! Real-time task scheduler implementation
#![no_std]

use super::task::{Task, TaskState, TaskControl};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use avr_device::atmega128::{TC0, interrupt};

const MAX_TASKS: usize = 16;
const TICK_MS: u32 = 1;

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
static SYSTEM_TICKS: AtomicU32 = AtomicU32::new(0);

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    current_task: Option<usize>,
    next_task: Option<usize>,
    timer: TC0,
    task_count: usize,
}

/*
struct TaskStatistics {
    total_runs: u32,
    total_runtime_us: u32,
    max_runtime_us: u32,
    min_runtime_us: u32,
    missed_deadlines: u32,
    stack_usage: u16,
}

struct TaskContext {
    stack_ptr: u16,
    program_counter: u16,
    status_reg: u8,
    registers: [u8; 32],
}
*/

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    current_time: u32,
    timer: Timer<TC0>,
    task_count: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: [None; MAX_TASKS],
            current_time: 0,
            timer: Timer::new(),
            task_count: 0,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.timer.tccr0.write(|w| w.bits(0x03));
            self.timer.ocr0.write(|w| w.bits(250));
            self.timer.timsk.write(|w| w.bits(0x02));
            
            interrupt::enable();
            SCHEDULER_RUNNING.store(true, Ordering::SeqCst);
        }
    }

    pub fn add_task(&mut self, function: TaskFunction, priority: TaskPriority, period_ms: u32) -> bool {
        if self.task_count >= MAX_TASKS {
            return false;
        }

        let task = Task {
            function,
            priority,
            period_ms,
            next_run: self.current_time,
            state: Cell::new(TaskState::Ready),
        };

        for slot in self.tasks.iter_mut() {
            if slot.is_none() {
                *slot = Some(task);
                self.task_count += 1;
                return true;
            }
        }
        false
    }

    pub fn remove_task(&mut self, function: TaskFunction) -> bool {
        for slot in self.tasks.iter_mut() {
            if let Some(task) = slot {
                if core::ptr::eq(task.function as *const (), function as *const ()) {
                    *slot = None;
                    self.task_count -= 1;
                    return true;
                }
            }
        }
        false
    }

    pub fn run(&mut self) -> ! {
        while SCHEDULER_RUNNING.load(Ordering::SeqCst) {
            if let Some(next) = self.schedule_next_task() {
                self.switch_task(next);
            } else {
                self.idle_task();
            }
        }
        loop {}
    }

    fn schedule_next_task(&mut self) -> Option<usize> {
        let current_time = SYSTEM_TICKS.load(Ordering::SeqCst);
        let mut highest_priority = None;
        
        for (idx, task_slot) in self.tasks.iter().enumerate() {
            if let Some(task) = task_slot {
                if task.control.state == TaskState::Ready {
                    match highest_priority {
                        None => highest_priority = Some(idx),
                        Some(current_highest) => {
                            if let Some(current_task) = &self.tasks[current_highest] {
                                if task.control.priority > current_task.control.priority {
                                    highest_priority = Some(idx);
                                }
                            }
                        }
                    }
                }
            }
        }
        highest_priority
    }

    fn idle_task(&self) {
        unsafe {
            avr_device::asm::sei();
            avr_device::asm::sleep_mode();
            avr_device::asm::cli();
        }
    }
    
    fn switch_task(&mut self, next_task: usize) {
        if let Some(current) = self.current_task {
            if let Some(task) = &mut self.tasks[current] {
                unsafe {
                    let sp: *mut u8;
                    core::arch::asm!("in {}, 0x3D", out(reg) sp);
                    task.save_context(sp);
                }
                task.control.state = TaskState::Ready;
            }
        }
        
        if let Some(task) = &mut self.tasks[next_task] {
            task.control.state = TaskState::Running;
            unsafe {
                let sp = task.get_stack_ptr();
                core::arch::asm!("out 0x3D, {}", in(reg) sp);
            }
        }
        
        self.current_task = Some(next_task);
    }

    extern "avr-interrupt" fn tick_handler() {
        unsafe {
            let scheduler = core::ptr::null_mut();
            (*scheduler).current_time += TICK_MS;
        }
    }
}

pub struct TaskBuilder {
    function: Option<TaskFunction>,
    priority: TaskPriority,
    period_ms: u32,
}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {
            function: None,
            priority: TaskPriority::Normal,
            period_ms: 1000,
        }
    }

    pub fn function(mut self, function: TaskFunction) -> Self {
        self.function = Some(function);
        self
    }

    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn period(mut self, period_ms: u32) -> Self {
        self.period_ms = period_ms;
        self
    }

    pub fn build(self, scheduler: &mut Scheduler) -> bool {
        if let Some(function) = self.function {
            scheduler.add_task(function, self.priority, self.period_ms)
        } else {
            false
        }
    }
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}
