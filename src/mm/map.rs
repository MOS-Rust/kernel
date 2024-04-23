#![allow(dead_code)]

use core::slice::from_raw_parts_mut;

use crate::error::Error;

use super::{
    addr::{PA, PPN, VA},
    layout::PteFlags,
    page::{alloc, dealloc, dec_ref, find_page, inc_ref}, tlb::tlb_invalidate,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pte(pub usize);

impl Pte {
    pub fn new(ppn: PPN, flags: PteFlags) -> Pte {
        Pte(ppn.0 << 10 | flags.bits())
    }

    pub fn empty() -> Pte {
        Pte(0)
    }

    pub fn ppn(self) -> PPN {
        PPN(self.0 >> 10)
    }

    pub fn flags(self) -> PteFlags {
        PteFlags::from_bits_truncate(self.0 & 0x3FF)
    }

    pub fn addr(self) -> PA {
        self.ppn().into()
    }

    pub fn set(&mut self, ppn: PPN, flags: PteFlags) {
        self.0 = ppn.0 << 10 | flags.bits();
    }

    pub fn flags_mut(&mut self) -> &mut PteFlags {
        unsafe { &mut *(self as *mut Pte as *mut PteFlags) }
    }
}

pub struct PageTable {
    ppn: PPN,
}

impl PageTable {
    pub fn new() -> PageTable {
        let ppn = alloc(true).expect("Failed to allocate a page for page table.");
        inc_ref(ppn);
        PageTable { ppn }
    }

    pub fn walk(&self, va: VA, create: bool) -> Result<Option<&mut Pte>, Error> {
        let base_ptr = PA::from(self.ppn.0).kaddr().0 as *mut Pte;
        let slice = unsafe { from_raw_parts_mut(base_ptr, 1024) };
        let pte = &mut slice[va.pdx()];
        if !pte.flags().contains(PteFlags::V) {
            if !create {
                return Ok(None);
            }
            let ppn = alloc(true);
            if ppn.is_none() {
                return Err(Error::NoMem);
            }
            inc_ref(ppn.unwrap());
            pte.set(ppn.unwrap(), PteFlags::V | PteFlags::Cached);
        }
        Ok(Some(pte))
    }

    pub fn insert(&self, asid: usize, ppn: PPN, va: VA, flags: PteFlags) -> Result<(), Error> {
        let pte = self.walk(va, false);
        if let Ok(Some(pte)) = pte {
           if pte.flags().contains(PteFlags::V) {
                if ppn == pte.ppn() {
                    tlb_invalidate(asid, va);
                    *pte.flags_mut() = flags | PteFlags::V | PteFlags::Cached;
                    return Ok(());
                } else {
                    self.remove(asid, va);
                }
           }
        }
        tlb_invalidate(asid, va);
        let pte = self.walk(va, true);
        if let Ok(Some(pte)) = pte {
            *pte = Pte::new(ppn, flags | PteFlags::V | PteFlags::Cached);
            inc_ref(ppn);
            Ok(())
        } else {
            Err(Error::NoMem)
        }
    }

    pub fn lookup(&self, va: VA) -> Option<(&mut Pte, PPN)> {
        let pte = self.walk(va, false);
        if let Ok(Some(pte)) = pte {
            if pte.flags().contains(PteFlags::V) {
                let ppn = pte.ppn();
                return Some((pte, ppn));
            }
        }
        None
    }

    pub fn remove(&self, asid: usize, va: VA) {
        match self.lookup(va) {
            Some((pte, ppn)) => {
                tlb_invalidate(asid, va);
                self.dec_ref(ppn);
                *pte = Pte::empty();
            },
            None => {},
        }
    }

    pub fn dec_ref(&self, ppn: PPN) {
        let page = find_page(ppn).unwrap();
        match page.ref_count() {
            0 => {
                panic!("PageTable::decref: page is not referenced.");
            },
            1 => {
                dec_ref(ppn);
                dealloc(ppn);
            },
            _ => {
                dec_ref(ppn);
            },
        }
    }
}

pub type PageDirectory = PageTable;
