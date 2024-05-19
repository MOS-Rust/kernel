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
fn panic(info: &core::panic::PanicInfo) -> ! {
    let sp: u32;
    let ra: u32;
    let badva: u32;
    let sr: u32;
    let cause: u32;
    let epc: u32;

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
        );
    }
    error!(
        "Kernel Panicked: \"{}\" at {}",
        info.message().unwrap(),
        info.location().unwrap()
    );
    println!(
        "ra:    0x{:08x}  sp:  0x{:08x}  Status: 0x{:08x}\nCause: 0x{:08x}  EPC: 0x{:08x}  BadVA:  0x{:08x}",
        ra, sp, sr, cause, epc, badva
    );
    unsafe {
        let envid = match crate::pm::ENV_MANAGER.curenv() {
            Some(env) => env.id,
            None => 0,
        };
        println!("envid: {:08x}", envid);
        println!(
            "cur_pgdir: 0x{:08x}",
            crate::pm::ENV_MANAGER.current_pgdir().page.kaddr().0
        );
    }
    match option_env!("MOS_HANG_ON_PANIC") {
        Some("1") => loop {},
        _ => halt(),
    }
}
