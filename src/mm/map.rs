//! This module contains the implementation of page entry table, page directory table, and related functions.

use crate::error::MosError;

use super::{
    addr::{PA, PPN, VA},
    layout::{PteFlags, PTE_HARDFLAG_SHIFT},
    page::{page_alloc, page_inc_ref, try_recycle, Page},
    tlb::tlb_invalidate,
};

/// Page table entry
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Pte(pub usize);

impl Pte {
    /// Construct a page entry by page's ppn
    /// Flags are set for this entry
    pub const fn new(ppn: PPN, flags: PteFlags) -> Self {
        Self(ppn.0 << 12 | flags.bits())
    }

    /// Construct an empty entry
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Acquire ppn of this entry
    pub const fn ppn(self) -> PPN {
        PPN(self.0 >> 12)
    }

    /// Acquire flags of this entry
    pub const fn flags(self) -> PteFlags {
        PteFlags::from_bits_truncate(self.0 & 0xFFF)
    }

    /// Acquire address of this entry
    pub fn addr(self) -> PA {
        self.ppn().into()
    }

    /// Set ppn and flags of this entry
    pub fn set(&mut self, ppn: PPN, flags: PteFlags) {
        self.0 = ppn.0 << 12 | flags.bits();
    }

    /// Set flag bits provided to this pte
    /// # Arguments
    /// * flags: Bit flags set to pte
    pub fn set_flags(&mut self, flags: PteFlags) {
        self.0 &= !0xFFF;
        self.0 |= flags.bits();
    }

    /// Check if this pte is valid(contains PteFlags::V)
    pub const fn is_valid(self) -> bool {
        self.flags().contains(PteFlags::V)
    }

    /// Get entry LO of this Pte
    pub const fn as_entrylo(self) -> u32 {
        self.0 as u32 >> PTE_HARDFLAG_SHIFT
    }
}

/// Page directory entry defination
pub type Pde = Pte;

/// Page directory
#[derive(Clone, Copy, Debug)]
pub struct PageTable {
    /// Actual page of page table
    pub page: Page,
}

impl PageTable {
    /// Initialize a new page table
    /// A page is allocated for the page table
    ///
    /// # Returns
    ///
    /// A tuple containing the page table and the page
    pub fn init() -> Result<(Self, Page), MosError> {
        page_alloc(true).map_or(Err(MosError::NoMem), |page| {
            page_inc_ref(page);
            Ok((PageDirectory { page }, page))
        })
    }

    /// create an empty page table
    ///
    /// # Returns
    ///
    /// An empty page table with page set to PPN(0)
    pub const fn empty() -> Self {
        Self {
            page: Page::new(PPN(0)),
        }
    }

    /// Return pte at this page's offset
    #[allow(clippy::mut_from_ref)]
    pub fn pte_at(&self, offset: usize) -> &mut Pte {
        let base_pd: *mut Pde = self.page.kaddr().as_mut_ptr::<Pde>();
        unsafe { &mut *base_pd.add(offset) }
    }

    /// Return pte at va of this page table directory
    /// Return ``MosError::NoMem`` if create is set and page allocation failed
    ///
    /// # Arguments
    /// * va: Virtual address for target pte
    /// * create: Create a new page if pte is not valid
    ///
    pub fn walk(&self, va: VA, create: bool) -> Result<Option<&mut Pte>, MosError> {
        let pde = self.pte_at(va.pdx());

        if !pde.flags().contains(PteFlags::V) {
            if !create {
                return Ok(None);
            }
            if let Some(page) = page_alloc(true) {
                page_inc_ref(page);
                pde.set(page.ppn(), PteFlags::V | PteFlags::Cacheable);
            } else {
                return Err(MosError::NoMem);
            }
        }
        let base_pt = pde.addr().kaddr().as_mut_ptr::<Pte>();
        let ret = unsafe { &mut *base_pt.add(va.ptx()) };

        Ok(Some(ret))
    }

    /// Map the physical page at virtual address va,
    /// the lower 12 bits of pte will be set to flags
    ///
    /// # Returns
    ///
    /// ``Ok(())`` if page is successfully inserted
    /// ``MosError::NoMem`` if page allocation failed
    pub fn insert(self, asid: usize, page: Page, va: VA, flags: PteFlags) -> Result<(), MosError> {
        let ppn = page.ppn();
        if let Ok(Some(pte)) = self.walk(va, false) {
            if pte.flags().contains(PteFlags::V) {
                if ppn == pte.ppn() {
                    tlb_invalidate(asid, va);
                    pte.set_flags(flags | PteFlags::V | PteFlags::Cacheable);
                    return Ok(());
                }
                self.remove(asid, va);
            }
        }

        tlb_invalidate(asid, va);

        if let Ok(Some(pte)) = self.walk(va, true) {
            *pte = Pte::new(ppn, flags | PteFlags::V | PteFlags::Cacheable);
            page_inc_ref(page);
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

    /// Unmap the page at virtual address va
    pub fn remove(self, asid: usize, va: VA) {
        if let Some((pte, page)) = self.lookup(va) {
            tlb_invalidate(asid, va);
            try_recycle(page);
            *pte = Pte::empty();
        }
    }
}

/// Page directory defination
pub type PageDirectory = PageTable;
