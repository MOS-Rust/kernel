#![allow(dead_code)]

use crate::println;

use crate::error::Error;

use super::{
    addr::{PA, PPN, VA},
    layout::PteFlags,
    page::{alloc, dealloc, dec_ref, find_page, inc_ref}, tlb::tlb_invalidate,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pde(pub usize);

impl Pde {
    
}

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
    pub fn init() -> (PageTable, PPN) {
        let ppn = alloc(true).expect("Failed to allocate a page for PageTable.");
        inc_ref(ppn);
        (PageTable { ppn }, ppn)
    }

    pub fn walk(&self, va: VA, create: bool) -> Result<Option<&mut Pte>, Error> {
        let base_pd = self.ppn.kaddr().as_mut_ptr::<Pte>();
        let pde = unsafe { &mut *base_pd.add(va.pdx()) };
        
        if !pde.flags().contains(PteFlags::V) {
            if !create {
                return Ok(None);
            }
            let ppn = alloc(true);
            if ppn.is_none() {
                return Err(Error::NoMem);
            }
            inc_ref(ppn.unwrap());
            pde.set(ppn.unwrap(), PteFlags::V | PteFlags::Cached);
        }
        let base_pt = pde.addr().kaddr().as_mut_ptr::<Pte>();
        let pte = unsafe { &mut *base_pt.add(va.ptx()) };
        Ok(Some(pte))
    }

    pub fn insert(&self, asid: usize, ppn: PPN, va: VA, flags: PteFlags) -> Result<(), Error> {
        if let Ok(Some(pte)) = self.walk(va, false) {
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
        if let Ok(Some(pte)) = self.walk(va, true) {
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
                PageTable::dec_ref(ppn);
                *pte = Pte::empty();
            },
            None => {},
        }
    }

    fn dec_ref(ppn: PPN) {
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

    unsafe fn nth(&self, n: usize) -> &mut Pte {
        assert!(n < 1024);
        let base_ptr = self.ppn.kaddr().as_mut_ptr::<Pte>();
        &mut *base_ptr.add(n)
    }
}

pub type PageDirectory = PageTable;

impl PageDirectory {
    fn va2pa(&self, va: VA) -> Option<PA> {
        let base_pd = self.ppn.kaddr().as_mut_ptr::<Pte>();
        let pde = unsafe { &*base_pd.add(va.pdx()) };
        if !pde.flags().contains(PteFlags::V) {
            return None;
        }
        let base_pt = pde.addr().kaddr().as_mut_ptr::<Pte>();
        let pte = unsafe { &*base_pt.add(va.ptx()) };
        if !pte.flags().contains(PteFlags::V) {
            return None;
        }
        Some(pte.addr())
    }
}

pub fn mapping_test() {
    let mut ppns = [PPN(0); 3];
    let (pd, pd_ppn) = PageTable::init();
    assert!(find_page(pd_ppn).unwrap().ref_count() == 1);
    for i in 0..3 {
        ppns[i] = alloc(true).expect("Failed to allocate a page.");
    }

    // Test inserting into pd
    assert!(pd.insert(0, ppns[0], VA(0x0), PteFlags::empty()).is_ok());
    assert!(find_page(ppns[0]).unwrap().ref_count() == 1);
    let pde = unsafe { pd.nth(0) };
    assert!(pde.flags().contains(PteFlags::V) && pde.flags().contains(PteFlags::Cached));
    let pte = pd.lookup(VA(0x0)).unwrap().0;
    assert!(pte.flags().contains(PteFlags::V) && pte.flags().contains(PteFlags::Cached));
    assert_eq!(pd.va2pa(VA(0x0)).unwrap(), ppns[0].into());

    // Inserting ppns[1] into 0x1000
    assert!(pd.insert(0, ppns[1], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), ppns[1].into());
    assert!(find_page(ppns[1]).unwrap().ref_count() == 1);

    // Replacing ppns[1] with ppns[2], ppns[1] should be deallocated
    assert!(pd.insert(0, ppns[2], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), ppns[2].into());
    assert!(find_page(ppns[1]).unwrap().ref_count() == 0);
    assert!(find_page(ppns[2]).unwrap().ref_count() == 1);

    // Replacing ppns[2] with ppns[0], ppns[2] should be deallocated
    assert!(pd.insert(0, ppns[0], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x0)).unwrap(), ppns[0].into());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), ppns[0].into());
    assert!(find_page(ppns[0]).unwrap().ref_count() == 2);
    assert!(find_page(ppns[2]).unwrap().ref_count() == 0);

    // Check if dealloc works
    let ppn2 = alloc(true).unwrap();
    let ppn1 = alloc(true).unwrap();
    assert_eq!(ppn1, ppns[1]);
    assert_eq!(ppn2, ppns[2]);
    dealloc(ppn1);
    dealloc(ppn2);

    // Test removing
    // Remove ppns[0] at 0x0, it should remain at 0x1000
    pd.remove(0, VA(0x0));
    assert!(pd.va2pa(VA(0x0)).is_none());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), ppns[0].into());
    assert!(find_page(ppns[0]).unwrap().ref_count() == 1);

    // Remove ppns[0] at 0x1000, it should be deallocated
    pd.remove(0, VA(0x1000));
    assert!(pd.va2pa(VA(0x1000)).is_none());
    assert!(find_page(ppns[0]).unwrap().ref_count() == 0);
    let ppn0 = alloc(true).unwrap();
    assert_eq!(ppn0, ppns[0]);
    dealloc(ppn0);

    // Free resources
    PageTable::dec_ref(pde.ppn());
    PageTable::dec_ref(pd_ppn);
    println!("Mapping test passed!");
}
