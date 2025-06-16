# Acore: A toy OS in rust

## Structure

Machine Mode

```
├── os/src
│   ├── config.rs
│   ├── entry.asm
│   ├── init.rs
│   ├── lang_items.rs
│   ├── link_kernel.S
│   ├── linker-qemu.ld
│   ├── loader.rs
│   ├── logging.rs
│   ├── main.rs
│   ├── sbi.rs
│   ├── trap
│   │   ├── context.rs
│   │   ├── mod.rs
│   │   ├── timer.rs
│   │   └── trap_m.S
│   └── uart.rs 
```

Supervisor Mode

```
├── sup/src
│   ├── config.rs
│   ├── drivers
│   │   ├── block
│   │   │   ├── mod.rs
│   │   │   └── virtio_blk.rs
│   │   └── mod.rs
│   ├── fs
│   │   ├── inode.rs
│   │   ├── mod.rs
│   │   ├── pipe.rs
│   │   └── stdio.rs
│   ├── lang_items.rs
│   ├── link_app.S
│   ├── linker-qemu.ld
│   ├── logging.rs
│   ├── main.rs
│   ├── mm
│   │   ├── address.rs
│   │   ├── allocator_test.rs
│   │   ├── buddy_allocator.rs
│   │   ├── frame_allocator.rs
│   │   ├── linked_list.rs
│   │   ├── memory_set.rs
│   │   ├── mod.rs
│   │   ├── page_table.rs
│   │   └── spin.rs
│   ├── sbi.rs
│   ├── sync.rs
│   ├── syscall.rs
│   ├── task
│   │   ├── context.rs
│   │   ├── manager.rs
│   │   ├── mod.rs
│   │   ├── pid.rs
│   │   ├── processor.rs
│   │   ├── switch.S
│   │   ├── switch.rs
│   │   └── task.rs
│   ├── timer.rs
│   ├── trap
│   │   ├── context.rs
│   │   ├── mod.rs
│   │   └── trap_s.S
│   └── uart.rs
```

User Mode

```
├── user/src
│   ├── batch.rs
│   ├── config.rs
│   ├── drivers
│   │   ├── block
│   │   │   ├── mod.rs
│   │   │   └── virtio_blk.rs
│   │   └── mod.rs
│   ├── fs
│   │   ├── inode.rs
│   │   ├── mod.rs
│   │   ├── pipe.rs
│   │   └── stdio.rs
│   ├── lang_items.rs
│   ├── link_app.S
│   ├── linker-qemu.ld
│   ├── logging.rs
│   ├── main.rs
│   ├── mm
│   │   ├── address.rs
│   │   ├── allocator_test.rs
│   │   ├── buddy_allocator.rs
│   │   ├── frame_allocator.rs
│   │   ├── linked_list.rs
│   │   ├── memory_set.rs
│   │   ├── mod.rs
│   │   ├── page_table.rs
│   │   └── spin.rs
│   ├── sbi.rs
│   ├── sync.rs
│   ├── syscall.rs
│   ├── task
│   │   ├── context.rs
│   │   ├── manager.rs
│   │   ├── mod.rs
│   │   ├── pid.rs
│   │   ├── processor.rs
│   │   ├── switch.S
│   │   ├── switch.rs
│   │   └── task.rs
│   ├── timer.rs
│   ├── trap
│   │   ├── context.rs
│   │   ├── mod.rs
│   │   └── trap_s.S
│   └── uart.rs
```

## Subtasks

- [x] Bootloader  
  - [x] Initialization  
  - [x] Entering S mode for the kernel  
- [x] Allocator  
  - [x] Buddy allocator  
  - [x] Frame allocator (or any fine-grained allocator for any size of memory)  
  - [ ] SLAB (Optional) 
- [x] Page table  
  - [x] For kernel  
  - [x] For each user process  
- [x] Console  
  - [x] Read  
  - [x] Write  
- [x] Message & data transfer  
  - [x] User \-\> Kernel  
  - [x] Kernel \-\> User  
  - [x] Kernel \-\> Kernel  
  - [x] User \-\> User  
- [x] Process  
  - [x] Process loading  
    - [x] ELF parsing  
    - [x] Sections loading (ref to page table)  
  - [x] Syscall  
    - [x] Kick off a new process (Something like fork and exec)  
    - [x] Wait for child processes (Something like wait)  
    - [x] Exit from a process (Something like exit)  
  - [x] Process manager  
    - [x] Process creation  
    - [x] Process termination  
  - [x] Scheduler  
    - [x] Context switch  
    - [x] Scheduling mechanism (must be time sharing)  
      - [ ] Advanced scheduling mechanism (Optional)
    - [x] Timer interrupt 
    - [ ] IPI (Optional)
  - [x] IPC 
    - [x] Pipe 
- [ ] Synchronization primitives  
  - [ ] Mutex  
  - [ ] Conditional variables (Optional) 
- [x] File system (Optional) 
  - [x] File/directory creation/deletion   
  - [x] File/directory renaming  
  - [x] File read  
  - [x] File write  
  - [x] File/directory moving  
  - [ ] (optional) access control, atime/mtime/…  
- [ ] Multicore (Optional) 
- [ ] Driver (Optional)
