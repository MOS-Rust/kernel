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

use super::{
    get_pagenum,
    layout::{KSEG0, PDSHIFT, PGSHIFT},
};
use core::{
    cmp::{Eq, Ord, PartialEq},
    ops::{Add, Sub},
};

/// Physical Address
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PA(pub usize);

/// Virtual Address
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VA(pub usize);

/// Physical Page Number
#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PPN(pub usize);

/// Virtual Page Number
#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VPN(pub usize);

impl Add<usize> for PA {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<usize> for VA {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<PA> for PPN {
    fn from(pa: PA) -> Self {
        Self(pa.0 >> PGSHIFT)
    }
}

impl From<PPN> for PA {
    fn from(ppn: PPN) -> Self {
        Self(ppn.0 << PGSHIFT)
    }
}

impl From<VA> for VPN {
    fn from(va: VA) -> Self {
        Self(va.0 >> PGSHIFT)
    }
}

impl From<VPN> for VA {
    fn from(vpn: VPN) -> Self {
        Self(vpn.0 << PGSHIFT)
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
    pub fn kaddr(self) -> VA {
        let ppn = PPN::from(self);
        assert!(ppn.0 < get_pagenum(), "PA::kaddr: Invalid physical address");
        VA(self.0 + KSEG0)
    }
}

impl VA {
    /// Page Directory Index
    pub const fn pdx(self) -> usize {
        (self.0 >> PDSHIFT) & 0x3ff
    }

    /// Page Table Index
    pub const fn ptx(self) -> usize {
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
    pub fn paddr(self) -> PA {
        assert!(self.0 >= KSEG0, "VA::paddr: Invalid virtual address");
        PA(self.0 - KSEG0)
    }

    /// Get the page table entry address from the virtual address
    pub const fn pte_addr(self) -> Self {
        Self(self.0 & !0xFFF)
    }

    /// Get the pointer from the virtual address
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    /// Get the mutable pointer from the virtual address
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl Add<usize> for PPN {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<usize> for VPN {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<Self> for PPN {
    type Output = usize;
    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<Self> for VPN {
    type Output = usize;
    fn sub(self, rhs: Self) -> Self::Output {
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
    pub fn kaddr(self) -> VA {
        PA::from(self).kaddr()
    }
}
