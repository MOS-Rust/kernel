pub mod addr;
mod heap;

pub fn init(_memsize: usize) {
    heap::init();
    heap::heap_test();
}

