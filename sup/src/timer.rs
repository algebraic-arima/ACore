use riscv::register::time;
use crate::config::*;

pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}


pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}

pub fn set_next_trigger() {
    unsafe {
        let mtimecmp_addr = (MTIMECMP as usize) as *mut u64;
        mtimecmp_addr.write_volatile(time::read() as u64 + TIME_INTERVAL);
    }
}