mod context;

use crate::syscall::syscall;
use context::TrapContext;
use core::arch::global_asm;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval,
    stvec::{self, TrapMode},
};

global_asm!(include_str!("trap_m.S"));

pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
        // set mtvec the address of __alltraps to answer the ecall from S-mode
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler_m(ctx: &mut TrapContext) {
    let scause = scause::read().cause();
    let stval = stval::read();
    match scause {
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
            ctx.x[10] = 0;
        }
        // Trap::Interrupt(Interrupt::MachineTimer) => {
        //     ctx.sepc += 4;
        //     // todo: handle timer interrupt
        //     ctx.x[10] = 0;
        // }
        _ => {
            panic!("Unhandled exception: {:?}, stval: {:#x}", scause, stval);
        }
    }
    unsafe extern "C" {
        safe fn __restore(ctx_addr: usize);
    }
    __restore(ctx as *mut TrapContext as usize);
}
