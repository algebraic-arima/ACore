use memory_set::KERNEL_SPACE;

use crate::println;

mod address;
mod allocator_test;
mod buddy_allocator;
mod linked_list;
mod frame_allocator;
mod page_table;
pub mod memory_set;
mod spin;

pub fn init() {
    buddy_allocator::init_heap();
    // allocator_test::heap_test();
    frame_allocator::init_frame();
    // allocator_test::frame_allocator_test();
    println!("start init");
    KERNEL_SPACE.exclusive_access().activate();
    println!("init done");
}
