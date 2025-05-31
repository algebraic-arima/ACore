//! Types related to task management
use core::cell::RefMut;
use core::fmt::Debug;

use super::TaskContext;
use super::pid::{KernelStack, PidHandle, pid_alloc};
use crate::config::{kernel_stack_position, PAGE_SIZE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::mm::{KERNEL_SPACE, MapPermission, MemorySet, PhysPageNum, VirtAddr, VirtPageNum};
use crate::sync::UPSafeCell;
use crate::trap::{TrapContext, trap_handler};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use log::warn;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub user_stack_mapped_va: VirtAddr,
    pub user_stack_bottom: VirtAddr,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    pub exit_code: i32,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
    pub fn new_page_for_stack(&mut self) {
        let new_user_stack_mapped_va = VirtAddr::from(self.user_stack_mapped_va.0 - PAGE_SIZE);
        assert!(new_user_stack_mapped_va.0 >= self.user_stack_bottom.0);
        self.memory_set.map_stack_page(new_user_stack_mapped_va);
        self.user_stack_mapped_va = new_user_stack_mapped_va;
        // warn!("user_stack_bottom: {:#x}, user_stack_mapped_va: {:#x}", self.user_stack_bottom.0, self.user_stack_mapped_va.0);
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        assert!(user_sp & 0x111 == 0);
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    user_stack_mapped_va: VirtAddr::from(user_sp),
                    user_stack_bottom: VirtAddr::from(user_sp - USER_STACK_SIZE),
                    parent: None,
                    children: Vec::new(),
                    fd_table: Vec::new(),
                    exit_code: 0,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    user_stack_mapped_va: parent_inner.user_stack_mapped_va,
                    user_stack_bottom: parent_inner.user_stack_bottom,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    fd_table: parent_inner.fd_table.clone(),
                    exit_code: 0,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- stop exclusively accessing parent/children PCB automatically
    }

    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** access inner exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // update user stack
        inner.user_stack_mapped_va = VirtAddr::from(user_sp);
        inner.user_stack_bottom = VirtAddr::from(user_sp - USER_STACK_SIZE);
        // update base size
        inner.base_size = user_sp;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** stop exclusively accessing inner automatically
    }

    // /// change the location of the program break. return None if failed.
    // pub fn change_program_brk(&mut self, size: i32) -> Option<usize> {
    //     let old_break = self.program_brk;
    //     let new_brk = self.program_brk as isize + size as isize;
    //     if new_brk < self.heap_bottom as isize {
    //         return None;
    //     }
    //     let result = if size < 0 {
    //         self.memory_set
    //             .shrink_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
    //     } else {
    //         self.memory_set
    //             .append_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
    //     };
    //     if result {
    //         self.program_brk = new_brk as usize;
    //         Some(old_break)
    //     } else {
    //         None
    //     }
    // }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
