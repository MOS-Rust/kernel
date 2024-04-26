#![allow(dead_code)]

use log::debug;

use crate::error::MosError;

use super::{
    addr::{PA, PPN, VA},
    layout::PteFlags,
    page::{alloc, find_page, inc_ref, page_alloc, page_dealloc, page_dec_ref, page_inc_ref, Page}, tlb::tlb_invalidate,
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

pub type Pde = Pte;

pub struct PageTable {
    page: Page,
}

impl PageTable {
    pub fn init() -> (PageTable, Page) {
        let ppn = alloc(true).expect("Failed to allocate a page for PageTable.");
        let page = Page::new(ppn);
        page_inc_ref(page);
        (PageTable { page }, page)
    }

    fn pte_at(&self, offset: usize) -> &mut Pte {
        let base_pd: *mut Pde = self.page.ppn().kaddr().as_mut_ptr::<Pde>();
        unsafe { &mut *base_pd.add(offset) }
    }

    pub fn walk(&self, va: VA, create: bool) -> Result<Option<&mut Pte>, MosError> {
        let pte = self.pte_at(va.pdx());
        
        if !pte.flags().contains(PteFlags::V) {
            if !create {
                return Ok(None);
            }
            if let Some(page) = page_alloc(true) {
                page_inc_ref(page);
                pte.set(page.ppn(), PteFlags::V | PteFlags::Cached);
            } else {
                return Err(MosError::NoMem);
            }
        }
        let base_pt = pte.addr().kaddr().as_mut_ptr::<Pte>();
        let ret = unsafe { &mut *base_pt.add(va.ptx()) };
        
        Ok(Some(ret))
    }

    pub fn insert(&self, asid: usize, page: Page, va: VA, flags: PteFlags) -> Result<(), MosError> {
        let ppn = page.ppn();
        
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
            Err(MosError::NoMem)
        }
    }

    pub fn lookup(&self, va: VA) -> Option<(&mut Pte, Page)> {
        let pte = self.walk(va, false);
        if let Ok(Some(pte)) = pte {
            if pte.flags().contains(PteFlags::V) {
                let ppn = pte.ppn();
                return Some((pte, Page::new(ppn)));
            }
        }
        None
    }

    pub fn remove(&self, asid: usize, va: VA) {
        match self.lookup(va) {
            Some((pte, page)) => {
                tlb_invalidate(asid, va);
                PageTable::try_recycle(page);
                *pte = Pte::empty();
            },
            None => {},
        }
    }

    fn try_recycle(page: Page) {
        if let Some(tracker) = find_page(page) {
            match tracker.ref_count() {
                0 => {
                    panic!("PageTable::decref: page is not referenced.");
                },
                1 => {
                    page_dec_ref(page);
                    page_dealloc(page);
                },
                _ => {
                    page_dec_ref(page);
                },
            }
        }
    }

    unsafe fn nth(&self, n: usize) -> &mut Pte {
        assert!(n < 1024);
        let base_ptr = self.page.ppn().kaddr().as_mut_ptr::<Pte>();
        &mut *base_ptr.add(n)
    }
}

pub type PageDirectory = PageTable;

impl PageDirectory {
    fn va2pa(&self, va: VA) -> Option<PA> {
        let base_pd = self.page.ppn().kaddr().as_mut_ptr::<Pte>();
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
    let mut pages = [Page::new(PPN(0)); 3];
    let (pd, pd_page) = PageTable::init();
    assert!(pd_page.ref_count() == 1);
    for i in 0..3 {
        pages[i] = page_alloc(true).expect("Failed to allocate a page.");
    }

    // Test inserting into pd
    assert!(pd.insert(0, pages[0], VA(0x0), PteFlags::empty()).is_ok());
    assert!(pages[0].ref_count() == 1);
    let pde = unsafe { pd.nth(0) };
    assert!(pde.flags().contains(PteFlags::V) && pde.flags().contains(PteFlags::Cached));
    let pte = pd.lookup(VA(0x0)).unwrap().0;
    assert!(pte.flags().contains(PteFlags::V) && pte.flags().contains(PteFlags::Cached));
    assert_eq!(pd.va2pa(VA(0x0)).unwrap(), pages[0].into());

    // Inserting ppns[1] into 0x1000
    assert!(pd.insert(0, pages[1], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[1].into());
    assert!(pages[1].ref_count() == 1);

    // Replacing ppns[1] with ppns[2], ppns[1] should be deallocated
    assert!(pd.insert(0, pages[2], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[2].into());
    assert!(pages[1].ref_count() == 0);
    assert!(pages[2].ref_count() == 1);

    // Replacing ppns[2] with ppns[0], ppns[2] should be deallocated
    assert!(pd.insert(0, pages[0], VA(0x1000), PteFlags::empty()).is_ok());
    assert_eq!(pd.va2pa(VA(0x0)).unwrap(), pages[0].into());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[0].into());
    assert!(pages[0].ref_count() == 2);
    assert!(pages[2].ref_count() == 0);

    // Check if dealloc works
    let page2 = page_alloc(true).unwrap();
    let page1 = page_alloc(true).unwrap();
    assert_eq!(page1, pages[1]);
    assert_eq!(page2, pages[2]);
    page_dealloc(page1);
    page_dealloc(page2);

    // Test removing
    // Remove ppns[0] at 0x0, it should remain at 0x1000
    pd.remove(0, VA(0x0));
    assert!(pd.va2pa(VA(0x0)).is_none());
    assert_eq!(pd.va2pa(VA(0x1000)).unwrap(), pages[0].into());
    assert!(pages[0].ref_count() == 1);

    // Remove ppns[0] at 0x1000, it should be deallocated
    pd.remove(0, VA(0x1000));
    assert!(pd.va2pa(VA(0x1000)).is_none());
    assert!(pages[0].ref_count() == 0);
    let page0 = page_alloc(true).unwrap();
    assert_eq!(page0, pages[0]);
    page_dealloc(page0);

    // Free resources
    PageTable::try_recycle(pde.ppn().into());
    PageTable::try_recycle(pd_page);
    debug!("Mapping test passed!");
}
