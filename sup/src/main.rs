#![no_std]
#![feature(linkage)]
#![no_main]

use log::warn;

#[macro_use]
mod lang_items;
mod syscall;
mod sbi;
mod mmio;
mod trap;
mod logging;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    logging::init();
    sbi::init_uart();
    warn!("[kernel] Switched to Supervisor Mode");
    sbi::shutdown(false)
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
