pub const MTIME: usize = 0x0200bff8;
pub const MTIMECMP: usize = 0x02004000;
pub const TIME_INTERVAL: u64 = 200000;

pub const MACHINE_START: usize = 0x80000000;
pub const SUPERVISOR_START: usize = 0x80200000;
pub const MEMORY_END: usize = 0x80800000;

pub const MACHINE_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;


pub const PA_WIDTH_SV39: usize = 56;
pub const VA_WIDTH_SV39: usize = 39;
pub const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
pub const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

pub const CLOCK_FREQ: usize = 12500000;
pub const TICKS_PER_SEC: usize = 100;
pub const MSEC_PER_SEC: usize = 1000;
pub const MICRO_PER_SEC: usize = 1_000_000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub const VIRT_UART0_BASE: usize = 0x10000000;
pub const VIRT_TEST_BASE: usize = 0x100000;

pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}