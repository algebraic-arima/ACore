use core::arch::asm;

use crate::init::switch_s;
use crate::trap::TrapContext;
use log::info;

const MACHINE_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;

const KERNEL_BASE_ADDRESS: usize = 0x80200000;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct MachineStack {
    data: [u8; MACHINE_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

static MACHINE_STACK: MachineStack = MachineStack {
    data: [0; MACHINE_STACK_SIZE],
};

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

impl MachineStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + MACHINE_STACK_SIZE
    }
    pub fn push_context(&self, trap_ctx: TrapContext) -> &'static mut TrapContext {
        let trap_ctx_ptr =
            (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = trap_ctx;
        }
        unsafe { trap_ctx_ptr.as_mut().unwrap() }
    }
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

pub fn load_kernel() {
    unsafe extern "C" {
        safe fn kernel_start();
        safe fn kernel_end();
    }

    let kernel_start_ptr = kernel_start as usize as *const usize;
    let kernel_end_ptr = kernel_end as usize as *const usize;
    let kernel_size = kernel_end_ptr as usize - kernel_start_ptr as usize;
    info!("[kernel] kernel asm size: {:#x}", kernel_size);

    (KERNEL_BASE_ADDRESS..KERNEL_BASE_ADDRESS + kernel_size)
        .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });

    let src = unsafe { core::slice::from_raw_parts(kernel_start_ptr as *const u8, kernel_size) };
    let dst =
        unsafe { core::slice::from_raw_parts_mut(KERNEL_BASE_ADDRESS as *mut u8, kernel_size) };
    dst.copy_from_slice(src);

    unsafe {
        asm!("fence.i");
    }
}

pub fn run_kernel() {
    switch_s(0x80200000, 0);
    
    unsafe extern "C" {
        safe fn __restore_m(ctx_addr: usize);
    }
    __restore_m(MACHINE_STACK.push_context(TrapContext::kernel_init_context(
        0x80200000,
        KERNEL_STACK.get_sp(),
    )) as *const _ as usize);
}
