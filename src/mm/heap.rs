use allocator::Allocator;
// For 64MiB of memory, it will take 26 bits to represend each byte.
// So 32 bits are enough.
#[global_allocator]
static ALLOCATOR: Allocator<32> = Allocator::new();
static mut KERNEL_HEAP: [u8; 10_000] = [0; 10_000];

pub fn init(_memsize: usize) {

    unsafe {
        ALLOCATOR.lock().add_size
        (KERNEL_HEAP.as_ptr() as usize, 10_000);
    }
}