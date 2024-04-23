#![allow(dead_code)] // TODO: Remove this

use core::arch::global_asm;

use super::addr::{VA, VPN};

global_asm!(include_str!("../../asm/mm/tlb.S"));

extern "C" {
    fn _tlb_out(entryhi: usize);
}

pub fn tlb_invalidate(asid: usize, va: VA) {
    let entryhi: usize = VPN::from(va).0 << 12 | asid;
    unsafe {
        _tlb_out(entryhi);
    }
}