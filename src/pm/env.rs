#![allow(dead_code)]

use super::{
    elf::{elf_load_seg, load_icode_mapper, Elf32, PT_LOAD},
    ipc::IpcInfo,
    schedule::schedule,
};
use crate::{
    error::MosError,
    exception::{
        clock::reset_kclock,
        trapframe::{Trapframe, TF_SIZE},
    },
    mm::{
        addr::{PA, VA},
        layout::{
            PteFlags, KSTACKTOP, NASID, PAGE_SIZE, PDSHIFT, PGSHIFT, UENVS, USTACKTOP, UTOP, UVPT,
        },
        map::{PageDirectory, PageTable, Pte},
        page::{page_dec_ref, Page},
        tlb::tlb_invalidate,
    },
    platform::cp0reg::{STATUS_EXL, STATUS_IE, STATUS_IM7, STATUS_UM},
    round,
};
use alloc::{collections::VecDeque, vec::Vec};
use core::{
    arch::asm,
    cell::RefCell,
    mem::size_of,
    ptr::{self, addr_of_mut},
};
use log::{info, warn};

pub const NENV: usize = 1024;

const NEW_ENV: Env = Env::new(0);
pub(crate) static mut ENVS: Envs = Envs {
    env_array: [NEW_ENV; NENV],
};
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
pub struct Envs {
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
    cur_pgdir: PageDirectory,
}

impl EnvManager {
    pub const fn new() -> EnvManager {
        EnvManager {
            base_pgdir: PageDirectory::empty(),
            free_list: RefCell::new(Vec::new()),
            schedule_list: RefCell::new(VecDeque::new()),
            cur: None,
            cur_pgdir: PageDirectory::empty(),
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
        let base_pgdir;
        if let Ok((pgdir, _)) = PageDirectory::init() {
            base_pgdir = pgdir;
        } else {
            panic!("failed to init base_pgdir");
        }
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
            let env = env_at(pos);

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
        match PageDirectory::init() {
            Ok((pgdir, page)) => {
                env.pgdir = pgdir;
                unsafe {
                    ptr::copy_nonoverlapping(
                        (self.base_pgdir.pte_at(VA(UTOP).pdx()) as *const Pte).cast::<u8>(),
                        (env.pgdir.pte_at(VA(UTOP).pdx()) as *mut Pte).cast::<u8>(),
                        size_of::<usize>() * (VA(UVPT).pdx() - VA(UTOP).pdx()),
                    );
                }
                *self.base_pgdir.pte_at(VA(UVPT).pdx()) = Pte::new(page.ppn(), PteFlags::V);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn alloc(&self, parent_id: usize) -> Result<&mut Env, MosError> {
        if let Ok(env) = self.get_free_env() {
            if let Err(error) = self.setup_vm(env) {
                return Err(error);
            }
            env.user_tlb_mod_entry = 0;
            env.runs = 0;
            env.id = mkenvid(env);
            if let Ok(asid) = asid_alloc() {
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

    pub fn create(&self, binary: &[u8], priority: u32) -> &mut Env {
        let env = self.alloc(0).expect("failed to alloc env");
        env.priority = priority;
        env.status = EnvStatus::Runnable;
        env.load_icode(binary);
        self.schedule_list
            .borrow_mut()
            .push_back(EnvTracker::new(env.pos));
        env
    }

    pub fn env_free(&mut self, env: &mut Env) {
        if let Some(curenv) = self.curenv() {
            if curenv.id != env.id && curenv.id != env.parent_id {
                warn!("{:x} try to free {:x} which is not its child", curenv.id, env.id);
            }
            info!("{:x} free env {:x}", curenv.id, env.id);
        } else {
            info!("0 free env {:x}", env.id);
        }
        for i in 0..VA(UTOP).pdx() {
            if !env.pgdir.pte_at(i).is_valid() {
                continue;
            }
            let pa: PA = env.pgdir.pte_at(i).ppn().into();
            let pt = pa.kaddr().as_mut_ptr::<Pte>();
            for j in 0..PAGE_SIZE / size_of::<Pte>() {
                let pte = unsafe { &mut *pt.add(j) };
                if pte.is_valid() {
                    env.pgdir
                        .remove(env.asid, VA((i << PDSHIFT) + (j << PGSHIFT)));
                }
            }
            unsafe { *pt = Pte::empty() };
            page_dec_ref(pa.into());
            tlb_invalidate(env.asid, VA(UVPT + i << PGSHIFT));
        }
        page_dec_ref(env.pgdir.page);
        asid_free(env.asid);
        tlb_invalidate(env.asid, VA(UVPT + VA(UVPT).pdx() << PGSHIFT));
        env.status = EnvStatus::Free;
        self.free_list.borrow_mut().push(EnvTracker::new(env.pos));
        self.schedule_list
            .borrow_mut()
            .retain(|&x| x != env.tracker());
    }

    pub fn env_destroy(&mut self, env: &mut Env) {
        self.env_free(env);
        if self.cur.is_some() && self.cur.unwrap().pos == env.pos {
            self.cur = None;
            info!("{:x}: I am killed ...", env.id);
            schedule(true);
        }
    }

    pub fn env_run(&mut self, env: &mut Env) -> ! {
        assert!(env.status == EnvStatus::Runnable);
        if let Some(cur) = self.curenv() {
            cur.tf = unsafe { *Trapframe::from_memory(VA(KSTACKTOP - TF_SIZE)) };
        }
        self.cur = Some(env.tracker());
        env.runs += 1;

        self.cur_pgdir = env.pgdir;

        unsafe { env_pop_trapframe(&mut env.tf, env.asid as u32) }
    }

    pub fn get_first(&self) -> Option<&mut Env> {
        if let Some(tracker) = self.schedule_list.borrow().front() {
            Some(env_at(tracker.pos))
        } else {
            None
        }
    }
    pub fn move_to_end(&self, env: &Env) {
        let tracker = env.tracker();
        // tracker should be the first element (?)
        assert!(self.schedule_list.borrow_mut().pop_front() == Some(tracker));
        self.schedule_list.borrow_mut().push_back(tracker);
    }

    pub fn current_pgdir(&mut self) -> &mut PageDirectory {
        &mut self.cur_pgdir
    }

    pub fn base_pgdir(&mut self) -> &mut PageDirectory {
        &mut self.base_pgdir
    }
}

fn map_segment(pgdir: PageDirectory, asid: usize, pa: PA, va: VA, size: usize, flags: PteFlags) {
    assert!(pa.0 % PAGE_SIZE == 0);
    assert!(va.0 % PAGE_SIZE == 0);
    assert!(size % PAGE_SIZE == 0);

    for i in (0..size).step_by(PAGE_SIZE) {
        pgdir
            .insert(asid, Page::from(pa + i), va + i, flags | PteFlags::V)
            .expect("failed on mapping");
    }
}

fn mkenvid(env: &Env) -> usize {
    static mut ENV_COUNT: usize = 0;
    unsafe { ENV_COUNT += 1 };
    ((unsafe { ENV_COUNT }) << 11) | env.pos
}

fn asid_alloc() -> Result<usize, MosError> {
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

fn asid_free(asid: usize) {
    let index = asid >> 5;
    let inner = asid & 31;
    unsafe {
        ASID_BITMAP[index] &= !(1 << inner);
    }
}

fn env_at(index: usize) -> &'static mut Env {
    if index >= NENV {
        panic!("index out of ENVS limit")
    }
    unsafe { &mut ENVS.env_array[index] }
}

unsafe fn env_pop_trapframe(tf: *mut Trapframe, asid: u32) -> ! {
    extern "C" {
        fn _ret_from_exception() -> !;
    }
    asm!(
        ".set noat",
        "mtc0 {}, $10",
        ".set at",
        in(reg) asid,
    );
    reset_kclock();
    asm!("ori $sp, {}, 0",
        in(reg) tf,
        options(nostack, nomem)
    );
    _ret_from_exception();
}
