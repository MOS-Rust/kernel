#![allow(dead_code)]

use core::cell::RefCell;
use core::mem::size_of;
use core::panic;
use core::ptr;
use core::ptr::addr_of_mut;

use crate::error::MosError;
use crate::exception::trapframe::Trapframe;
use crate::mm::addr::{PA, VA};
use crate::mm::layout::{PteFlags, NASID, PAGE_SIZE, UENVS, USTACKTOP, UTOP, UVPT};
use crate::mm::map::PageDirectory;
use crate::mm::map::PageTable;
use crate::mm::page::{page_alloc, page_inc_ref, Page};
use crate::platform::cp0reg::{STATUS_EXL, STATUS_IE, STATUS_IM7, STATUS_UM};
use crate::pm::schedule::schedule;
use crate::round;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use log::info;

use super::elf::elf_load_seg;
use super::elf::load_icode_mapper;
use super::elf::Elf32;
use super::elf::PT_LOAD;

use super::ipc::IpcInfo;

const NENV: usize = 1024;

const NEW_ENV: Env = Env::new(0);
static mut ENVS: Envs = Envs {
    env_array: [NEW_ENV; NENV],
};
static mut ENV_COUNT: usize = 0;
static mut ASID_BITMAP: [usize; NASID / 32] = [0; NASID / 32];

#[derive(Debug, PartialEq, Eq)]
pub enum EnvStatus {
    Free,
    Runnable,
    NotRunnable,
}

#[derive(Debug)]
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

    fn tracker(&self) -> EnvTracker {
        EnvTracker::new(self.pos)
    }

    fn load_icode(&mut self, binary: &[u8]) {
        if Elf32::is_elf32_format(binary) {
            let elf = Elf32::from_bytes(&binary);
            let ehdr = elf.ehdr();
            for i in 0..ehdr.e_phnum as usize {
                let phdr = elf.phdr(i);
                if phdr.p_type == PT_LOAD as u32 {
                    if let Err(_error) = elf_load_seg(
                        phdr,
                        &binary[phdr.p_offset as usize..],
                        load_icode_mapper,
                        self,
                    ) {
                        panic!();
                    }
                }
            }
            self.tf.cp0_epc = ehdr.e_entry;
        } else {
            panic!("bad elf at 0x{:p}", binary);
        }
    }

    pub fn runnable(&self) -> bool {
        self.status == EnvStatus::Runnable
    }
}

#[repr(C, align(4096))]
struct Envs {
    env_array: [Env; NENV],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnvTracker {
    pos: usize,
}

impl EnvTracker {
    pub const fn new(pos: usize) -> EnvTracker {
        EnvTracker { pos }
    }
}

pub struct EnvManager {
    base_pgdir: PageDirectory,
    free_list: RefCell<Vec<EnvTracker>>,
    schedule_list: RefCell<VecDeque<EnvTracker>>,
    cur: Option<EnvTracker>,
}

impl EnvManager {
    pub const fn new() -> EnvManager {
        EnvManager {
            base_pgdir: PageDirectory::empty(),
            free_list: RefCell::new(Vec::new()),
            schedule_list: RefCell::new(VecDeque::new()),
            cur: None,
        }
    }

    pub fn init(&mut self) {
        let mut free_list = Vec::with_capacity(NENV);
        for i in (0..NENV).rev() {
            unsafe {
                ENVS.env_array[i] = Env::new(i);
            }
            free_list.push(EnvTracker::new(i));
        }
        let base_pgdir = PageDirectory::init().0;
        // TODO: Map mm::pages to UPAGES
        unsafe {
            map_segment(
                base_pgdir,
                0,
                VA(addr_of_mut!(ENVS) as usize).paddr(),
                VA(UENVS),
                round!(size_of::<Env>(), PAGE_SIZE),
                PteFlags::G,
            )
        }
        self.base_pgdir = base_pgdir;
        self.free_list = RefCell::new(free_list);
        self.schedule_list = RefCell::new(VecDeque::with_capacity(NENV));
    }

    pub fn envx(id: usize) -> usize {
        id & ((1 << 10) - 1)
    }

    pub fn at(&self, pos: usize) -> &mut Env {
        env_at(pos)
    }

    pub fn curenv(&self) -> Option<&mut Env> {
        if let Some(tracker) = self.cur {
            Some(env_at(tracker.pos))
        } else {
            None
        }
    }

    pub fn get_free_env(&self) -> Result<&mut Env, MosError> {
        if self.free_list.borrow().is_empty() {
            return Err(MosError::NoFreeEnv);
        }

        let tracker = self.free_list.borrow_mut().pop().unwrap();
        Ok(env_at(tracker.pos))
    }

    pub fn from_id(&self, id: usize, check_perm: bool) -> Result<&mut Env, MosError> {
        if id == 0 {
            Ok(self.curenv().unwrap())
        } else {
            let pos = Self::envx(id);
            let env = self.at(pos);

            if env.status == EnvStatus::Free || env.id != id {
                return Err(MosError::BadEnv);
            }

            if check_perm {
                if self.curenv().unwrap().id != env.id && self.curenv().unwrap().id != env.parent_id
                {
                    return Err(MosError::BadEnv);
                }
            }

            Ok(env)
        }
    }

    fn setup_vm(&self, env: &mut Env) -> Result<(), MosError> {
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

    pub fn alloc(&mut self, parent_id: usize) -> Result<&mut Env, MosError> {
        if let Ok(env) = self.get_free_env() {
            if let Err(error) = self.setup_vm(env) {
                return Err(error);
            }
            env.user_tlb_mod_entry = 0;
            env.runs = 0;
            env.id = mkenvid(env);
            if let Ok(asid) = alloc_asid() {
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

    fn create(&self, binary: &[u8], priority: u32) -> &mut Env {
        let env = self.get_free_env().expect("failed on env allocation");
        env.priority = priority;
        env.status = EnvStatus::Runnable;
        env.load_icode(binary);
        self.schedule_list
            .borrow_mut()
            .push_back(EnvTracker::new(env.pos));
        env
    }

    fn env_free(&mut self, _env: &mut Env) {
        unimplemented!()
    }

    pub fn env_destroy(&mut self, env: &mut Env) {
        self.env_free(env);
        if self.cur.is_some() && self.cur.unwrap().pos == env.pos {
            self.cur = None;
            info!("{}: I am killed ...", env.id);
            schedule(true);
        }
    }

    pub fn env_run(&mut self, _env: &mut Env) -> ! {
        loop {}
    }
}

fn map_segment(pgdir: PageDirectory, asid: usize, pa: PA, va: VA, size: usize, flags: PteFlags) {
    assert!(pa.0 % PAGE_SIZE == 0);
    assert!(va.0 % PAGE_SIZE == 0);
    assert!(size % PAGE_SIZE == 0);

    for i in (0..size).step_by(PAGE_SIZE) {
        pgdir.insert(asid, Page::from(pa + i), va + i, flags).expect("failed on mapping");
    }
}

fn mkenvid(env: &Env) -> usize {
    unsafe { ENV_COUNT += 1 };
    ((unsafe { ENV_COUNT } - 1) << 11) | env.pos
}

fn alloc_asid() -> Result<usize, MosError> {
    for i in 0..NASID {
        let index = i >> 5;
        let inner = i & 31;
        unsafe {
            if ASID_BITMAP[index] & (1 << inner) == 0 {
                ASID_BITMAP[index] |= 1 << inner;
                return Ok(i);
            }
        }
    }
    Err(MosError::NoFreeEnv)
}

fn env_at(index: usize) -> &'static mut Env {
    if index >= NENV {
        panic!("index out of ENVS limit")
    }
    unsafe { &mut ENVS.env_array[index] }
}