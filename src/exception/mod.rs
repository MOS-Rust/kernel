#![allow(dead_code)] // TODO: Remove this

pub mod clock;
mod handlers;
pub mod trapframe;

use core::{arch::global_asm, mem::size_of, ptr::addr_of_mut};
use log::trace;
use mips::registers::cp0::{
    cause::{self, Exception}, ebase, epc, status
};

use crate::{exception::{handlers::{handle_int, handle_tlb, unhandled}, trapframe::Trapframe}, mm::{addr::VA, layout::KSTACKTOP}, platform::cp0reg::{STATUS_EXL, STATUS_IE, STATUS_UM}, println};

global_asm!(include_str!("../../asm/exception/exception_entry.S"));

#[no_mangle]
pub unsafe extern "C" fn exception_handler() {
    println!("epc:{:x}", epc::read());
    let tf = Trapframe::from_memory(VA(KSTACKTOP - size_of::<Trapframe>()));

    match cause::read_struct().exception() {
        Exception::Int => {
            handle_int();
        }
        Exception::TLBL => {
            handle_tlb();
        }
        Exception::TLBS => {
            trace!("");
            handle_tlb();
        }
        Exception::Mod => {
            panic!("Unhandled TLB exception");
        }
        Exception::Sys => {
            panic!("Unhandled syscall");
        }
        _ => {
            unhandled(tf);
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
