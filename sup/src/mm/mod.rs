pub use address::{PhysPageNum, VirtAddr};
pub use memory_set::{KERNEL_SPACE, MapPermission, MemorySet, remap_test};
pub use page_table::*;

mod address;
mod allocator_test;
mod buddy_allocator;
mod frame_allocator;
mod linked_list;
mod memory_set;
mod page_table;
mod spin;

pub fn init() {
    buddy_allocator::init_heap();
    // allocator_test::heap_test();
    frame_allocator::init_frame();
    // allocator_test::frame_allocator_test();
    // println!("start init");
    KERNEL_SPACE.exclusive_access().activate();
    // println!("init done");
}
