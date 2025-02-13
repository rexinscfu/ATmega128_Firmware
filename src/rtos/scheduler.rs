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

#[derive(Debug)]
pub enum SchedulerError {
    TaskLimitReached,
    TaskNotFound,
    InvalidPriority,
    AlreadyRunning,
    EventQueueFull,
    Timeout,
    NoSemaphoresAvailable,
    InvalidSemaphore,
    SemaphoreLocked,
}

pub type Result<T> = core::result::Result<T, SchedulerError>;

#[derive(Copy, Clone, PartialEq)]
pub enum EventType {
    Timer,
    Gpio,
    Uart,
    Adc,
    Custom(u8),
}

#[derive(Copy, Clone)]
pub struct Event {
    event_type: EventType,
    data: u32,
    timestamp: u32,
}

pub struct EventQueue {
    events: [Option<Event>; 32],
    head: usize,
    tail: usize,
}

impl EventQueue {
    pub const fn new() -> Self {
        Self {
            events: [None; 32],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, event: Event) -> bool {
        let next = (self.tail + 1) % self.events.len();
        if next != self.head {
            self.events[self.tail] = Some(event);
            self.tail = next;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<Event> {
        if self.head != self.tail {
            let event = self.events[self.head].take();
            self.head = (self.head + 1) % self.events.len();
            event
        } else {
            None
        }
    }
}

pub struct Semaphore {
    count: AtomicU8,
    waiting_tasks: [Option<usize>; MAX_TASKS],
    num_waiting: usize,
}

impl Semaphore {
    pub const fn new(initial: u8) -> Self {
        Self {
            count: AtomicU8::new(initial),
            waiting_tasks: [None; MAX_TASKS],
            num_waiting: 0,
        }
    }

    pub fn acquire(&mut self) -> bool {
        loop {
            let current = self.count.load(Ordering::Relaxed);
            if current > 0 {
                if self.count.compare_exchange(
                    current,
                    current - 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ).is_ok() {
                    return true;
                }
            } else {
                return false;
            }
        }
    }

    pub fn release(&mut self) {
        self.count.fetch_add(1, Ordering::Release);
    }
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
    event_queue: EventQueue,
    semaphores: [Semaphore; 8],
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
            event_queue: EventQueue::new(),
            semaphores: [Semaphore::new(0); 8],
        }
    }

    pub fn init(&mut self) -> Result<()> {
        if SCHEDULER_RUNNING.load(Ordering::SeqCst) {
            return Err(SchedulerError::AlreadyRunning);
        }

        // Configure Timer0 for 1ms ticks
        self.timer.tccr0.write(|w| unsafe { 
            w.cs0().bits(0b011) // Prescaler 64
             .wgm0().bits(0b10) // CTC mode
        });
        self.timer.ocr0.write(|w| unsafe { w.bits(250) }); // 16MHz/64/250 = 1kHz
        self.timer.timsk.modify(|_, w| w.ocie0().set_bit());

        // Create and add idle task
        self.idle_task_index = self.add_task(Self::idle_task, TaskPriority::Idle, 0)
            .ok_or(SchedulerError::TaskLimitReached)?;
        
        SCHEDULER_RUNNING.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn add_task(&mut self, function: fn() -> !, priority: TaskPriority, period_ms: u32) -> Result<usize> {
        if self.task_count >= MAX_TASKS {
            return Err(SchedulerError::TaskLimitReached);
        }

        let task = Task::new(priority.into(), "task", function);
        
        // Find empty slot
        for (i, slot) in self.tasks.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(task);
                self.task_count += 1;
                return Ok(i);
            }
        }
        Err(SchedulerError::TaskLimitReached)
    }

    pub fn remove_task(&mut self, task_id: usize) -> Result<()> {
        if task_id >= MAX_TASKS || self.tasks[task_id].is_none() {
            return Err(SchedulerError::TaskNotFound);
        }

        self.tasks[task_id] = None;
        self.task_count -= 1;
        Ok(())
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

    pub fn post_event(&mut self, event_type: EventType, data: u32) -> Result<()> {
        let event = Event {
            event_type,
            data,
            timestamp: SYSTEM_TICKS.load(Ordering::Relaxed),
        };

        if !self.event_queue.push(event) {
            return Err(SchedulerError::EventQueueFull);
        }

        // Wake up tasks waiting for events
        self.wake_event_tasks(event_type);
        Ok(())
    }

    pub fn wait_for_event(&mut self, event_type: EventType, timeout_ms: u32) -> Result<Event> {
        let deadline = SYSTEM_TICKS.load(Ordering::Relaxed) + timeout_ms;
        
        loop {
            if let Some(event) = self.event_queue.pop() {
                if event.event_type == event_type {
                    return Ok(event);
                }
            }

            if SYSTEM_TICKS.load(Ordering::Relaxed) >= deadline {
                return Err(SchedulerError::Timeout);
            }

            // Put current task to sleep
            if let Some(current) = self.current_task {
                self.tasks[current].as_mut().unwrap().control.state = TaskState::Blocked;
            }

            // Switch to next task
            if let Some(next) = self.schedule_next_task() {
                self.switch_task(next);
            }
        }
    }

    fn wake_event_tasks(&mut self, event_type: EventType) {
        for task in self.tasks.iter_mut().flatten() {
            if task.control.state == TaskState::Blocked {
                task.control.state = TaskState::Ready;
            }
        }
    }

    pub fn create_semaphore(&mut self, initial: u8) -> Result<usize> {
        for (i, sem) in self.semaphores.iter_mut().enumerate() {
            if sem.count.load(Ordering::Relaxed) == 0 {
                *sem = Semaphore::new(initial);
                return Ok(i);
            }
        }
        Err(SchedulerError::NoSemaphoresAvailable)
    }

    pub fn semaphore_acquire(&mut self, sem_id: usize) -> Result<()> {
        if sem_id >= self.semaphores.len() {
            return Err(SchedulerError::InvalidSemaphore);
        }

        if !self.semaphores[sem_id].acquire() {
            if let Some(current) = self.current_task {
                self.tasks[current].as_mut().unwrap().control.state = TaskState::Blocked;
            }
            return Err(SchedulerError::SemaphoreLocked);
        }

        Ok(())
    }

    pub fn semaphore_release(&mut self, sem_id: usize) -> Result<()> {
        if sem_id >= self.semaphores.len() {
            return Err(SchedulerError::InvalidSemaphore);
        }

        self.semaphores[sem_id].release();
        Ok(())
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
            scheduler.add_task(function, self.priority, self.period_ms).is_ok()
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
