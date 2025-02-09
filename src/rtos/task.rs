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

        task.control.stack_base = task.stack.as_mut_ptr();
        task.control.stack_ptr = task.init_stack(entry);
        task
    }

    fn init_stack(&mut self, entry: TaskFunction) -> StackPtr {
        let mut sp = self.stack.as_mut_ptr().wrapping_add(self.stack.len());

        unsafe {
            sp = sp.wrapping_sub(core::mem::size_of::<usize>());
            *(sp as *mut usize) = entry as usize;

            sp = sp.wrapping_sub(32);
            *(sp as *mut u32) = 0;
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
        let mut used = 0;
        let base = self.control.stack_base as usize;
        
        for i in 0..self.control.stack_size {
            if unsafe { *self.control.stack_base.wrapping_add(i) } != 0 {
                used = self.control.stack_size - i;
                break;
            }
        }
        used
    }

    pub fn suspend(&mut self) {
        self.control.state = TaskState::Suspended;
    }

    pub fn resume(&mut self) {
        if self.control.state == TaskState::Suspended {
            self.control.state = TaskState::Ready;
        }
    }

    pub fn block(&mut self) {
        self.control.state = TaskState::Blocked;
    }

    pub fn unblock(&mut self) {
        if self.control.state == TaskState::Blocked {
            self.control.state = TaskState::Ready;
        }
    }
}
