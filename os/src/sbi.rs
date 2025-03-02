use crate::mmio::MmioSerialPort;
use core::fmt::{self, Write};

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
        MSP = Some(MmioSerialPort::new());
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
