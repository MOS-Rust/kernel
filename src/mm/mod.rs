mod addr;
mod heap;
pub mod layout;

use crate::println;

static mut MEMSIZE: usize = 0;
static mut PAGENUM: usize = 0;

pub fn init(memsize: usize) {
    unsafe {
        set_memsize(memsize);
    }
    println!(
        "Memory size: {} KiB, number of pages: {}.",
        get_memsize() / 1024,
        get_pagenum()
    );
    heap::init();
    heap::heap_test();
}

unsafe fn set_memsize(memsize: usize) {
    if MEMSIZE != 0 {
        panic!("Memory size has been set.");
    }
    MEMSIZE = memsize;
    PAGENUM = memsize / layout::PAGE_SIZE;
}

pub fn get_memsize() -> usize {
    unsafe { MEMSIZE }
}

pub fn get_pagenum() -> usize {
    unsafe { PAGENUM }
}
