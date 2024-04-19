//! Contains exactly the same thing in MOS `include/mmu.h`
#![allow(dead_code)] // TODO: Remove this

pub const NASID: usize = 256;
pub const PAGE_SIZE: usize = 4096;
pub const PTMAP: usize = PAGE_SIZE;
pub const PDMAP: usize = 0x0040_0000; // Bytes mapped by a page directory entry, 4 MiB
pub const PGSHIFT: usize = 12;
pub const PDSHIFT: usize = 22;

pub const PTE_HARDFLAG_SHIFT: usize = 6;

pub const PTE_G: usize = 0x0001 << PTE_HARDFLAG_SHIFT;
pub const PTE_V: usize = 0x0002 << PTE_HARDFLAG_SHIFT;
pub const PTE_D: usize = 0x0004 << PTE_HARDFLAG_SHIFT;

pub const PTE_C_CACHEABLE: usize = 0x0018 << PTE_HARDFLAG_SHIFT;
pub const PTE_C_UNCACHEABLE: usize = 0x0010 << PTE_HARDFLAG_SHIFT;

pub const PTE_COW: usize = 0x0001;
pub const PTE_LIBRARY: usize = 0x0002;

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

