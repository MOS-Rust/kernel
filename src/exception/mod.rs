#![allow(dead_code)] // TODO: Remove this
use core::arch::global_asm;

global_asm!(include_str!("../../asm/exception/exception_entry.S"));

#[no_mangle]
pub extern "C" fn exception_handler() {
    
}