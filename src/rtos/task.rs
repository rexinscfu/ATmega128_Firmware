#![no_std]

use core::sync::atomic::{AtomicU8, Ordering};

static NEXT_TASK_ID: AtomicU8 = AtomicU8::new(0);

pub type TaskFunction = fn() -> !;
pub type StackPtr = *mut u8;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Suspended,
}

#[derive(Copy, Clone)]
pub struct TaskControl {
    pub id: u8,
    pub stack_ptr: StackPtr,
    pub stack_base: StackPtr,
    pub stack_size: usize,
    pub state: TaskState,
    pub priority: u8,
    pub name: &'static str,
}

pub struct Task {
    pub control: TaskControl,
    stack: [u8; 512],
}

impl Task {
    pub fn new(
        priority: u8,
        name: &'static str,
        entry: TaskFunction,
    ) -> Self {
        let mut task = Task {
            control: TaskControl {
                id: NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed),
                stack_ptr: core::ptr::null_mut(),
                stack_base: core::ptr::null_mut(),
                stack_size: 512,
                state: TaskState::Ready,
                priority,
                name,
            },
            stack: [0; 512],
        };

        // Initialize stack
        let stack_top = task.init_stack(entry);
        task.control.stack_ptr = stack_top;
        task.control.stack_base = task.stack.as_mut_ptr();
        
        task
    }

    fn init_stack(&mut self, entry: TaskFunction) -> StackPtr {
        let mut sp = unsafe {
            self.stack.as_mut_ptr().add(self.stack.len())
        };

        // Push initial context
        unsafe {
            // Program counter (entry point)
            sp = sp.sub(1);
            *(sp as *mut u8) = (entry as u16 & 0xFF) as u8;
            sp = sp.sub(1);
            *(sp as *mut u8) = ((entry as u16 >> 8) & 0xFF) as u8;

            // Status register (interrupts enabled)
            sp = sp.sub(1);
            *(sp as *mut u8) = 0x80;

            // General purpose registers
            for _ in 0..32 {
                sp = sp.sub(1);
                *(sp as *mut u8) = 0;
            }
        }

        sp
    }

    pub fn save_context(&mut self, sp: StackPtr) {
        self.control.stack_ptr = sp;
    }

    pub fn get_stack_ptr(&self) -> StackPtr {
        self.control.stack_ptr
    }

    pub fn get_stack_usage(&self) -> usize {
        let mut unused = 0;
        for &byte in self.stack.iter() {
            if byte == 0 {
                unused += 1;
            }
        }
        self.stack.len() - unused
    }

    pub fn suspend(&mut self) {
        if self.control.state == TaskState::Running {
            self.control.state = TaskState::Suspended;
        }
    }

    pub fn resume(&mut self) {
        if self.control.state == TaskState::Suspended {
            self.control.state = TaskState::Ready;
        }
    }

    pub fn block(&mut self) {
        if self.control.state == TaskState::Running {
            self.control.state = TaskState::Blocked;
        }
    }

    pub fn unblock(&mut self) {
        if self.control.state == TaskState::Blocked {
            self.control.state = TaskState::Ready;
        }
    }
}
