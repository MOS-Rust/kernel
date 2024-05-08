//! Address types and conversion functions
//!
//! This module provides types and conversion functions for working with physical and virtual addresses in the kernel.
//! It defines the following types:
//! - `PA`: Represents a physical address.
//! - `VA`: Represents a virtual address.
//! - `PPN`: Represents a physical page number.
//! - `VPN`: Represents a virtual page number.
//!
//! The module also provides conversion functions between these types, as well as arithmetic operations on them.

use core::cmp::{Eq, Ord, PartialEq};
use core::ops::{Add, Sub};

use super::get_pagenum;
use super::layout::{PDSHIFT, PGSHIFT, ULIM};

/// Physical Address
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PA(pub usize);

/// Virtual Address
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VA(pub usize);

/// Physical Page Number
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PPN(pub usize);

/// Virtual Page Number
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
    /// 
    /// # Panics
    /// 
    /// Panics if the physical address is beyond the physical memory size
    /// 
    /// # Returns
    /// 
    /// The kernel virtual address
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
    /// 
    /// # Panics
    /// 
    /// Panics if the virtual address is not in the kernel space
    /// 
    /// # Returns
    /// 
    /// The physical address
    pub fn paddr(&self) -> PA {
        if self.0 < ULIM {
            panic!("VA::paddr: Invalid virtual address");
        }
        PA(self.0 - ULIM)
    }

    /// Get the page table entry address from the virtual address
    pub fn pte_addr(&self) -> VA {
        VA(self.0 & !0xFFF)
    }

    // /// Get the pointer from the virtual address
    // pub fn as_ptr<T>(&self) -> *const T {
    //     self.0 as *const T
    // }

    /// Get the mutable pointer from the virtual address
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
    /// Translates from physical page number to kernel virtual address
    /// 
    /// # Panics
    /// 
    /// Panics if the corresponding physical address is beyond the physical memory size
    /// 
    /// # Returns
    /// 
    /// The kernel virtual address
    pub fn kaddr(&self) -> VA {
        PA::from(*self).kaddr()
    }
}
