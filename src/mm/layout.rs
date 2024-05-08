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
//! # PteFlags
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
#![allow(dead_code)] // TODO: Remove this

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

// TODO: Probably this should move to mips/cp0/entrylo.rs
bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PteFlags: usize {
        /// the 6 bits below are those stored in cp0.entry_lo
        const G = 1 << 0;
        const V = 1 << 1;
        const D = 1 << 2;

        // Only used internally
        const C0 = 1 << 3;
        const C1 = 1 << 4;
        const C2 = 1 << 5;
        
        const Cacheable = PteFlags::C2.bits() | PteFlags::C1.bits();
        const Uncached = PteFlags::C2.bits() & !PteFlags::C1.bits();

        /// the bits below are controlled by software
        const COW = 1 << 6;
        const SHARED = 1 << 7;
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

pub const KUSEG: usize = 0x0000_0000;
pub const KSEG0: usize = 0x8000_0000;
pub const KSEG1: usize = 0xa000_0000;
pub const KSEG2: usize = 0xc000_0000;

pub const KERNBASE: usize = 0x8002_0000;
pub const ULIM: usize = 0x8000_0000;

pub const UVPT: usize = ULIM - PDMAP;
pub const UPAGES: usize = UVPT - PDMAP;
pub const UENVS: usize = UPAGES - PDMAP;

pub const UTOP: usize = UENVS;
pub const UXSTACKTOP: usize = UTOP;

pub const USTACKTOP: usize = UTOP - 2 * PTMAP;
pub const UTEXT: usize = PDMAP;
pub const UCOW: usize = UTEXT - PTMAP;
pub const UTEMP: usize = UCOW - PTMAP;
