//! Contains exactly the same thing in MOS `include/mmu.h`
#![allow(dead_code)] // TODO: Remove this

pub const NASID: usize = 256;
pub const PAGE_SIZE: usize = 4096;
pub const PTMAP: usize = PAGE_SIZE;
pub const PDMAP: usize = 0x0040_0000; // Bytes mapped by a page directory entry, 4 MiB
pub const PGSHIFT: usize = 12;
pub const PDSHIFT: usize = 22;

const PTE_COW: usize = 0x0001;
const PTE_LIBRARY: usize = 0x0002;

bitflags! {
    #[derive(Debug)]
    pub struct PteFlags: usize {
        /// the 6 bits below are those stored in cp0.entry_lo
        const G = 1 << 0;
        const V = 1 << 1;
        const D = 1 << 2;
        
        // Only used internally
        const C0 = 1 << 3;
        const C1 = 1 << 4;
        const C2 = 1 << 5;
        
        const Cached = PteFlags::C2.bits() | PteFlags::C1.bits();
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

