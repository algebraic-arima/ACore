mod context;

// use crate::syscall::syscall;
use crate::sbi::*;
use context::TrapContext;
use core::arch::global_asm;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval,
    stvec::{self, TrapMode},
};

global_asm!(include_str!("trap_s.S"));

pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[unsafe(no_mangle)] // handle traps from s and u
pub fn trap_handler_s(ctx: &mut TrapContext) {
    // println!("dfddfd");
    let mcause = scause::read().cause();
    let stval = stval::read();
    // println!("mcause: {:?}, stval: {:#x}", mcause, stval);
    match mcause {
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            // syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
            ctx.x[10] = 0;
        }
        // Trap::Interrupt(Interrupt::SupervisorTimer) => {
        //     ctx.sepc += 4;
        //     // todo: handle timer interrupt
        //     ctx.x[10] = 0;
        // }
        _ => {
            panic!("Unhandled exception: {:?}, stval: {:#x}", mcause, stval);
        }
    }
    unsafe extern "C" {
        safe fn __restore(ctx_addr: usize);
    }
    __restore(ctx as *mut TrapContext as usize);
}
