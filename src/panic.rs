//! Panic handler

use core::arch::asm;

use crate::{platform::halt, println};

#[cfg(target_arch = "mips")]
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
            "move {sp}, $29",
            "move {ra}, $31",
            "mfc0 {badva}, $8",
            "mfc0 {sr}, $12",
            "mfc0 {cause}, $13",
            "mfc0 {epc}, $14",
            sp = out(reg) sp,
            ra = out(reg) ra,
            badva = out(reg) badva,
            sr = out(reg) sr,
            cause = out(reg) cause,
            epc = out(reg) epc,
        )
    }
    println!(
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