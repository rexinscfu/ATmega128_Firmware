//! Real-time task scheduler implementation
#![no_std]

use core::cell::Cell;
use avr_device::atmega128::TC0;
use crate::hal::timer::Timer;

const MAX_TASKS: usize = 16;
const TICK_MS: u32 = 1;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Copy, Clone)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Suspended,
}

type TaskFunction = fn() -> TaskState;

pub struct Task {
    function: TaskFunction,
    priority: TaskPriority,
    period_ms: u32,
    next_run: u32,
    state: Cell<TaskState>,
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
        self.timer.init();
        self.timer.set_callback(Self::tick_handler);
        self.timer.start(TICK_MS);
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
        loop {
            self.dispatch_tasks();
            self.idle_task();
        }
    }

    fn dispatch_tasks(&mut self) {
        let current_time = self.current_time;
        
        for priority in [TaskPriority::Critical, TaskPriority::High, TaskPriority::Normal, TaskPriority::Low].iter() {
            for task_slot in self.tasks.iter() {
                if let Some(task) = task_slot {
                    if task.priority == *priority && 
                       task.state.get() != TaskState::Suspended &&
                       task.next_run <= current_time {
                        
                        task.state.set(TaskState::Running);
                        let new_state = (task.function)();
                        task.state.set(new_state);
                        
                        if new_state == TaskState::Ready {
                            let next_run = task.next_run + task.period_ms;
                            if next_run > current_time {
                                task.next_run = next_run;
                            } else {
                                task.next_run = current_time + task.period_ms;
                            }
                        }
                    }
                }
            }
        }
    }

    fn idle_task(&self) {
        avr_device::asm::sleep_mode();
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
