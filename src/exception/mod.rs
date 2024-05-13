#![allow(dead_code)] // TODO: Remove this

pub mod clock;
mod handlers;
pub mod trapframe;

use core::{arch::global_asm, ptr::addr_of_mut};
use mips::registers::cp0::{
    cause::{self, Exception},
    ebase,
    status,
};

use crate::println;

global_asm!(include_str!("../../asm/exception/exception_entry.S"));

#[no_mangle]
pub unsafe extern "C" fn exception_handler() {
    println!("Exception handler");
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

pub fn init() {
    extern "C" {
        static mut _exception_entry: u8;
    }
    unsafe {
        ebase::write(addr_of_mut!(_exception_entry) as u32);
    }
}
