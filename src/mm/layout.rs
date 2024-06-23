//! Contains the layout constants for memory management in the MOS kernel.
//!
//! This module defines various constants related to memory layout, such as page sizes, address
//! ranges, and flags for page table entries. It also includes a diagram illustrating the memory
//! layout for different sections of the kernel and user space.
//!
//! # Constants
//!
//! - `NASID`: Maximum number of Address Space Identifiers (ASIDs).
//! - `PAGE_SIZE`: Bytes per page.
//! - `PTMAP`: Bytes mapped by a page table entry (4 KiB).
//! - `PDMAP`: Bytes mapped by a page directory entry (4 MiB).
//! - `PGSHIFT`: Page shift value (12).
//! - `PDSHIFT`: Page directory shift value (22).
//!
//! # `PteFlags`
//!
//! The `PteFlags` bitflags struct represents the flags for a page table entry.
//!
//! # Memory Layout Diagram
//!
//! The memory layout diagram shows the different sections of memory in the kernel and user space.
//! It includes the following sections:
//!
//! - `KSEG2`: Kernel segment 2.
//! - `KSEG1`: Kernel segment 1.
//! - `KSEG0`: Kernel segment 0.
//! - `KSTACKTOP`: Kernel stack top.
//! - `KERNBASE`: Kernel base address.
//! - `ULIM`: User limit.
//! - `UVPT`: User virtual page table.
//! - `UPAGES`: User pages.
//! - `UENVS`: User environments.
//! - `UTOP`: User top.
//! - `UXSTACKTOP`: User exception stack top.
//! - `USTACKTOP`: User stack top.
//! - `UTEXT`: User text.
//! - `UCOW`: User copy-on-write.
//! - `UTEMP`: User temporary.
//!
//! The diagram also includes the address ranges for each section.
//!
//! For more information, refer to the MOS `include/mmu.h` file.

use crate::{
    const_export_usize,
    platform::malta::{IDE_BASE, SERIAL_BASE},
};

/// Maximum number of Address Space Identifiers(ASIDs)
pub const NASID: usize = 256;
/// Bytes per page
pub const PAGE_SIZE: usize = 4096;
/// Bytes mapped by a page table entry, 4 KiB
pub const PTMAP: usize = PAGE_SIZE;
/// Bytes mapped by a page directory entry, 4 MiB
pub const PDMAP: usize = 0x0040_0000;
/// Page shift value
pub const PGSHIFT: usize = 12;
/// Page directory shift value
pub const PDSHIFT: usize = 22;
/// PTE flag shift
pub const PTE_HARDFLAG_SHIFT: usize = 6;

bitflags! {
    /// Pte flag definations
    #[derive(Clone, Copy, Debug)]
    pub struct PteFlags: usize {
        // The 6 bits below are those stored in cp0.entry_lo
        /// Global bit
        const G = 1 << 0 << PTE_HARDFLAG_SHIFT;
        /// Valid bit
        const V = 1 << 1 << PTE_HARDFLAG_SHIFT;
        /// Dirty bit
        const D = 1 << 2 << PTE_HARDFLAG_SHIFT;

        // Only used internally

        /// C0
        const C0 = 1 << 3 << PTE_HARDFLAG_SHIFT;
        /// C1
        const C1 = 1 << 4 << PTE_HARDFLAG_SHIFT;
        /// C2
        const C2 = 1 << 5 << PTE_HARDFLAG_SHIFT;

        /// Cacheable, noncoherent, write-back, write allocate
        const Cacheable = PteFlags::C0.bits() | PteFlags::C1.bits();
        /// Uncached
        const Uncached = PteFlags::C1.bits() & !PteFlags::C0.bits();

        // The bits below are controlled by software
        /// Copy-on-write bit
        const COW = 0x1;
        /// Read-only bit
        const SHARED = 0x2;
    }
}
/*
 o     4G ----------->  +----------------------------+------------0x100000000
 o                      |       ...                  |  kseg2
 o      KSEG2    -----> +----------------------------+------------0xc000 0000
 o                      |          Devices           |  kseg1
 o      KSEG1    -----> +----------------------------+------------0xa000 0000
 o                      |      Invalid Memory        |   /|\
 o                      +----------------------------+----|-------Physical Memory Max
 o                      |       ...                  |  kseg0
 o      KSTACKTOP-----> +----------------------------+----|-------0x8040 0000-------end
 o                      |       Kernel Stack         |    | KSTKSIZE            /|\
 o                      +----------------------------+----|------                |
 o                      |       Kernel Text          |    |                    PDMAP
 o      KERNBASE -----> +----------------------------+----|-------0x8002 0000    |
 o                      |      Exception Entry       |   \|/                    \|/
 o      ULIM     -----> +----------------------------+------------0x8000 0000-------
 o                      |         User VPT           |     PDMAP                /|\
 o      UVPT     -----> +----------------------------+------------0x7fc0 0000    |
 o                      |           pages            |     PDMAP                 |
 o      UPAGES   -----> +----------------------------+------------0x7f80 0000    |
 o                      |           envs             |     PDMAP                 |
 o  UTOP,UENVS   -----> +----------------------------+------------0x7f40 0000    |
 o  UXSTACKTOP -/       |     user exception stack   |     PTMAP                 |
 o                      +----------------------------+------------0x7f3f f000    |
 o                      |                            |     PTMAP                 |
 o      USTACKTOP ----> +----------------------------+------------0x7f3f e000    |
 o                      |     normal user stack      |     PTMAP                 |
 o                      +----------------------------+------------0x7f3f d000    |
 a                      |                            |                           |
 a                      ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~                           |
 a                      .                            .                           |
 a                      .                            .                         kuseg
 a                      .                            .                           |
 a                      |~~~~~~~~~~~~~~~~~~~~~~~~~~~~|                           |
 a                      |                            |                           |
 o       UTEXT   -----> +----------------------------+------------0x0040 0000    |
 o                      |      reserved for COW      |     PTMAP                 |
 o       UCOW    -----> +----------------------------+------------0x003f f000    |
 o                      |   reversed for temporary   |     PTMAP                 |
 o       UTEMP   -----> +----------------------------+------------0x003f e000    |
 o                      |       invalid memory       |                          \|/
 a     0 ------------>  +----------------------------+ ----------------------------
 o
*/

// pub const KUSEG: usize = 0x0000_0000;
/// KSEG0 address
pub const KSEG0: usize = 0x8000_0000;
/// KSEG1 address
pub const KSEG1: usize = 0xa000_0000;
// pub const KSEG2: usize = 0xc000_0000;

const_export_usize!(KSTACKTOP, 0x80400000);
// pub const KERNBASE: usize = 0x8002_0000;
/// ULIM address
pub const ULIM: usize = 0x8000_0000;

/// UVPT address
pub const UVPT: usize = ULIM - PDMAP;
/// UPAGES address
pub const UPAGES: usize = UVPT - PDMAP;
/// UENVS address
pub const UENVS: usize = UPAGES - PDMAP;

/// UTOP address
pub const UTOP: usize = UENVS;
/// UXSTACKTOP address
pub const UXSTACKTOP: usize = UTOP;

/// USTACKTOP address
pub const USTACKTOP: usize = UTOP - 2 * PTMAP;
/// UTEXT address
pub const UTEXT: usize = PDMAP;
/// UCOW address
pub const UCOW: usize = UTEXT - PTMAP;
/// UTEMP address
pub const UTEMP: usize = UCOW - PTMAP;

/// Check if provided va is illegal in user space
#[inline]
pub fn is_illegal_user_va(va: usize) -> bool {
    !(UTEMP..UTOP).contains(&va)
}

/// Check if provided va range([va, va + len]) is illegal in user space
#[inline]
pub fn is_illegal_user_va_range(va: usize, len: usize) -> bool {
    if len == 0 {
        return false;
    }
    va < UTEMP || va.checked_add(len).is_none() || va.checked_add(len).unwrap() > UTOP
}

/// Check if provided va range([va, va + len]) is illegal in kernal space
#[inline]
pub const fn is_dev_va_range(va: usize, len: usize) -> bool {
    const CONOLE_ADDR_LEN: usize = 0x20;
    const IDE_ADDR_LEN: usize = 0x8;
    (va >= SERIAL_BASE && va + len <= SERIAL_BASE + CONOLE_ADDR_LEN)
        || (va >= IDE_BASE && va + len <= IDE_BASE + IDE_ADDR_LEN)
}
