//! Rust entry point for the kernel
//!
//! This crate is the entry point for the kernel. It is responsible for initializing the kernel and starting its execution.

#![deny(missing_docs)]
// #![deny(warnings)]
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
mod exception;
mod logging;
mod macros;
mod mm;
mod mutex;
mod panic;
mod platform;
mod pm;
mod syscall;

use crate::mutex::Mutex;
use core::{
    arch::global_asm,
    include_str,
    ptr::{addr_of_mut, write_bytes},
};
use log::info;
use pm::schedule;

global_asm!(include_str!("../asm/init/entry.S"));

/// Kernel initialization function
///
/// This function is the entry point of the kernel. It is called by ``_entry()`` in init/entry.S when the kernel starts and is responsible
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
    display_banner();
    logging::init();
    info!("MOS-Rust started!");
    exception::init();
    mm::init(ram_size);
    pm::init();

    // test6_1 pipe tasks
    // env_create!(testptelibrary, "../mos_exec/testptelibrary.b");
    // env_create!(fs_serv, "../mos_exec/serv.b");
    // env_create!(testpipe, "../mos_exec/testpipe.b");
    // env_create!(testpiperace, "../mos_exec/testpiperace.b");

    // test6_2 shell tasks
    env_create!(icode, "../mos_exec/icode.b");
    env_create!(fs_serv, "../mos_exec/serv.b");

    // memory pool task
    //env_create!(pool_test, "../mos_exec/pool_test.b");

    schedule(true);
}

/// Clear the .bss section
///
/// This function clears the `.bss` section of the kernel.
/// All memory locations in the `.bss` section (i.e. from ``__start_bss`` to ``__end_bss``) are set to 0 in this function.
fn clear_bss() {
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

fn display_banner() {
    const BANNER: &str = r#"
              __  __    ___     ___             ___                     _     
        o O O|  \/  |  / _ \   / __|    ___    | _ \   _  _     ___    | |_   
       o     | |\/| | | (_) |  \__ \   |___|   |   /  | +| |   (_-<    |  _|  
      TS__[O]|_|__|_|  \___/   |___/   _____   |_|_\   \_,_|   /__/_   _\__|  
     {======|_|"""""|_|"""""|_|"""""|_|     |_|"""""|_|"""""|_|"""""|_|"""""| 
    ./o--000'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-' 
    "#;
    println!("{}", BANNER);
}
