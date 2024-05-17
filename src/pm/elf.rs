#![allow(dead_code)]

use core::{cmp::min, ptr::copy_nonoverlapping};

use crate::{mm::layout::PteFlags, round_down};

use crate::{
    error::MosError,
    mm::{addr::VA, layout::PAGE_SIZE, page::page_alloc},
};

use super::env::Env;

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

pub struct Elf32<'a> {
    binary: &'a [u8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf32Ehdr {
    pub e_indent: [u8; EI_INDENT],
    pub e_type: Elf32Half,
    pub e_machine: Elf32Half,
    pub e_version: Elf32Word,
    pub e_entry: Elf32Addr,
    pub e_phoff: Elf32Off,
    pub e_shoff: Elf32Off,
    pub e_flags: Elf32Word,
    pub e_ehsize: Elf32Half,
    pub e_phentsize: Elf32Half,
    pub e_phnum: Elf32Half,
    pub e_shentsize: Elf32Half,
    pub e_shnum: Elf32Half,
    pub e_shstrndx: Elf32Half,
}

pub const ELFMAG0: u8 = 0x7f;
pub const ELFMAG1: u8 = b'E';
pub const ELFMAG2: u8 = b'L';
pub const ELFMAG3: u8 = b'F';

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf32Phdr {
    pub p_type: Elf32Word,
    pub p_offset: Elf32Off,
    pub p_vaddr: Elf32Addr,
    pub p_paddr: Elf32Addr,
    pub p_filesz: Elf32Word,
    pub p_memsz: Elf32Word,
    pub p_flags: Elf32Word,
    pub p_align: Elf32Word,
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

pub type ElfMapperFn = fn(&mut Env, VA, usize, PteFlags, Option<&[u8]>) -> Result<(), MosError>;

impl<'a> Elf32<'a> {
    pub fn is_elf32_format(data: &[u8]) -> bool {
        data.len() >= 5
            && data[0] == 0x7f
            && data[1] == b'E'
            && data[2] == b'L'
            && data[3] == b'F'
            && data[4] == 1
    }

    pub fn from_bytes(data: &'a [u8]) -> Self {
        Self { binary: data }
    }

    pub fn ehdr(&self) -> &Elf32Ehdr {
        unsafe { &*(self.binary.as_ptr() as *const Elf32Ehdr) }
    }

    pub fn phdr(&self, n: usize) -> &Elf32Phdr {
        let ehdr = self.ehdr();
        let ph_offset = ehdr.e_phoff as usize;
        let ph_size = ehdr.e_phentsize as usize;
        let ph_num = ehdr.e_phnum as usize;
        let ph_start = ph_offset + n * ph_size;
        if n < ph_num {
            unsafe { &*(self.binary.as_ptr().add(ph_start) as *const Elf32Phdr) }
        } else {
            panic!("phdr index out of range");
        }
    }
}

pub fn elf_load_seg(
    ph: &Elf32Phdr,
    bin: &[u8],
    map_page: ElfMapperFn,
    env: &mut Env,
) -> Result<(), MosError> {
    let va: VA = VA(ph.p_vaddr as usize);
    let bin_size: usize = ph.p_filesz as usize;
    let sg_size: usize = ph.p_memsz as usize;
    let mut perm = PteFlags::V;
    if ph.p_flags & PF_W != 0 {
        perm |= PteFlags::D;
    }

    let offset = va.0 - round_down!(va.0, PAGE_SIZE);
    if offset != 0 {
        let len = min(bin_size, PAGE_SIZE - offset);
        map_page(env, va, offset, perm, Some(&bin[..len]))?
    }

    let mut i: usize = 0;
    if offset != 0 {
        i = min(bin_size, PAGE_SIZE - offset);
    }

    while i < bin_size {
        let len = min(sg_size - i, PAGE_SIZE);
        map_page(env, va + i, 0, perm, Some(&bin[i..i + len]))?;
        i += PAGE_SIZE;
    }

    while i < sg_size {
        map_page(env, va + i, 0, perm, None)?;
        i += PAGE_SIZE;
    }

    Ok(())
}

pub fn load_icode_mapper(
    env: &mut Env,
    va: VA,
    offset: usize,
    perm: PteFlags,
    src: Option<&[u8]>,
) -> Result<(), MosError> {
    let p = page_alloc(true).unwrap();

    if let Some(data) = src {
        unsafe {
            copy_nonoverlapping(
                data.as_ptr(),
                (p.ppn().kaddr() + offset).0 as *mut u8,
                data.len(),
            )
        }
    }
    // debug!("KVA: {:x}, VA: {:x}, perm: {:?}, offset: {:?}", p.ppn().kaddr().0, va.0, perm, offset);
    env.pgdir.insert(env.asid, p, va, perm)
}
