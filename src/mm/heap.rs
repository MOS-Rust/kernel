use alloc::Allocator;

use super::addr;

// For 64MiB of memory, it will take 26 bits to represend each byte.
// So 32 bits are enough.
#[global_allocator]
static ALLOCATOR: Allocator<32> = Allocator::new();

#[alloc_error_handler]
pub fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn init(memsize: usize) {
    unsafe{
        ALLOCATOR.lock()
        .add_size(addr::KSEG0, memsize);
    }
}