mod context;
mod timer;

use crate::{config::*, sbi::shutdown};
pub use context::TrapContext;
use core::arch::{asm, global_asm};
use log::{error, info};
use riscv::register::{
    mcause::{self, Exception, Interrupt, Trap},
    mip, mtval,
    mtvec::{self, TrapMode},
    scause, sip, stval, time,
};
pub use timer::*;

global_asm!(include_str!("trap_m.S"));

pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps_m();
    }
    unsafe {
        mtvec::write(__alltraps_m as usize, TrapMode::Direct);
        // set mtvec the address of __alltraps_m to answer the ecall from S-mode
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler_m(ctx: &mut TrapContext) {
    let mip = mip::read();
    let mcause = mcause::read().cause();
    let mtval = mtval::read();
    // info!("end: {}", time::read());
    // error!(
    //     "trap_handler_m: mip: {:?}, mcause: {:?}, mtval: {:#x}",
    //     mip, mcause, mtval
    // );

    match mcause {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            info!("Machine Timer Interrupt at {}", time::read());
            set_next_trigger(0);
            unsafe {
                asm!("csrw sip, 2");
            }
        }
        _ => {
            let mscratch: usize;
            let sp: usize;
            unsafe {
                asm!(
                    "csrr {0}, mscratch",
                    "mv {1}, sp",
                    out(reg) mscratch,
                    out(reg) sp,
                    options(nostack)
                );
            }

            error!("mscratch: {:#x}, sp: {:#x}", mscratch, sp);
            panic!("Unhandled exception: {:?}, mtval: {:#x}", mcause, mtval);
        }
    }
    unsafe extern "C" {
        safe fn __restore_m(ctx_addr: usize);
    }

    __restore_m(ctx as *mut TrapContext as usize);
}
