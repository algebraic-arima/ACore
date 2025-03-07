    .align 4
    .section .data
    .global kernel_start
    .global kernel_end
kernel_start:
    .incbin "../sup/target/riscv64gc-unknown-none-elf/release/sup.bin"
kernel_end:
