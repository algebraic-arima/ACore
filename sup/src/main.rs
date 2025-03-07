#![no_std]
#![feature(linkage)]
#![no_main]

use core::arch::asm;

use log::info;
use riscv::register::time;

#[macro_use]
mod lang_items;
mod logging;
mod mmio;
mod sbi;
mod syscall;
mod trap;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    logging::init();
    sbi::init_uart();
    println!("time:{}", get_time());
    info!("[kernel] Switched to Supervisor Mode");
    println!("time:{}", get_time());
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
