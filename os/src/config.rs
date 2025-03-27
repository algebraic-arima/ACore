pub const MTIME: *const u64 = 0x0200bff8 as *const u64;
pub const MTIMECMP: *mut u64 = 0x02004000 as *mut u64;
pub const TIME_INTERVAL: u64 = 200000;

pub const MACHINE_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const CLOCK_FREQ: usize = 12500000;
pub const TICKS_PER_SEC: usize = 100;
pub const MSEC_PER_SEC: usize = 1000;
pub const MICRO_PER_SEC: usize = 1_000_000;