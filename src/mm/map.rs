//! This module contains the implementation of page entry table, page directory table, and related functions.
use crate::error::MosError;

use super::{
    addr::{PA, PPN, VA},
    layout::PteFlags,
    page::{find_page, inc_ref, page_alloc, page_dealloc, page_dec_ref, page_inc_ref, Page},
    tlb::tlb_invalidate,
};

/// Page table entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pte(pub usize);

impl Pte {
    /// construct a page entry by page's ppn
    /// flags are set for this entry
    pub fn new(ppn: PPN, flags: PteFlags) -> Pte {
        Pte(ppn.0 << 10 | flags.bits())
    }

    /// construct an empty entry
    pub fn empty() -> Pte {
        Pte(0)
    }

    /// acquire ppn of this entry
    pub fn ppn(self) -> PPN {
        PPN(self.0 >> 10)
    }

    /// acquire flags of this entry
    pub fn flags(self) -> PteFlags {
        PteFlags::from_bits_truncate(self.0 & 0x3FF)
    }

    /// acquire address of this entry
    pub fn addr(self) -> PA {
        self.ppn().into()
    }

    /// set ppn and flags of this entry
    // is this method necessary? (we can construct new entry instead modify old ones)
    pub fn set(&mut self, ppn: PPN, flags: PteFlags) {
        self.0 = ppn.0 << 10 | flags.bits();
    }

    pub fn flags_mut(&mut self) -> &mut PteFlags {
        unsafe { &mut *(self as *mut Pte as *mut PteFlags) }
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PteFlags::V)
    }
}

pub type Pde = Pte;

/// Page directory
#[derive(Clone, Copy, Debug)]
pub struct PageTable {
    pub page: Page,
}

impl PageTable {
    /// Initialize a new page table
    /// A page is allocated for the page table
    ///
    /// # Returns
    ///
    /// A tuple containing the page table and the page
    pub fn init() -> Result<(PageTable, Page), MosError> {
        if let Some(page) = page_alloc(true) {
            page_inc_ref(page);
            return Ok((PageTable { page }, page))
        } else {
            return Err(MosError::NoMem);
        }
    }

    // pub fn kaddr(&self) -> VA {
    //     self.page.ppn().kaddr()
    // }

    pub const fn empty() -> PageTable {
        PageTable {
            page: Page::new(PPN(0)),
        }
    }

    /// return pte at this page's offset
    pub fn pte_at(&self, offset: usize) -> &mut Pte {
        let base_pd: *mut Pde = self.page.ppn().kaddr().as_mut_ptr::<Pde>();
        unsafe { &mut *base_pd.add(offset) }
    }

    /// return pte at va of this page table directory
    /// return MosError::NoMem if create is set and page allocation failed
    ///
    /// # arguments
    /// * va: virtual address for target pte
    /// * create: create a new page if pte is not valid
    ///
    pub fn walk(&self, va: VA, create: bool) -> Result<Option<&mut Pte>, MosError> {
        let pte = self.pte_at(va.pdx());

        if !pte.flags().contains(PteFlags::V) {
            if !create {
                return Ok(None);
            }
            if let Some(page) = page_alloc(true) {
                page_inc_ref(page);
                pte.set(page.ppn(), PteFlags::V | PteFlags::Cacheable);
            } else {
                return Err(MosError::NoMem);
            }
        }
        let base_pt = pte.addr().kaddr().as_mut_ptr::<Pte>();
        let ret = unsafe { &mut *base_pt.add(va.ptx()) };

        Ok(Some(ret))
    }

    /// Map the physical page at virtual address va,
    /// the lower 12 bits of pte will be set to flags
    ///
    /// # Returns
    ///
    /// Ok(()) if page is successfully inserted
    /// MosError::NoMem if page allocation failed
    pub fn insert(&self, asid: usize, page: Page, va: VA, flags: PteFlags) -> Result<(), MosError> {
        let ppn = page.ppn();
        if let Ok(Some(pte)) = self.walk(va, false) {
            if pte.flags().contains(PteFlags::V) {
                if ppn == pte.ppn() {
                    tlb_invalidate(asid, va);
                    *pte.flags_mut() = flags | PteFlags::V | PteFlags::Cacheable;
                    return Ok(());
                } else {
                    self.remove(asid, va);
                }
            }
        }

        tlb_invalidate(asid, va);

        if let Ok(Some(pte)) = self.walk(va, true) {
            *pte = Pte::new(ppn, flags | PteFlags::V | PteFlags::Cacheable);
            inc_ref(ppn);
            Ok(())
        } else {
            Err(MosError::NoMem)
        }
    }

    /// Lookup the page that virtual address va is mapped to
    ///
    /// # Returns
    ///
    /// Ok((pte, page)) if page found valid
    /// * pte: &mut Pte, page table entry of va
    /// * page: Page, page of va
    ///
    /// None if page not found valid
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

    /// unmap the page at virtual address va
    pub fn remove(&self, asid: usize, va: VA) {
        match self.lookup(va) {
            Some((pte, page)) => {
                tlb_invalidate(asid, va);
                PageTable::try_recycle(page);
                *pte = Pte::empty();
            }
            None => {}
        }
    }

    /// decrease the ref_count of page
    /// if page's ref_count is set to 0, deallocate the page
    pub fn try_recycle(page: Page) {
        if let Some(tracker) = find_page(page) {
            match tracker.ref_count() {
                0 => {
                    panic!("PageTable::try_recycle: page is not referenced.");
                }
                1 => {
                    page_dec_ref(page);
                    page_dealloc(page);
                }
                _ => {
                    page_dec_ref(page);
                }
            }
        }
    }
}

pub type PageDirectory = PageTable;

impl PageDirectory {
    /// convert virtual address va to physical address pa in current page directory
    pub fn va2pa(&self, va: VA) -> Option<PA> {
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
