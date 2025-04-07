extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr::NonNull;

use crate::mm::linked_list::*;

const MAX_ORDER: usize = 32;

use crate::sync::UPSafeCell;
use core::ptr::addr_of_mut;
use log::*;

const USER_HEAP_SIZE: usize = 16384;

#[global_allocator]
pub static HEAP_ALLOCATOR: BuddyAllocator = BuddyAllocator::new();

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.0.exclusive_access().init(
            addr_of_mut!(HEAP_SPACE) as usize,
            addr_of_mut!(HEAP_SPACE) as usize + USER_HEAP_SIZE,
        );
    }
}

pub struct Heap {
    list: [LinkedList; MAX_ORDER],
    idle: usize,
}

impl Heap {
    pub const fn new() -> Self {
        Heap {
            list: [LinkedList::new(); MAX_ORDER],
            idle: 0,
        }
    }

    pub unsafe fn init(&mut self, mut start: usize, mut end: usize) {
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end = end & (!size_of::<usize>() + 1);
        if start >= end {
            error!(
                "Invalid memory region: start = {:#x}, end = {:#x}",
                start, end
            );
            return;
        }

        while start + size_of::<usize>() <= end {
            let lev = (end - start).trailing_zeros() as usize;
            unsafe { self.list[lev].push(start as *mut usize) };
            start += 1 << lev;
            self.idle += 1 << lev;
        }
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        // info!("alloc {} bytes", layout.size());
        let size = layout.size();
        let align = layout.align();
        let bsize = max(size.next_power_of_two(), max(align, size_of::<usize>()));
        let lev = bsize.trailing_zeros() as usize;
        for i in lev..MAX_ORDER {
            if self.list[i].is_empty() {
                continue;
            }
            for j in (lev + 1..i + 1).rev() {
                if let Some(seg) = self.list[j].pop() {
                    unsafe {
                        self.list[j - 1].push(seg);
                        self.list[j - 1].push((seg as usize + (1 << (j - 1))) as *mut usize);
                    }
                } else {
                    return Err(());
                }
            }
            // println!("size = {:#x}", layout.size());
            // println!("bsize = {:#x}", bsize);
            // println!("idle size = {:#x}", self.idle);
            // println!("level = {}", lev);
            let result = NonNull::new(
                self.list[lev]
                    .pop()
                    .expect("current block should have free space now") as *mut u8,
            );
            if let Some(result) = result {
                self.idle -= bsize;
                // println!("idle size after = {:#x}", self.idle);
                return Ok(result);
            } else {
                return Err(());
            }
        }
        Err(())
    }

    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        // info!("dealloc {} bytes", layout.size());
        let size = layout.size();
        let align = layout.align();
        let bsize = max(size.next_power_of_two(), max(align, size_of::<usize>()));
        let lev = bsize.trailing_zeros() as usize;
        unsafe {
            self.list[lev].push(ptr.as_ptr() as *mut usize);

            let mut cur_ptr = ptr.as_ptr() as usize;
            let mut cur_lev = lev;
            while cur_lev < MAX_ORDER {
                let bud = cur_ptr ^ (1 << cur_lev);
                let mut found = false;
                for block in self.list[cur_lev].iter_mut() {
                    if block.as_ptr() == bud as *mut usize {
                        block.pop();
                        found = true;
                        break;
                    }
                }

                if !found {
                    break;
                }
                self.list[cur_lev].pop();
                cur_ptr = min(cur_ptr, bud);
                cur_lev += 1;
                self.list[cur_lev].push(cur_ptr as *mut usize);
            }
            self.idle += bsize;
            // println!("idle size after = {:#x}", self.idle);
        }
    }
}

pub struct BuddyAllocator(UPSafeCell<Heap>);

impl BuddyAllocator {
    pub const fn new() -> BuddyAllocator {
        BuddyAllocator(unsafe { UPSafeCell::new(Heap::new()) })
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            self.0
                .exclusive_access()
                .alloc(layout)
                .ok()
                .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.0
                .exclusive_access()
                .dealloc(NonNull::new_unchecked(ptr), layout)
        }
    }
}
