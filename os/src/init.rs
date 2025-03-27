use core::arch::asm;
use riscv::register::*;
use crate::trap::*;

pub fn switch_s(s_mode_entry: usize, hartid: usize) {
    unsafe {
        mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
        mepc::write(s_mode_entry as usize);
        // may call os::trap::context::os_init_context here and get a ctx

        satp::write(0);

        asm!("csrw medeleg, {}", in(reg) !0);
        asm!("csrw mideleg, {}", in(reg) !0);

        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();

        pmpaddr0::write(0x3fffffffffffffusize);
        pmpcfg0::write(0xf);

        set_next_trigger(0);

        unsafe extern "C" {
            safe fn __alltraps_m();
        } // all-using trap handler from s to m
        mtvec::write(
            __alltraps_m as usize,
            riscv::register::mtvec::TrapMode::Direct,
        );

        mstatus::set_mie();
        mie::set_mtimer();
    };
}
