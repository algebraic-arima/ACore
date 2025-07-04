mod context;
pub use context::TrapContext;
use log::info;
use log::warn;
use riscv::register::mcause;
use riscv::register::mtval;
use riscv::register::sie;
use riscv::register::time;

use crate::{config::*, println, syscall::syscall, task::*, timer::*};
use core::arch::asm;
use core::arch::global_asm;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval,
    stvec::{self, TrapMode},
};

global_asm!(include_str!("trap_s.S"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            let stval = stval::read();
            let mapped_stack = current_user_mapped_stack();
            let user_stack_bottom = current_user_stack_bottom();
            // check if the faulting address is in the mapped stack
            if stval >= mapped_stack || stval < user_stack_bottom {
                info!(
                    "[kernel] Illegal memory access in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                    stval, cx.sepc
                );
                exit_current_and_run_next(-2);
            } else {
                info!(
                    "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, new page mapped in stack.",
                    stval, cx.sepc
                );
                expand_user_stack();
                suspend_current_and_run_next(); // by the way schedule
            }
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            // SSI is used for machine timer interrupt
            // info!("Supervisor Timer Interrupt at {}", time::read());
            // set_next_trigger();
            unsafe {
                let sip = sie::read().bits();
                asm!("csrw sip, {sip}", sip = in(reg) sip ^ 2);
            }
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[unsafe(no_mangle)]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    unsafe extern "C" {
        unsafe fn __alltraps();
        unsafe fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    // info!("trap_return");
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}
#[unsafe(no_mangle)]
/// Todo: Chapter 9: I/O device
pub fn trap_from_kernel() -> ! {
    let c = scause::read().cause();
    let t = stval::read();
    panic!(
        "[kernel] Unsupported trap from kernel: {:?}, stval = {:#x}!",
        c, t
    );
}
