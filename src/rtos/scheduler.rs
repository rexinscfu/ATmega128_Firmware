//! Real-time task scheduler implementation
#![no_std]

use super::task::{Task, TaskState, TaskControl};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use avr_device::atmega128::{TC0, interrupt};

const MAX_TASKS: usize = 16;
const TICK_MS: u32 = 1;

static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
static SYSTEM_TICKS: AtomicU32 = AtomicU32::new(0);

#[derive(Copy, Clone)]
struct TaskStatistics {
    total_runs: u32,
    total_runtime_us: u32,
    max_runtime_us: u32,
    min_runtime_us: u32,
    missed_deadlines: u32,
    stack_usage: u16,
}

#[derive(Copy, Clone)]
struct TaskContext {
    stack_ptr: u16,
    program_counter: u16,
    status_reg: u8,
    registers: [u8; 32],
}

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    current_task: Option<usize>,
    next_task: Option<usize>,
    timer: TC0,
    task_count: usize,
    statistics: [TaskStatistics; MAX_TASKS],
    contexts: [TaskContext; MAX_TASKS],
    idle_task_index: Option<usize>,
}

impl Scheduler {
    pub fn new(timer: TC0) -> Self {
        Self {
            tasks: [None; MAX_TASKS],
            current_task: None,
            next_task: None,
            timer,
            task_count: 0,
            statistics: [TaskStatistics {
                total_runs: 0,
                total_runtime_us: 0,
                max_runtime_us: 0,
                min_runtime_us: u32::MAX,
                missed_deadlines: 0,
                stack_usage: 0,
            }; MAX_TASKS],
            contexts: [TaskContext {
                stack_ptr: 0,
                program_counter: 0,
                status_reg: 0,
                registers: [0; 32],
            }; MAX_TASKS],
            idle_task_index: None,
        }
    }

    pub fn init(&mut self) {
        // Configure Timer0 for 1ms ticks
        self.timer.tccr0.write(|w| unsafe { 
            w.cs0().bits(0b011) // Prescaler 64
             .wgm0().bits(0b10) // CTC mode
        });
        self.timer.ocr0.write(|w| unsafe { w.bits(250) }); // 16MHz/64/250 = 1kHz
        self.timer.timsk.modify(|_, w| w.ocie0().set_bit());

        // Create and add idle task
        self.idle_task_index = self.add_task(Scheduler::idle_task, TaskPriority::Idle, 0);
        
        SCHEDULER_RUNNING.store(true, Ordering::SeqCst);
    }

    pub fn add_task(&mut self, function: fn() -> !, priority: TaskPriority, period_ms: u32) -> Option<usize> {
        if self.task_count >= MAX_TASKS {
            return None;
        }

        let task = Task::new(priority.into(), "task", function);
        
        // Find empty slot
        for (i, slot) in self.tasks.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(task);
                self.task_count += 1;
                return Some(i);
            }
        }
        None
    }

    pub fn run(&mut self) -> ! {
        unsafe { avr_device::interrupt::enable() };
        
        loop {
            if let Some(next) = self.schedule_next_task() {
                self.switch_task(next);
            }
            
            // Wait for next tick
            avr_device::asm::sleep();
        }
    }

    fn schedule_next_task(&mut self) -> Option<usize> {
        let mut highest_priority = TaskPriority::Idle;
        let mut selected_task = self.idle_task_index;

        for (i, task) in self.tasks.iter().enumerate() {
            if let Some(task) = task {
                if task.control.state == TaskState::Ready && 
                   task.control.priority > highest_priority.into() {
                    highest_priority = task.control.priority.into();
                    selected_task = Some(i);
                }
            }
        }

        selected_task
    }

    fn switch_task(&mut self, next_task: usize) {
        if let Some(current) = self.current_task {
            // Save current task context
            self.save_context(current);
            
            // Update statistics
            self.update_task_statistics(current);
        }

        // Load next task context
        self.load_context(next_task);
        
        self.current_task = Some(next_task);
        self.tasks[next_task].as_mut().unwrap().control.state = TaskState::Running;
    }

    fn save_context(&mut self, task_index: usize) {
        unsafe {
            // Save registers
            core::arch::asm!(
                "push r0",
                "push r1",
                // ... save all registers
                "in r0, 0x3F", // Save SREG
                "push r0",
            );
            
            // Save stack pointer
            let sp: u16;
            core::arch::asm!("in {}, 0x3D", "in {{}, 0x3E}", out(reg) sp);
            self.contexts[task_index].stack_ptr = sp;
        }
    }

    fn load_context(&mut self, task_index: usize) {
        unsafe {
            // Restore stack pointer
            let sp = self.contexts[task_index].stack_ptr;
            core::arch::asm!("out 0x3D, {}", "out 0x3E, {}", in(reg) sp);
            
            // Restore registers
            core::arch::asm!(
                "pop r0",
                "out 0x3F, r0", // Restore SREG
                "pop r1",
                "pop r0",
                // ... restore all registers
            );
        }
    }

    fn update_task_statistics(&mut self, task_index: usize) {
        let stats = &mut self.statistics[task_index];
        stats.total_runs += 1;
        
        let task = &self.tasks[task_index].as_ref().unwrap();
        stats.stack_usage = task.get_stack_usage() as u16;
        
        // TODO: Implement runtime measurement when hardware timer available
    }

    fn idle_task() -> ! {
        loop {
            unsafe { avr_device::asm::sleep() };
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum TaskPriority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

pub struct TaskBuilder {
    function: Option<fn() -> !>,
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

    pub fn function(mut self, function: fn() -> !) -> Self {
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
            scheduler.add_task(function, self.priority, self.period_ms).is_some()
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
