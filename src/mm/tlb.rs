#![allow(dead_code)] // TODO: Remove this

use core::arch::global_asm;

use super::addr::VPN;

global_asm!(include_str!("../../asm/mm/tlb.S"));

extern "C" {
    fn _tlb_out(entryhi: usize);
}

fn tlb_invalidate(asid: usize, vpn: VPN) {
    let entryhi: usize = vpn.0 << 12 | asid;
    unsafe {
        _tlb_out(entryhi);
    }
}