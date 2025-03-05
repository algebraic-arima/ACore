// use core::fmt::{self, Write};
// use crate::syscall::{sys_write, sys_exit};

// struct Stdout;

// const STDOUT: usize = 1;

// pub fn write(fd: usize, buf: &[u8]) -> isize {
//     sys_write(fd, buf)
// }

// pub fn exit(exit_code: i32) -> isize {
//     sys_exit(exit_code)
// }

// impl Write for Stdout {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         write(STDOUT, s.as_bytes());
//         Ok(())
//     }
// }

// pub fn print(args: fmt::Arguments) {
//     Stdout.write_fmt(args).unwrap();
// }

// #[macro_export]
// macro_rules! print {
//     ($fmt: literal $(, $($arg: tt)+)?) => {
//         $crate::console::print(format_args!($fmt $(, $($arg)+)?));
//     }
// }

// #[macro_export]
// macro_rules! println {
//     ($fmt: literal $(, $($arg: tt)+)?) => {
//         $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
//     }
// }
