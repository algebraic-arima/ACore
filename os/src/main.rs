#![no_std]
#![no_main]

use core::arch::global_asm;

#[macro_use]
mod sbi;
mod lang_items;
mod logging;
mod mmio;
mod trap;
mod syscall;
mod init;

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
    println!("[kernel] POWERON");
    init::switch_s(0x80200000, 0);
    sbi::shutdown(false)
}
