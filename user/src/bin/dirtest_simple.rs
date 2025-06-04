#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, mkdir, open, read, remove, rename, write, OpenFlags};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let test_str = "Hello, world!";
    let dir_name = "tmp\0";
    assert!(mkdir(dir_name) == 0, "Failed to create directory");
    let filea = "tmp/filea\0";
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, test_str.as_bytes());
    close(fd);

    let fd = open(filea, OpenFlags::RDONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);

    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap(),);

    let fileb = "tmp/fileb\0";
    let fd = open(fileb, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, test_str.as_bytes());
    close(fd);

    rename(fileb, "venillalemon\0");

    let renamed_file = "tmp/venillalemon\0";
    let fd = open(renamed_file, OpenFlags::RDONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);

    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap(),);

    assert!(remove(dir_name) == 0, "Failed to remove directory");

    println!("dir_test passed!");
    0
}
