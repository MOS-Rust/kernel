//! Rust entry point for the kernel
//!
//! This crate is the entry point for the kernel. It is responsible for initializing the kernel and starting its execution.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(asm_experimental_arch)]
#![feature(asm_const)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

mod console;
mod error;
mod export;
mod logging;
mod mm;
mod panic;
mod platform;
mod test;
mod exception;

use core::{
    arch::global_asm,
    include_str,
    ptr::{addr_of_mut, write_bytes},
};

use log::info;
use mips::registers::cp0::{compare, count};

global_asm!(include_str!("../asm/init/entry.S"));

/// Kernel initialization function
///
/// This function is the entry point of the kernel. It is called by _entry() in init/entry.S when the kernel starts and is responsible
/// for initializing various modules of the kernel.
///
#[no_mangle]
pub extern "C" fn kernel_init(
    _argc: usize,
    _argv: *const *const char,
    _envp: *const *const char,
    ram_size: usize,
) -> ! {
    clear_bss();
    logging::init();
    info!("MOS-Rust started!");
    mm::init(ram_size);
    unsafe {exception::trapframe::init();}
    println!("{}",unsafe { compare::read() });
    println!("{}",unsafe { count::read() });
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
        write_bytes(
            addr_of_mut!(__start_bss),
            0,
            addr_of_mut!(__end_bss) as usize - addr_of_mut!(__start_bss) as usize,
        );
    }
}
