mod addr;
mod heap;
pub mod layout;

pub fn init(_memsize: usize) {
    heap::init();
    heap::heap_test();
}

