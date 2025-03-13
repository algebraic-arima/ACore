#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![no_main]

extern crate alloc;

use core::arch::asm;

use log::info;
use riscv::register::{medeleg, time, mideleg};

#[macro_use]
mod sync;
mod config;
mod lang_items;
mod logging;
mod mmio;
mod sbi;
mod syscall;
mod trap;
mod mm;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    logging::init();
    sbi::init_uart();
    println!("time:{}", get_time());
    info!("[kernel] Switched to Supervisor Mode");
    let mut cnt = 0;
    while cnt < 1000 {
        info!("time = {} at loop {}, {}", get_time(), cnt, cnt * cnt);
        cnt += 1;
    }
    mm::init();
    sbi::shutdown(false)
}

pub fn get_time() -> usize {
    time::read()
}

fn clear_bss() {
    unsafe extern "C" {
        safe fn start_bss();
        safe fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}
