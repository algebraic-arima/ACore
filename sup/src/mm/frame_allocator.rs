use crate::config::*;
use alloc::vec::Vec;

use crate::sync::UPSafeCell;

use super::address::*;

pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();

pub fn init_frame() {
    unsafe extern "C" {
        safe fn ekernel();
    }
    FRAME_ALLOCATOR.0.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

pub struct StackFrameAllocator {
    bottom: usize,
    end: usize,
    recycled: Vec<usize>,
}

trait FrameAlloca {
    fn init(&mut self, bottom: PhysPageNum, end: PhysPageNum);
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

impl StackFrameAllocator {
    pub const fn new() -> Self {
        StackFrameAllocator {
            bottom: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
}

impl FrameAlloca for StackFrameAllocator {
    fn init(&mut self, bottom: PhysPageNum, end: PhysPageNum) {
        self.bottom = bottom.0;
        self.end = end.0;
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if self.bottom == self.end {
            return None;
        } else if let Some(ppn) = self.recycled.pop() {
            return Some(PhysPageNum(ppn));
        } else {
            self.bottom += 1;
            return Some(PhysPageNum(self.bottom - 1));
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.bottom {
            panic!("DEALLOC: Frame ppn={:#x} has not been allocated!", ppn);
        }
        for i in 0..self.recycled.len() {
            if self.recycled[i] == ppn {
                panic!("DEALLOC: Frame ppn={:#x} has not been allocated!", ppn);
            }
        }
        self.recycled.push(ppn);
    }
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.dealloc(self.ppn);
    }
}

pub struct FrameAllocator(UPSafeCell<StackFrameAllocator>);

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator(unsafe { UPSafeCell::new(StackFrameAllocator::new()) })
    }

    pub fn alloc(&self) -> Option<FrameTracker> {
        self.0.exclusive_access().alloc().map(FrameTracker::new)
    }

    pub fn dealloc(&self, ppn: PhysPageNum) {
        self.0.exclusive_access().dealloc(ppn);
    }
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.alloc()
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.dealloc(ppn);
}
