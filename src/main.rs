//! Rust entry point for the kernel
#![deny(missing_docs)]
#![deny(warnings)] 

#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

mod export;
#[cfg(target_arch = "mips")]
#[path ="platform/mips/mod.rs"]
mod platform;
mod panic;
mod console;
mod error;
mod mm;

use core::{arch::global_asm, include_str, ptr::{addr_of_mut, write_bytes}};


#[cfg(target_arch = "mips")]
global_asm!(include_str!("../asm/init/entry.S"));

/// Entry point for the kernel, called by _entry() in init/entry.S
#[no_mangle]
pub extern "C" fn kernel_init(
    _argc: usize,
    _argv: *const *const char,
    _envp: *const *const char,
    ram_size: usize,
) -> ! {
    clear_bss();
    println!("MOS-Rust started!");
    mm::init(ram_size);
    panic!()
}

/// Clear the .bss section
fn clear_bss() {
    extern "C" {
        static mut __start_bss: u8;
        static mut __end_bss: u8;
    }
    unsafe {
        write_bytes(addr_of_mut!(__start_bss), 0, addr_of_mut!(__end_bss) as usize - addr_of_mut!(__start_bss) as usize);
    }
}