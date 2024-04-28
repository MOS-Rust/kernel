//! Address types and conversion functions

use core::cmp::{PartialEq, Eq, Ord};
use core::ops::{Add, Sub};

use super::get_pagenum;
use super::layout::{PDSHIFT, PGSHIFT, ULIM};

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

impl Add<usize> for PA {
    type Output = PA;
    fn add(self, rhs: usize) -> Self::Output {
        PA(self.0 + rhs)
    }
}

impl Add<usize> for VA {
    type Output = VA;
    fn add(self, rhs: usize) -> Self::Output {
        VA(self.0 + rhs)
    }
}

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

    pub fn pte_addr(&self) -> VA {
        VA(self.0 & !0xFFF)
    }

    // pub fn as_ptr<T>(&self) -> *const T {
    //     self.0 as *const T
    // }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
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

impl PPN {
    pub fn kaddr(&self) -> VA {
        PA::from(*self).kaddr()
    }
}