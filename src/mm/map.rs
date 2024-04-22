#![allow(dead_code)]
use core::ops::Deref;
use crate::impl_usize;
use super::addr::{PA, PPN, VA};
use super::super::error::MosError;
use super::layout::PteFlag;
use super::page::PAGE_ALLOCATOR;

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]

pub struct Pde(pub usize);

impl_usize!(Pde);

impl Pde {
    pub fn addr(&self) -> usize {
        let addr: usize = self.0;
        addr & !0xfff
    }

    pub fn flags(&self) -> usize {
        let addr: usize = self.0;
        addr & 0xfff
    }

    pub fn item(&self, offset: usize) -> Pte {
        let addr: *const Pte = (self.0 + offset) as *const Pte;
        unsafe { *addr }
    }

    pub fn set_item(&self, offset: usize, value: usize) {
        let addr: *mut usize = (self.0 + offset) as *mut usize;
        unsafe {*addr = value};
    }
}

impl Pde {
    fn walk(&self, va: VA, create: bool) -> Result<Option<Pte>, MosError> {
        let pte = self.item(va.pdx());
        let ppn: PPN;
        let ret: Pte;

        let value: usize = pte.into();
        if value & PteFlag::V as usize == 0 {
            if create {
                let result = unsafe {PAGE_ALLOCATOR.alloc(true)};
                match result {
                    Ok(value) => ppn = value,
                    Err(err) => return Err(err),
                }
                if let Some(page) = unsafe {PAGE_ALLOCATOR.find_page(ppn)} {
                    page.inc_ref();
                    let value: usize = page.to_pa().into();
                    let set_value 
                        = value | PteFlag::V as usize | PteFlag::Cacheable as usize;
                    self.set_item(va.pdx(), set_value);
                    ret = Pte(page.to_kva().into());
                } else {
                    panic!("invalid ppn");
                }
            } else {
                return Ok(None);
            }
        } else {
            ret = Pte(PA::from(pte.addr()).kaddr().into());
        }
        let value: usize = ret.into();

        Ok(Some(Pte(value + va.ptx())))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Pte(pub usize);

impl_usize!(Pte);

impl Pte {
    pub fn addr(&self) -> usize {
        let addr: usize = self.0;
        addr & !0xfff
    }

    pub fn flags(&self) -> usize {
        let addr: usize = self.0;
        addr & 0xfff
    }
}