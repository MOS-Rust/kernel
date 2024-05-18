use core::arch::global_asm;

use log::warn;

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
    if (USTACKTOP..USTACKTOP + PAGE_SIZE).contains(&va_val) {
        panic!("Passive alloc: invalid address.");
    }
    if (UENVS..UPAGES).contains(&va_val) {
        panic!("Passive alloc: envs zone.");
    }
    if (UPAGES..UVPT).contains(&va_val) {
        warn!("{:x}", va_val);
        panic!("Passive alloc: pages zone.");
    }
    if va_val >= ULIM {
        panic!("Passive alloc: kernel address");
    }

    let page = page_alloc(true).unwrap();
    let flags = if (UVPT..ULIM).contains(&va_val) {
        PteFlags::empty()
    } else {
        PteFlags::D
    };
    pgdir.insert(asid, page, va.pte_addr(), flags).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn do_tlb_refill(pentrylo: *mut u32, va: u32, asid: u32) {
    let va = VA(va as usize);
    let asid = asid as usize;
    tlb_invalidate(asid, va);

    let pte_base: *mut Pte;
    loop {
        if let Some((pte, _)) = ENV_MANAGER.current_pgdir().lookup(va) {
            pte_base = ((pte as *mut Pte as usize) & !0x7) as *mut Pte;
            break;
        }
        passive_alloc(va, ENV_MANAGER.current_pgdir(), asid);
    }
    pentrylo.write_volatile((*pte_base).as_entrylo());
    pentrylo
        .add(1)
        .write_volatile((*pte_base.add(1)).as_entrylo());
}
