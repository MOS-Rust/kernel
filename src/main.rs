//! Rust entry point for the kernel
#![deny(missing_docs)]
#![deny(warnings)] 

#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[path ="platform/qemu/lib.rs"]
mod platform;
mod panic;

use core::{arch::global_asm, include_str, ptr::{addr_of_mut, write_bytes}};
use platform::{print_char, halt};
global_asm!(include_str!("init/entry.S"));

/// Entry point for the kernel, called by _entry() in init/entry.S
#[no_mangle]
pub fn kernel_init() -> ! {
    clear_bss();
    for c in "MOS-Rust started!\n".chars() {
        print_char(c);
    }
    halt()
}

/// Clear the .bss section
pub fn clear_bss() {
    extern "C" {
        static mut __start_bss: u8;
        static mut __end_bss: u8;
    }
    unsafe {
        write_bytes(addr_of_mut!(__start_bss), 0, addr_of_mut!(__end_bss) as usize - addr_of_mut!(__start_bss) as usize);
    }
}