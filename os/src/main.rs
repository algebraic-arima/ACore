#![no_std]
#![no_main]

use core::arch::global_asm;
use mmio::{shutdown, switch_s};

#[macro_use]
mod sbi;
mod lang_items;
mod logging;
mod mmio;

global_asm!(include_str!("entry.asm"));

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
    println!("[kernel] Hello QEMU");
    let c = sbi::scan();
    if c == 'Q' as u8 {
        shutdown(false);
    }
    switch_s(0x80000000, 0);
    shutdown(false)
}
