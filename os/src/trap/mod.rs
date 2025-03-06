mod context;

use context::TrapContext;
use core::arch::global_asm;
use log::info;
use riscv::register::{
    mcause::{self, Exception, Interrupt, Trap},
    mtval,
    mtvec::{self, TrapMode},
    time,
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
    info!("trap_handler_m: mcause: {:?}, mtval: {:#x}", mcause, mtval);
    match mcause {
        Trap::Interrupt(Interrupt::MachineTimer) => {
            ctx.mepc += 4;
            // todo: handle timer interrupt
            ctx.x[10] = 0;
        }
        _ => {
            panic!("Unhandled exception: {:?}, mtval: {:#x}", mcause, mtval);
        }
    }
    unsafe extern "C" {
        safe fn __restore_m(ctx_addr: usize);
    }
    __restore_m(ctx as *mut TrapContext as usize);
}
