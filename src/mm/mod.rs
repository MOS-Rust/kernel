pub mod addr;
mod heap;

pub fn init(memsize: usize) {
    heap::init(memsize);
}