mod heap;
pub mod addr;
pub mod page;
pub mod layout;
pub mod map;
mod tlb;

use log::info;

static mut MEMSIZE: usize = 0;
static mut PAGENUM: usize = 0;

pub fn init(memsize: usize) {
    unsafe {
        set_memsize(memsize);
    }
    info!(
        "Memory size: {} KiB, number of pages: {}.",
        get_memsize() / 1024,
        get_pagenum()
    );
    heap::init();
    heap::heap_test();
    page::init();
    page::alloc_test();
    map::mapping_test();
}

unsafe fn set_memsize(memsize: usize) {
    if MEMSIZE != 0 {
        panic!("Memory size has been set.");
    }
    MEMSIZE = memsize;
    PAGENUM = memsize / layout::PAGE_SIZE;
}

#[inline]
pub fn get_memsize() -> usize {
    unsafe { MEMSIZE }
}

#[inline]
pub fn get_pagenum() -> usize {
    unsafe { PAGENUM }
}
