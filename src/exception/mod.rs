#![allow(dead_code)] // TODO: Remove this

mod clock;
pub mod trapframe;
mod handlers;

use core::arch::global_asm;
use mips::registers::cp0::{cause::{self, Exception}, status};

global_asm!(include_str!("../../asm/exception/exception_entry.S"));

#[no_mangle]
pub unsafe extern "C" fn exception_handler() {
    let mut status = status::read_struct();
    status.clear_exl();
    status.clear_ie();
    status.set_kernel_mode();
    status::write_struct(status);
    match cause::read_struct().exception() {
        Exception::Int => {
            panic!("Unhandled interrupt");
        }
        _ => {
            panic!("Unhandled exception");
        }
    }
}
