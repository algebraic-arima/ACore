mod address;
mod allocator_test;
mod buddy_allocator;
mod linked_list;
mod frame_allocator;
mod page_table;
mod spin;

pub fn init() {
    buddy_allocator::init_heap();
    allocator_test::heap_test();
    frame_allocator::init_frame();
    allocator_test::frame_allocator_test();
}
