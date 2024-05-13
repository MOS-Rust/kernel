#![allow(dead_code)]

use core::mem::size_of;
use core::panic;
use core::ptr;
use core::ptr::addr_of_mut;

use crate::error::MosError;
use crate::exception::trapframe::Trapframe;
use crate::mm::addr::{PA, VA};
use crate::mm::layout::PteFlags;
use crate::mm::layout::NASID;
use crate::mm::layout::PAGE_SIZE;
use crate::mm::layout::UENVS;
use crate::mm::layout::USTACKTOP;
use crate::mm::layout::UTOP;
use crate::mm::layout::UVPT;
use crate::mm::map::PageDirectory;
use crate::mm::map::PageTable;
use crate::mm::page::page_alloc;
use crate::mm::page::page_inc_ref;
use crate::mm::page::Page;
use crate::platform::cp0reg::{STATUS_EXL, STATUS_IE, STATUS_IM7, STATUS_UM};
use crate::round;
use alloc::vec::Vec;

use super::elf::elf_load_seg;
use super::elf::load_icode_mapper;
use super::elf::Elf32;
use super::elf::PT_LOAD;
use super::ipc::IpcInfo;

const NENV: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvStatus {
    Free,
    Runnable,
    NotRunnable,
}

#[derive(Clone, Copy, Debug)]
pub struct Env {
    pub pos: usize,

    pub tf: Trapframe,

    pub id: usize,
    pub asid: usize,
    pub parent_id: usize,
    pub status: EnvStatus,
    pub pgdir: PageTable,
    pub priority: u32,

    // IPC
    ipc_info: IpcInfo,

    user_tlb_mod_entry: usize,

    runs: u32,
}

impl Env {
    pub const fn new(pos: usize) -> Env {
        Env {
            pos,
            tf: Trapframe::new(),

            id: 0,
            asid: 0,
            parent_id: 0,
            status: EnvStatus::Free,
            pgdir: PageTable::empty(),
            priority: 0,

            ipc_info: IpcInfo::new(),

            user_tlb_mod_entry: 0,

            runs: 0,
        }
    }

    fn load_icode(&mut self, binary: &[u8]) {
        if Elf32::is_elf32_format(binary) {
            let elf = Elf32::from_bytes(&binary);
            let ehdr = elf.ehdr();
            for i in 0..ehdr.e_phnum as usize {
                let phdr = elf.phdr(i);
                if phdr.p_type == PT_LOAD as u32 {
                    if let Err(_error) = elf_load_seg(phdr, &binary[phdr.p_offset as usize..], load_icode_mapper, self) {
                        panic!();
                    }
                }
            }
            self.tf.cp0_epc = ehdr.e_entry;
        } else {
            panic!("bad elf at 0x{:p}", binary);
        }
    }
}

#[repr(C, align(4096))]
struct Envs {
    env_array: [Env; NENV],
}

static mut ENVS: Envs = Envs {
    env_array: [Env::new(0); NENV],
};

fn at_envs(index: usize) -> Env {
    if index >= NENV {
        panic!("index out of ENVS limit")
    }
    unsafe { ENVS.env_array[index] }
}

#[derive(Clone, Copy, Debug)]
pub struct EnvTracker {
    pos: usize,
}

impl EnvTracker {
    pub fn new(pos: usize) -> EnvTracker {
        EnvTracker { pos: pos }
    }
}

pub struct EnvManager {
    base_pgdir: PageDirectory,
    id_iter: usize,
    free_list: Vec<EnvTracker>,
    cur: EnvTracker,
    asid_bitmap: [usize; NASID / 32],
}

impl EnvManager {
    pub fn init() -> EnvManager {
        let mut free_list = Vec::with_capacity(NENV);
        let base_pgdir: PageDirectory;
        for i in (0..NENV).rev() {
            unsafe {
                ENVS.env_array[i] = Env::new(i);
            }
            free_list.push(EnvTracker::new(i));
        }

        if let Some(_page) = page_alloc(true) {
            base_pgdir = PageDirectory::init().0;
            unsafe {
                // TODO: pages not mapped, map if it is used
                map_segment(
                    base_pgdir,
                    0,
                    PA(addr_of_mut!(ENVS) as usize),
                    VA(UENVS),
                    round!(size_of::<Env>(), PAGE_SIZE),
                    PteFlags::G,
                )
            }

            EnvManager {
                base_pgdir,
                id_iter: 0,
                free_list: free_list,
                cur: EnvTracker::new(0),
                asid_bitmap: [0; NASID / 32],
            }
        } else {
            panic!("failed on page allocation");
        }
    }

    fn alloc_asid(&mut self) -> Result<usize, MosError> {
        for i in 0..NASID {
            let index = i >> 5;
            let inner = i & 31;
            if self.asid_bitmap[index] & (1 << inner) == 0 {
                self.asid_bitmap[index] |= 1 << inner;
                return Ok(i);
            }
        }

        Err(MosError::NoFreeEnv)
    }

    pub fn mkenvid(&mut self, env: Env) -> usize {
        self.id_iter += 1;
        ((self.id_iter - 1) << 11) | env.pos
    }

    pub fn envx(id: usize) -> usize {
        id & ((1 << 10) - 1)
    }

    pub fn at(&self, pos: usize) -> Env {
        at_envs(pos)
    }

    pub fn curenv(&self) -> Env {
        self.at(self.cur.pos)
    }

    pub fn get_free_env(&mut self) -> Result<Env, MosError> {
        if self.free_list.is_empty() {
            return Err(MosError::NoFreeEnv);
        }

        let tracker = self.free_list.pop().unwrap();
        Ok(at_envs(tracker.pos))
    }

    pub fn from_id(&self, id: usize, check_perm: bool) -> Result<Env, MosError> {
        if id == 0 {
            Ok(self.curenv())
        } else {
            let pos = Self::envx(id);
            let env = self.at(pos);

            if env.status == EnvStatus::Free || env.id != id {
                return Err(MosError::BadEnv);
            }

            if check_perm {
                if self.curenv().id != env.id && self.curenv().id != env.parent_id {
                    return Err(MosError::BadEnv);
                }
            }

            Ok(env)
        }
    }

    fn setup_vm(&mut self, mut env: Env) -> Result<(), MosError> {
        if let Some(page) = page_alloc(true) {
            page_inc_ref(page);
            env.pgdir = PageDirectory { page };
            unsafe {
                ptr::copy(
                    (env.pgdir.kaddr() + VA(UTOP).pdx()).as_mut_ptr::<u8>(),
                    (self.base_pgdir.kaddr() + VA(UTOP).pdx()).as_mut_ptr::<u8>(),
                    size_of::<usize>() * (VA(UVPT).pdx() - VA(UTOP).pdx()),
                );
            }

            // TODO: map page table of env itself to UVPT
            Ok(())
        } else {
            return Err(MosError::NoFreeEnv);
        }
    }

    pub fn alloc(&mut self, parent_id: usize) -> Result<Env, MosError> {
        let ret = self.get_free_env();
        if ret.is_ok() {
            let mut env = ret.unwrap();
            let r = self.setup_vm(env);
            if r.is_err() {
                return Err(r.unwrap_err());
            }

            env.user_tlb_mod_entry = 0;
            env.runs = 0;
            env.id = self.mkenvid(env);
            if let Ok(asid) = self.alloc_asid() {
                env.asid = asid;
            } else {
                return Err(MosError::NoFreeEnv);
            }
            env.parent_id = parent_id;

            env.tf.cp0_status = (STATUS_IM7 | STATUS_IE | STATUS_EXL | STATUS_UM) as u32;
            env.tf.regs[29] = (USTACKTOP - size_of::<i32>() - size_of::<usize>()) as u32;

            Ok(env)
        } else {
            return Err(MosError::NoFreeEnv);
        }
    }

    fn create(&mut self, binary: &[u8], priority: u32) -> Env {
        if let Ok(mut ret) = self.alloc(0) {
            ret.priority = priority;
            ret.status = EnvStatus::Runnable;

            ret.load_icode(binary);
            // TODO: add this env to env_sched_list

            return ret;
        } else {
            panic!("failed on env allocation");
        }
    }
}

fn map_segment(pgdir: PageDirectory, asid: usize, pa: PA, va: VA, size: usize, flags: PteFlags) {
    assert!(pa.0 % PAGE_SIZE == 0);
    assert!(va.0 % PAGE_SIZE == 0);
    assert!(size % PAGE_SIZE == 0);

    for i in 0..size {
        let _r = pgdir.insert(asid, Page::from(pa + i), va + i, flags);
    }
}
