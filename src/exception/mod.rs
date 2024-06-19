mod clock;
mod handlers;
mod trapframe;

use core::{
    arch::{asm, global_asm},
    ptr::addr_of_mut,
};

use log::info;

pub use clock::reset_kclock;
pub use trapframe::{Trapframe, TF_SIZE};

global_asm!(include_str!("../../asm/exception/exception_entry.S"));
global_asm!(include_str!("../../asm/exception/handlers.S"));

#[repr(C)]
pub struct Vector(unsafe extern "C" fn());

extern "C" {
    fn _handle_int();
    fn _handle_tlb();
    fn _handle_mod();
    fn _handle_syscall();
    fn _handle_unhandled();
    fn _handle_ade();
}

/// Exception handler vector
#[no_mangle]
pub static exception_handlers: [Vector; 32] = [
    Vector(_handle_int),       // 00: Int
    Vector(_handle_mod),       // 01: Mod
    Vector(_handle_tlb),       // 02: TLBL
    Vector(_handle_tlb),       // 03: TLBS
    Vector(_handle_ade),       // 04: AdEL
    Vector(_handle_ade),       // 05: AdES
    Vector(_handle_unhandled), // 06
    Vector(_handle_unhandled), // 07
    Vector(_handle_syscall),   // 08: Syscall
    Vector(_handle_unhandled), // 09
    Vector(_handle_unhandled), // 10
    Vector(_handle_unhandled), // 11
    Vector(_handle_unhandled), // 12
    Vector(_handle_unhandled), // 13
    Vector(_handle_unhandled), // 14
    Vector(_handle_unhandled), // 15
    Vector(_handle_unhandled), // 16
    Vector(_handle_unhandled), // 17
    Vector(_handle_unhandled), // 18
    Vector(_handle_unhandled), // 19
    Vector(_handle_unhandled), // 20
    Vector(_handle_unhandled), // 21
    Vector(_handle_unhandled), // 22
    Vector(_handle_unhandled), // 23
    Vector(_handle_unhandled), // 24
    Vector(_handle_unhandled), // 25
    Vector(_handle_unhandled), // 26
    Vector(_handle_unhandled), // 27
    Vector(_handle_unhandled), // 28
    Vector(_handle_unhandled), // 29
    Vector(_handle_unhandled), // 30
    Vector(_handle_unhandled), // 31
];

/// Init exception handling feature
pub fn init() {
    extern "C" {
        static mut _tlb_refill_entry: u8;
    }
    unsafe {
        asm!(
            ".set noat",
            "mtc0 {}, $15, 1",
            ".set at",
            in(reg) addr_of_mut!(_tlb_refill_entry) as u32,
        );
        info!(
            "Exception entry set at 0x{:08x}",
            addr_of_mut!(_tlb_refill_entry) as u32
        );
    }
}
