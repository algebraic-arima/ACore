//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;
mod pid;
mod processor;
mod manager;

use crate::loader::get_app_data_by_name;
use crate::println;
use alloc::sync::Arc;
use lazy_static::*;
use processor::{schedule, take_current_task};
use task::{TaskControlBlock, TaskStatus};
pub use processor::*;
pub use manager::*;

pub use context::TaskContext;

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

#[unsafe(no_mangle)]
pub fn suspend_current_and_run_next(){
    let task = take_current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut inner.task_cx as *mut TaskContext;
    inner.task_status = TaskStatus::Ready;
    drop(inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub const IDLE_PID: usize = 0;

pub fn exit_current_and_run_next(exit_code: i32){
    // take from Processor
    let task = take_current_task().unwrap();
    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ stop exclusively accessing parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** stop exclusively accessing current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}
