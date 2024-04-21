#![allow(dead_code)] // TODO: Remove this

use core::cmp::{PartialEq, Eq, Ord};
use core::convert::{From, Into};
use core::ops::{Add, Sub, Deref};

use super::get_pagenum;
use super::layout::{PDSHIFT, PGSHIFT, ULIM};

macro_rules! impl_usize {
    ($name:ident) => {
        impl From<usize> for $name {
            fn from(x: usize) -> Self {
                $name(x)
            }
        }
        impl Into<usize> for $name {
            fn into(self) -> usize {
                self.0
            }
        }
        impl Deref for $name {
            type Target = usize;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PA(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VA(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PPN(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VPN(pub usize);

impl_usize!(PA);
impl_usize!(VA);
impl_usize!(PPN);
impl_usize!(VPN);

impl From<PA> for PPN {
    fn from(pa: PA) -> Self {
        PPN(pa.0 >> PGSHIFT)
    }
}

impl From<PPN> for PA {
    fn from(ppn: PPN) -> Self {
        PA(ppn.0 << PGSHIFT)
    }
}

impl From<VA> for VPN {
    fn from(va: VA) -> Self {
        VPN(va.0 >> PGSHIFT)
    }
}

impl From<VPN> for VA {
    fn from(vpn: VPN) -> Self {
        VA(vpn.0 << PGSHIFT)
    }
}

impl PA {
    /// Translates from physical address to kernel virtual address
    pub fn kaddr(&self) -> VA {
        let ppn = PPN::from(*self);
        if ppn.0 >= get_pagenum() {
            panic!("PA::kaddr: Invalid physical address");
        }
        VA(self.0 + ULIM)
    }
}

impl VA {
    /// Page Directory Index
    pub fn pdx(&self) -> usize {
        (self.0 >> PDSHIFT) & 0x3ff
    }

    /// Page Table Index
    pub fn ptx(&self) -> usize {
        (self.0 >> PGSHIFT) & 0x3ff
    }

    /// Translates from kernel virtual address to physical address
    pub fn paddr(&self) -> PA {
        if self.0 < ULIM {
            panic!("VA::paddr: Invalid virtual address");
        }
        PA(self.0 - ULIM)
    }
}

impl Add<usize> for PPN {
    type Output = PPN;
    fn add(self, rhs: usize) -> Self::Output {
        PPN(self.0 + rhs)
    }
}

impl Add<usize> for VPN {
    type Output = VPN;
    fn add(self, rhs: usize) -> Self::Output {
        VPN(self.0 + rhs)
    }
}

impl Sub<PPN> for PPN {
    type Output = usize;
    fn sub(self, rhs: PPN) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<VPN> for VPN {
    type Output = usize;
    fn sub(self, rhs: VPN) -> Self::Output {
        self.0 - rhs.0
    }
}