//! Panic handler
//!
//! This module provides the panic handler for the kernel. When a panic occurs, this handler is called
//! to print diagnostic information and halt the machine.
//!
//! By default, the machine will halt when a panic occurs. If the environment variable `MOS_HANG_ON_PANIC` is set to `1`,
//! the machine will hang instead of halting.

use core::arch::asm;

use log::error;

use crate::{platform::halt, println};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let sp: usize;
    let ra: usize;
    let badva: usize;
    let sr: usize;
    let cause: usize;
    let epc: usize;

    unsafe {
        asm!(
            "move $8, $29",
            "move $9, $31",
            "mfc0 $10, $8",
            "mfc0 $11, $12",
            "mfc0 $12, $13",
            "mfc0 $13, $14",
            out("$8") sp,
            out("$9") ra,
            out("$10") badva,
            out("$11") sr,
            out("$12") cause,
            out("$13") epc,
        )
    }
    error!(
        "Kernel Panicked: \"{}\" at {}",
         _info.message().unwrap(), _info.location().unwrap()
    );
    println!(
        "ra:    0x{:08x}  sp:  0x{:08x}  Status: 0x{:08x}\nCause: 0x{:08x}  EPC: 0x{:08x}  BadVA:  0x{:08x}",
        ra, sp, sr, cause, epc, badva
    );
    match option_env!("MOS_HANG_ON_PANIC") {
        Some("1") => loop {},
        _ => {
            halt()
        }
    }
}