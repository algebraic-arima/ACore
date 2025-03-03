mod context;

use crate::syscall::syscall;
use context::TrapContext;
use core::arch::global_asm;
use riscv::register::{
    mcause::{self, Exception, Trap},
    mtval, mtvec,
    mtvec::TrapMode,
};

global_asm!(include_str!("trap_m.S"));

pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        mtvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler_m(ctx: &mut TrapContext) -> &mut TrapContext {
    let mcause = mcause::read().cause();
    let mtval = mtval::read();
    match mcause {
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.mepc += 4;
            syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
            ctx.x[10] = 0;
        }
        _ => {
            panic!("Unhandled exception: {:?}, mtval: {:#x}", mcause, mtval);
        }
    }
    ctx
}
