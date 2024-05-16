#![allow(dead_code)] // TODO: Remove this

use core::arch::global_asm;

use crate::pm::ENV_MANAGER;

use super::{
    addr::{VA, VPN},
    layout::{PteFlags, PAGE_SIZE, UENVS, ULIM, UPAGES, USTACKTOP, UTEMP, UVPT},
    map::{PageDirectory, Pte},
    page::page_alloc,
};

global_asm!(include_str!("../../asm/mm/tlb.S"));

extern "C" {
    fn _tlb_out(entryhi: u32);
}

pub fn tlb_invalidate(asid: usize, va: VA) {
    let entryhi: u32 = (VPN::from(va).0 << 12 | asid) as u32;
    unsafe {
        _tlb_out(entryhi);
    }
}

pub fn passive_alloc(va: VA, pgdir: &mut PageDirectory, asid: usize) {
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
    if va_val >= ULIM {
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


// TODO: NOT GUARANTEED TO WORK
#[no_mangle]
pub unsafe extern "C" fn do_tlb_refill(pentrylo: *mut u32, va: u32, asid: u32) {
    let va = VA(va as usize);
    let asid = asid as usize;
    tlb_invalidate(asid, va);

    let pte_base: *mut Pte;
    loop {
        if let Some((pte, _)) = ENV_MANAGER.current_pgdir().lookup(va) {
            pte_base = ((pte as *mut Pte as *mut _ as usize) & !0x7) as *mut Pte;
            break;
        }
        passive_alloc(va, ENV_MANAGER.current_pgdir(), asid);
    }
    pentrylo.write_volatile((*pte_base).0 as u32 >> 6);
    pentrylo.add(1).write_volatile((*pte_base.add(1)).0 as u32 >> 6);
}
