mod context;

use crate::{config::*, sbi::shutdown};
pub use context::TrapContext;
use core::arch::{asm, global_asm};
use log::{error, info};
use riscv::register::{
    mcause::{self, Exception, Interrupt, Trap},
    mtval,
    mtvec::{self, TrapMode},
    scause, stval, time,
};

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
    let mcause = mcause::read().cause();
    let mtval = mtval::read();
    // info!("end: {}", time::read());
    error!("trap_handler_m: mcause: {:?}, mtval: {:#x}", mcause, mtval);
    error!(
        "trap_handler_m: scause: {:?}, stval: {:#x}",
        scause::read().cause(),
        stval::read()
    );
    
    match mcause {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            ctx.mepc += 4;
            error!("time interrupt at {}", time::read());
            unsafe {
                let mtimecmp_addr = (MTIMECMP as usize) as *mut u64;
                mtimecmp_addr.write_volatile(time::read() as u64 + TIME_INTERVAL);
            }
            ctx.x[10] = 0;
        }
        _ => {
            let mscratch:usize;
            let sp:usize;
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
