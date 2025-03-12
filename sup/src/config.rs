pub const MTIME: *const u64 = 0x0200bff8 as *const u64;
pub const MTIMECMP: *mut u64 = 0x02004000 as *mut u64;
pub const TIME_INTERVAL: u64 = 100000;
pub const MEMORY_END: usize = 0x80800000;

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;