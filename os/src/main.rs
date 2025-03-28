#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

#[macro_use]
mod sbi;
mod config;
mod init;
mod lang_items;
mod loader;
mod logging;
mod trap;
mod uart;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_kernel.S"));

/// clear BSS segment
pub fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// the rust entry-point of m mode, at 0x80000000
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    sbi::init_uart();
    logging::init();
    log::info!("[kernel] POWERON");
    loader::load_kernel();
    loader::run_kernel();
    sbi::shutdown(false)
}
