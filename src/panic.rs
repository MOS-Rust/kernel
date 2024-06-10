//! Panic handler
//!
//! This module provides the panic handler for the kernel. When a panic occurs, this handler is called
//! to print diagnostic information and halt the machine.
//!
//! By default, the machine will halt when a panic occurs. If the environment variable `MOS_HANG_ON_PANIC` is set to `1`,
//! the machine will hang instead of halting.

use core::{arch::asm, mem::size_of};

use alloc::{format, string::String};
use log::error;

use crate::mutex::Mutex;
use crate::{
    mm::layout::{KSEG0, KSEG1}, platform::halt, pm::ENV_MANAGER, println
};

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
    let backtrace = backtrace();
    error!(
        "Kernel Panicked: \"{}\" at {}\n{}",
        info.message().unwrap(),
        info.location().unwrap(),
        backtrace
    );
    println!(
        "ra:    0x{:08x}  sp:  0x{:08x}  Status: 0x{:08x}\nCause: 0x{:08x}  EPC: 0x{:08x}  BadVA:  0x{:08x}",
        ra, sp, sr, cause, epc, badva
    );
    unsafe { ENV_MANAGER.force_unlock() };
    let envid = match ENV_MANAGER.lock().curenv() {
        Some(env) => env.id,
        None => 0,
    };
    println!("envid: {:08x}", envid);
    println!(
        "cur_pgdir: 0x{:08x}",
        ENV_MANAGER.lock().cur_pgdir().page.kaddr().0
    );
    match option_env!("MOS_HANG_ON_PANIC") {
        Some("1") => loop {},
        _ => halt(),
    }
}

fn backtrace() -> String {
    let mut current_ra: usize;
    let mut current_sp: usize;

    unsafe {
        asm!(
            "move $8, $29",
            "move $9, $31",
            out("$8") current_sp,
            out("$9") current_ra,
        );
    }
    let mut result = String::new();
    unsafe {
        // only works when  "-C", "force-frame-pointers=yes" is set
        // haven't investigated why
        let mut depth = 0;
        let mut stack_size: usize = 0;
        let mut ra_offset: usize;
        const INST_OP_MASK: usize = 0xffff0000;
        // ADDIU SP, SP, IMM
        const ADDIU_SP_INST: usize = 0x27bd0000;
        const JR_RA_INST: usize = 0x03e00008;
        const SW_RA_INST: usize = 0xafbf0000;

        result.push_str("Backtrace:\n");

        let mut current_addr = backtrace as *const () as usize;

        while stack_size == 0 {
            let inst = *(current_addr as *const usize);
            if (inst & INST_OP_MASK) == ADDIU_SP_INST {
                stack_size = ((inst & 0xffff) as i16).unsigned_abs() as usize; // Bytes
            } else if inst == JR_RA_INST {
                break;
            }
            current_addr += size_of::<usize>();
        }

        current_sp += stack_size;

        // only track backtrace within kernel space
        while (KSEG0..KSEG1).contains(&current_ra) {
            result.push_str(&format!(
                "  {:02}: RA:0x{:08x} SP:0x{:08x}",
                depth, current_ra, current_sp
            ));
            depth += 1;
            current_addr = current_ra;
            ra_offset = 0;
            stack_size = 0;
            while stack_size == 0 || ra_offset == 0 {
                let inst = *(current_addr as *const usize);
                if (inst & INST_OP_MASK) == ADDIU_SP_INST {
                    stack_size = ((inst & 0xffff) as i16).unsigned_abs() as usize;
                    // Bytes
                } else if (inst & INST_OP_MASK) == SW_RA_INST {
                    ra_offset = inst & 0xffff; // Bytes
                    stack_size = 0; // Stack size is always set BEFORE ra_offset,
                                    // so anything after that is garbage
                } else if inst == 0x3c1c0000 {
                    return result;
                }
                current_addr -= size_of::<usize>();
            }

            current_ra = *((current_sp + ra_offset) as *const usize);
            current_sp += stack_size;
            result.push_str(&format!(
                " Subroutine:0x{:08x}\n",
                current_addr + size_of::<usize>()
            ));
        }
    }
    result
}
