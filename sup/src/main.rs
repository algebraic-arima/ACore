#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![no_main]

extern crate alloc;

use log::info;
use mm::remap_test;

#[macro_use]
mod sync;
mod config;
mod lang_items;
mod logging;
mod uart;
mod sbi;
mod syscall;
mod trap;
mod mm;
mod task;
mod timer;
mod loader;

core::arch::global_asm!(include_str!("link_app.S"));

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    logging::init();
    sbi::init_uart();
    info!("[kernel] Switched to Supervisor Mode");
    mm::init();
    remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    sbi::shutdown(false)
}

fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}
