use alloc::{string::String, vec::Vec};
use allocator::Allocator;

use crate::println;

// 1MiB
const KERNEL_HEAP_SIZE: usize = 0x100_000;

// For 64MiB of memory, it will take 26 bits to represent each byte.
// So 32 bits are enough.
#[global_allocator]
static ALLOCATOR: Allocator<32> = Allocator::new();
static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        ALLOCATOR.lock().add_size
        (KERNEL_HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
    println!("Initialized {} KiB of kernel heap.", KERNEL_HEAP_SIZE / 1024);
}

pub fn heap_test() {
    let mut v = Vec::<u8>::new();
    for i in 0..=255 {
        v.push(i);
    }
    for i in 0..=255 {
        assert_eq!(v[i], i as u8);
    }
    let s = String::from("Hello, world!");
    assert_eq!(s, "Hello, world!");
    drop(v);
    println!("Heap test passed!");
}