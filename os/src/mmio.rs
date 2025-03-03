use core::sync::atomic::{AtomicPtr, Ordering};

use bitflags::bitflags;
use log::*;

static VIRT_UART0_BASE: usize = 0x10000000;
static VIRT_TEST_BASE: usize = 0x100000;

/// A memory-mapped UART.
#[derive(Debug)]
pub struct MmioSerialPort {
    data: AtomicPtr<u8>,
    int_en: AtomicPtr<u8>,
    fifo_ctrl: AtomicPtr<u8>,
    line_ctrl: AtomicPtr<u8>,
    modem_ctrl: AtomicPtr<u8>,
    line_sts: AtomicPtr<u8>,
}

bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop()
        }
    };
}

impl MmioSerialPort {
    /// Creates a new UART interface on the given memory mapped address.
    ///
    /// This function is unsafe because the caller must ensure that the given base address
    /// really points to a serial port device.
    // #[rustversion::attr(since(1.61), const)]
    pub unsafe fn new() -> Self {
        let base_pointer = VIRT_UART0_BASE as *mut u8;
        Self {
            data: AtomicPtr::new(base_pointer),
            int_en: AtomicPtr::new(unsafe { base_pointer.add(1) }),
            fifo_ctrl: AtomicPtr::new(unsafe { base_pointer.add(2) }),
            line_ctrl: AtomicPtr::new(unsafe { base_pointer.add(3) }),
            modem_ctrl: AtomicPtr::new(unsafe { base_pointer.add(4) }),
            line_sts: AtomicPtr::new(unsafe { base_pointer.add(5) }),
        }
    }

    /// Initializes the memory-mapped UART.
    ///
    /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
    pub fn init(&mut self) {
        let self_int_en = self.int_en.load(Ordering::Relaxed);
        let self_line_ctrl = self.line_ctrl.load(Ordering::Relaxed);
        let self_data = self.data.load(Ordering::Relaxed);
        let self_fifo_ctrl = self.fifo_ctrl.load(Ordering::Relaxed);
        let self_modem_ctrl = self.modem_ctrl.load(Ordering::Relaxed);
        unsafe {
            // Disable interrupts
            self_int_en.write(0x00);

            // Enable DLAB
            self_line_ctrl.write(0x80);

            // Set maximum speed to 38400 bps by configuring DLL and DLM
            self_data.write(0x03);
            self_int_en.write(0x00);

            // Disable DLAB and set data word length to 8 bits
            self_line_ctrl.write(0x03);

            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            self_fifo_ctrl.write(0xC7);

            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            self_modem_ctrl.write(0x0B);

            // Enable interrupts
            self_int_en.write(0x01);
        }
    }

    fn line_sts(&mut self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(*self.line_sts.load(Ordering::Relaxed)) }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        let self_data = self.data.load(Ordering::Relaxed);
        unsafe {
            match data {
                8 | 0x7F => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data.write(8);
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data.write(b' ');
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data.write(8)
                }
                _ => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self_data.write(data);
                }
            }
        }
    }

    /// Receives a byte on the serial port.
    pub fn receive(&mut self) -> u8 {
        let self_data = self.data.load(Ordering::Relaxed);
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::INPUT_FULL));
            self_data.read()
        }
    }
}

pub fn shutdown(failure: bool) -> ! {
    let base_pointer = VIRT_TEST_BASE as *mut u32;
    let shutdown_addr = AtomicPtr::new(base_pointer);

    unsafe {
        if failure {
            error!("[kernel] Unspecified Error. POWEROFF");
            shutdown_addr.load(Ordering::Relaxed).write(0x3333);
        } else {
            warn!("[kernel] POWEROFF");
            shutdown_addr.load(Ordering::Relaxed).write(0x5555);
        }
    }
    error!("Shutdown Failed");
    loop {}
}

pub fn reset() -> ! {
    let base_pointer = VIRT_TEST_BASE as *mut u32;
    let reset_addr = AtomicPtr::new(base_pointer);

    unsafe {
        info!("[kernel] RESET");
        reset_addr.load(Ordering::Relaxed).write(0x5555);
    }
    panic!("Reset Failed");
}

use core::arch::asm;
use riscv::register::*;

// #[inline(always)]
// unsafe fn write_csr(csr: usize, value: usize) {
//     unsafe {
//         asm!("csrw {0}, {1}",
//             in(reg) csr,
//             in(reg) value,
//             options(nostack, preserves_flags)
//         );
//     }
// }

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

        let timer_handler: usize = timer_interrupt_handler as usize;
        mtvec::write(timer_handler, riscv::register::mtvec::TrapMode::Direct);

        mstatus::set_mie();
        mie::set_mtimer();

        asm!("mret", options(noreturn));
    };
}

extern "C" fn timer_interrupt_handler() {}
