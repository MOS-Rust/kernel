//! Memory management module for MOS
//!
//! This module provides memory management functionality for the MOS kernel.
//!
//! It includes functions for initializing memory, managing the heap and handling page allocation and mapping.

pub mod addr;
mod heap;
pub mod layout;
pub mod map;
pub mod page;
mod tlb;

pub use tlb::tlb_invalidate;

use log::info;

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
    info!(
        "Memory size: {} KiB, number of pages: {}.",
        get_memsize() / 1024,
        get_pagenum()
    );
    heap::init();
    page::init();
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
