#![allow(dead_code)]

use core::{cmp::min, mem::size_of, ptr::{copy_nonoverlapping, null}};

use alloc::slice;
use crate::mm::layout::PteFlags;

use crate::{error::MosError, mm::{addr::VA, layout::PAGE_SIZE, page::page_alloc}};

use super::{env::Env, tools::round_down};

pub const EI_INDENT: usize = 16;

pub type Elf32Half = u16;
pub type Elf32Word = u32;
pub type Elf32Sword = i32;
pub type Elf32Xword = u64;
pub type Elf32Sxword = i64;
pub type Elf32Addr = u32;
pub type Elf32Off = u32;
pub type Elf32Section = u16;
pub type Elf32Symndx = u32;

#[derive(Clone, Copy, Debug)]
pub struct Elf32Ehdr {
    pub(crate) e_indent: [u8; EI_INDENT],
    pub(crate) e_type: Elf32Half,
    pub(crate) e_machine: Elf32Half,
    pub(crate) e_version: Elf32Word,
    pub(crate) e_entry: Elf32Addr,
    pub(crate) e_phoff: Elf32Off,
    pub(crate) e_shoff: Elf32Off,
    pub(crate) e_flags: Elf32Word,
    pub(crate) e_ehsize: Elf32Half,
    pub(crate) e_phentsize: Elf32Half,
    pub(crate) e_phnum: Elf32Half,
    pub(crate) e_shentsize: Elf32Half,
    pub(crate) e_shnum: Elf32Half,
    pub(crate) e_shstrndx: Elf32Half,
}

pub const EI_MAG0: u8 = 0;
pub const ELFMAG0: u8 = 0x7f;
pub const EI_MAG1: u8 = 1;
pub const ELFMAG1: u8 = 'E' as u8;
pub const EI_MAG2: u8 = 2;
pub const ELFMAG2: u8 = 'L' as u8;
pub const EI_MAG3: u8 = 3;
pub const ELFMAG3: u8 = 'F' as u8;

#[derive(Clone, Copy, Debug)]
pub struct Elf32Phdr {
    pub(crate) p_type: Elf32Word,
    pub(crate) p_offset: Elf32Off,
    pub(crate) p_vaddr: Elf32Addr,
    pub(crate) p_paddr: Elf32Addr,
    pub(crate) p_filesz: Elf32Word,
    pub(crate) p_memsz: Elf32Word,
    pub(crate) p_flags: Elf32Word,
    pub(crate) p_align: Elf32Word,
}

pub const PT_NULL: usize = 0;
pub const PT_LOAD: usize = 1;
pub const PT_DYNAMIC: usize = 2;
pub const PT_INTERP: usize = 3;
pub const PT_NOTE: usize = 4;
pub const PT_SHLIB: usize = 5;
pub const PT_PHDR: usize = 6;
pub const PT_NUM: usize = 7;
pub const PT_LOOS: usize = 0x60000000;
pub const PT_HIOS: usize = 0x6fffffff;
pub const PT_LOPROC: usize = 0x70000000;
pub const PT_HIPROC: usize = 0x7fffffff;

pub const PF_X: u32 = 1 << 0;
pub const PF_W: u32 = 1 << 1;
pub const PF_R: u32 = 1 << 2;
pub const PF_MASKPROC: u32 = 0xf0000000;

pub type ElfMapperFn = fn(&mut Env, VA, usize, PteFlags, *const u8, usize) -> Result<(), MosError>;

pub fn elf_from(binary: *const u8, size: usize) -> Option<*const Elf32Ehdr> {
    let ehdr = binary as *const Elf32Ehdr;
    if size >= size_of::<Elf32Ehdr>() {
        unsafe {
            let ehdr_slice = slice::from_raw_parts(ehdr as *const u8, size_of::<Elf32Ehdr>());
            if ehdr_slice[0] == ELFMAG0 && ehdr_slice[1] == ELFMAG1 && ehdr_slice[2] == ELFMAG2 && ehdr_slice[3] == ELFMAG3 {
                let ehdr_struct = &*ehdr;
                if ehdr_struct.e_type == 2 {
                    return Some(ehdr);
                }
            }
        }
    }
    None
}

pub fn elf_load_seg(ph: &Elf32Phdr, bin: *const u8, map_page: ElfMapperFn, env: &mut Env) -> Result<(), MosError> {
    let va: VA = VA(ph.p_vaddr as usize);
    let bin_size: usize = ph.p_filesz as usize;
    let sg_size: usize = ph.p_memsz as usize;
    let mut perm = PteFlags::V;
    if ph.p_flags & PF_W != 0 {
        perm |= PteFlags::D;
    }

    let offset = va.0 - round_down(va.0, PAGE_SIZE);
    if offset != 0 {
        if let Err(error) = map_page(env, va, offset, perm, bin, min(bin_size, PAGE_SIZE - offset)) {
            return Err(error)
        }
    }

    let mut i: usize = 0;
    if offset != 0 {
        i = min(bin_size, PAGE_SIZE - offset);
    }

    while i < bin_size {
        if let Err(error) = map_page(env, va + i, 0, perm , null(), min(sg_size - i, PAGE_SIZE)) {
            return Err(error)
        }
        i += PAGE_SIZE;
    }

    Ok(())
}

pub fn load_icode_mapper(env: &mut Env, va: VA, offset: usize, perm: PteFlags, src: *const u8, len: usize) -> Result<(), MosError> {
    let p = page_alloc(true).unwrap();

    if !src.is_null() {
        unsafe {copy_nonoverlapping(src, (p.ppn().kaddr() + offset).0 as *mut u8, len)}
    }

    env.pgdir.insert(env.asid, p, va, perm)
}