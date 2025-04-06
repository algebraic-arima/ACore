use std::fs::File;
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../sup/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_kernel_data().unwrap();
}

static TARGET_PATH: &str = "../sup/target/riscv64gc-unknown-none-elf/release/sup.bin";

fn insert_kernel_data() -> Result<()> {
    let mut f = File::create("../os/src/link_kernel.S").unwrap();

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global kernel_start
    .global kernel_end
kernel_start:
    .incbin "{}"
kernel_end:"#,
        TARGET_PATH
    )?;
    Ok(())
}
