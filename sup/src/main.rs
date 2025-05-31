#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![no_main]

extern crate alloc;

use log::info;

#[macro_use]
mod sbi;
mod sync;
mod config;
pub mod lang_items;
mod logging;
mod uart;
pub mod syscall;
pub mod trap;
pub mod mm;
pub mod task;
pub mod timer;
pub mod fs;
mod drivers;

core::arch::global_asm!(include_str!("link_app.S"));

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    logging::init();
    sbi::init_uart();
    info!("[kernel] Switched to Supervisor Mode");
    mm::init();
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
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
