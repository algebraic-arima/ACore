
    .align 3
    .section .data

    .quad kernel_start
    .quad kernel_end

    .section .data
    .global kernel_start
    .global kernel_end
kernel_start:
    li a0, 2
    .incbin "../sup/target/riscv64gc-unknown-none-elf/release/sup.bin"
kernel_end:
