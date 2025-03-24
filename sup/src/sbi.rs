use log::{error, warn, info};

use crate::uart::MmioSerialPort;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::fmt::{self, Write};

static VIRT_UART0_BASE: usize = 0x10000000;
static VIRT_TEST_BASE: usize = 0x100000;

static mut MSP: Option<MmioSerialPort> = None;

impl fmt::Write for MmioSerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

pub fn init_uart() {
    unsafe {
        MSP = Some(MmioSerialPort::new(VIRT_UART0_BASE));
        if let Some(ref mut uart) = MSP {
            uart.init();
        }
    }
}

pub fn print(args: fmt::Arguments) {
    unsafe {
        if let Some(ref mut uart) = MSP {
            uart.write_fmt(args).unwrap();
        }
    }
}

pub fn scan() -> u8 {
    unsafe {
        if let Some(ref mut uart) = MSP {
            uart.receive()
        } else {
            0
        }
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::sbi::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::sbi::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
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
        reset_addr.load(Ordering::Relaxed).write(0x7777);
    }
    panic!("Reset Failed");
}
