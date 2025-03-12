mod address;
mod buddy_test;
mod buddy_allocator;
pub mod linked_list;
mod spin;

pub fn init() {
    buddy_allocator::init_heap();
    buddy_test::heap_test();
}
