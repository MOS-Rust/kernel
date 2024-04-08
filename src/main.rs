//! Rust entry point for the kernel
#![deny(missing_docs)]
#![deny(warnings)]

#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod panic;

use core::{arch::global_asm, include_str};

global_asm!(include_str!("init/entry.S"));

/// Entry point for the kernel, called by _entry() in init/entry.S
#[no_mangle]
pub fn kernel_init() -> ! {


    loop {

    }
}
