use core::arch::asm;
use riscv::register::*;

const MTIME: *const u64 = 0x0200bff8 as *const u64;
const MTIMECMP: *mut u64 = 0x02004000 as *mut u64;
const TIME_INTERVAL: u64 = 1000000;

unsafe fn set_medeleg() {
    unsafe {
        medeleg::set_breakpoint();
        medeleg::set_illegal_instruction();
        medeleg::set_instruction_fault();
        medeleg::set_instruction_misaligned();
        medeleg::set_instruction_page_fault();
        medeleg::set_load_fault();
        medeleg::set_load_misaligned();
        medeleg::set_load_page_fault();
        medeleg::set_store_fault();
        medeleg::set_store_misaligned();
        medeleg::set_store_page_fault();
        medeleg::set_machine_env_call();
        medeleg::set_supervisor_env_call();
        medeleg::set_user_env_call();
    }
}

unsafe fn set_mideleg() {
    unsafe {
        mideleg::set_sext();
        mideleg::set_ssoft();
        mideleg::set_stimer();
        mideleg::set_uext();
        mideleg::set_usoft();
        mideleg::set_utimer();
    }
}

pub fn switch_s(s_mode_entry: usize, hartid: usize) {
    unsafe {
        mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
        mepc::write(s_mode_entry as usize);
        // may call os::trap::context::os_init_context here and get a ctx

        satp::write(0);

        set_medeleg();
        set_mideleg();

        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();

        pmpaddr0::write(0x3fffffffffffff);
        pmpcfg0::write(0xf);

        let mtime = MTIME.read_volatile();
        let mtimecmp_addr = (MTIMECMP as usize + 8 * hartid) as *mut u64;
        mtimecmp_addr.write_volatile(mtime + TIME_INTERVAL);

        unsafe extern "C" {
            safe fn __alltraps();
        } // all-using trap handler from s to m
        mtvec::write(__alltraps as usize, riscv::register::mtvec::TrapMode::Direct);

        mstatus::set_mie();
        mie::set_mtimer();

        asm!("mret", options(noreturn));
    };
}

