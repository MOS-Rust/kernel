#![allow(dead_code)] // TODO: Remove this

use core::arch::global_asm;

use super::{addr::{VA, VPN}, layout::{PteFlags, PAGE_SIZE, UENVS, UPAGES, USTACKTOP, UTEMP, UVPT}, map::PageDirectory, page::page_alloc};

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

pub fn passive_alloc(va: VA, pgdir: PageDirectory, asid: usize) {
    let va_val = va.0;
    if va_val < UTEMP {
        panic!("Passive alloc: address too low.");
    }
    if va_val >= USTACKTOP && va_val < USTACKTOP + PAGE_SIZE {
        panic!("Passive alloc: invalid address.");
    }
    if va_val >= UENVS && va_val < UPAGES {
        panic!("Passive alloc: envs zone.");
    }
    if va_val >= UPAGES && va_val < UVPT {
        panic!("Passive alloc: pages zone.");
    }
    if va_val >= va_val {
        panic!("Passive alloc: kernel address");
    }

    let page = page_alloc(true).unwrap();
    let flags = if va_val >= UVPT && va_val < UVPT + PAGE_SIZE {
        PteFlags::empty()
    } else {
        PteFlags::D
    };
    pgdir.insert(asid, page, va.pte_addr(), flags).unwrap();
}

/// This function returns (entrylo0, entrylo1).
pub fn do_tlb_refill(va: VA, asid: usize) -> (u32, u32) {
    tlb_invalidate(asid, va);
    // TODO:
    (0, 0)
}