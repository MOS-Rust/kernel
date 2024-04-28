//! Memory management module for MOS
//!
//! This module provides memory management functionality for the MOS kernel.
//! 
//! It includes functions for initializing memory, managing the heap and handling page allocation and mapping.

mod addr;
mod heap;
mod page;
pub mod layout;
pub mod map;
pub mod tlb;

use crate::println;

static mut MEMSIZE: usize = 0;
static mut PAGENUM: usize = 0;

/// Initializes the memory management module.
///
/// This function sets the memory size, initializes the heap, and performs various tests.
/// It also prints the memory size and number of pages to the console.
///
/// # Arguments
///
/// * `memsize` - The total size of memory in bytes.
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
    page::init();
    page::alloc_test();
    map::mapping_test();
}

/// Sets the total memory size.
/// 
/// Should only be called once.
/// 
/// # Arguments
/// 
/// * `memsize` - The total size of memory in bytes.
/// 
/// # Panics
/// 
/// Panics if the memory size has already been set.
unsafe fn set_memsize(memsize: usize) {
    if MEMSIZE != 0 {
        panic!("Memory size has been set.");
    }
    MEMSIZE = memsize;
    PAGENUM = memsize / layout::PAGE_SIZE;
}

/// Returns the total memory size in bytes.
///
/// # Returns
///
/// The total memory size in bytes.
#[inline]
pub fn get_memsize() -> usize {
    unsafe { MEMSIZE }
}

/// Returns the number of pages in memory.
///
/// # Returns
///
/// The number of physical pages in memory.
#[inline]
pub fn get_pagenum() -> usize {
    unsafe { PAGENUM }
}
