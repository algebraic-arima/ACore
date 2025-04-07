mod allocator_test;
mod buddy_allocator;
mod linked_list;

extern crate alloc;

pub fn init() {
    buddy_allocator::init_heap();
}
