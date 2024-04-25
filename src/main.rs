//! Rust entry point for the kernel
//!
//! This crate is the entry point for the kernel. It is responsible for initializing the kernel and starting its execution.

#![deny(missing_docs)]
#![deny(warnings)] 

#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod export;
#[cfg(target_arch = "mips")]
#[path ="platform/qemu/lib.rs"]
mod platform;
mod panic;
mod console;

use core::{arch::global_asm, include_str, ptr::{addr_of_mut, write_bytes}};

#[cfg(target_arch = "mips")]
global_asm!(include_str!("../asm/init/entry.S"));

/// Kernel initialization function
///
/// This function is the entry point of the kernel. It is called by _entry() in init/entry.S when the kernel starts and is responsible
/// for initializing various modules of the kernel.
///
#[no_mangle]
pub fn kernel_init() -> ! {
    clear_bss();
    println!("MOS-Rust started!");
    panic!()
}

/// Clear the .bss section
///
/// This function clears the `.bss` section of the kernel. 
/// All memory locations in the `.bss` section (i.e. from __start_bss to __end_bss) are set to 0 in this function.
pub fn clear_bss() {
    extern "C" {
        static mut __start_bss: u8;
        static mut __end_bss: u8;
    }
    unsafe {
        write_bytes(addr_of_mut!(__start_bss), 0, addr_of_mut!(__end_bss) as usize - addr_of_mut!(__start_bss) as usize);
    }
}