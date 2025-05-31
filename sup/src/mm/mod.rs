pub use address::{PhysPageNum, VirtAddr, VirtPageNum};
pub use memory_set::{KERNEL_SPACE, MapPermission, MemorySet, remap_test, kernel_token};
pub use page_table::*;
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};


mod address;
mod allocator_test;
mod buddy_allocator;
mod frame_allocator;
mod linked_list;
mod memory_set;
mod page_table;

pub fn init() {
    buddy_allocator::init_heap();
    // allocator_test::heap_test();
    frame_allocator::init_frame();
    // allocator_test::frame_allocator_test();
    // println!("start init");
    KERNEL_SPACE.exclusive_access().activate();
    // println!("init done");
}
