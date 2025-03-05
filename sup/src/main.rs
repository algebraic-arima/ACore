#![no_std]
#![feature(linkage)]
#![no_main]
#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    main();
    console::exit(0);
    panic!("unreachable after sys_exit!");
}

#[unsafe(no_mangle)]
fn main() -> () {
    println!("[kernel] supervisor mode");
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
