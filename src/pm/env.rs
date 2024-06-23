//! Implementation of process manager

use super::{
    elf::{elf_load_seg, load_icode_mapper, Elf32, PT_LOAD},
    ipc::IpcInfo,
    schedule::schedule,
};
use crate::{
    error::MosError,
    exception::{reset_kclock, Trapframe, TF_SIZE},
    mm::{
        layout::{
            PteFlags, KSTACKTOP, NASID, PAGE_SIZE, PDSHIFT, PGSHIFT, UENVS, UPAGES, USTACKTOP,
            UTOP, UVPT,
        },
        map::{PageDirectory, Pte},
        page::{page_dec_ref, Page, PAGE_ALLOCATOR},
        tlb_invalidate, PA, PPN, VA,
    },
    mutex::Mutex,
    platform::cp0reg::{STATUS_EXL, STATUS_IE, STATUS_IM7, STATUS_UM},
    pm::ENV_MANAGER,
    round,
    syscall::pool_remove_user_on_exit,
};
use alloc::{collections::VecDeque, vec::Vec};
use core::{
    arch::asm,
    cell::RefCell,
    mem::size_of,
    ptr::{self, addr_of_mut},
};
use log::{info, warn};

const NENV: usize = 1024;
const NEW_ENV: Env = Env::new();
static mut ENVS: Envs = Envs {
    env_array: [NEW_ENV; NENV],
};
static mut ASID_BITMAP: [usize; NASID / 32] = [0; NASID / 32];

/// Implementation of env->env_status of original mos
#[repr(u32)]
#[derive(PartialEq, Eq, Debug)]
pub enum EnvStatus {
    /// Indicating env is free
    Free = 0,
    /// Indicating env is runnable
    Runnable = 1,
    /// Indicating env is not runnable
    NotRunnable = 2,
}

/// Struct Env as process controller block,
/// Same as struct Env in mos
#[repr(C)]
#[derive(Debug)]
pub struct Env {
    pub tf: Trapframe,

    // the two placeholders exist to keep compatibility with the original mos, 
    // which uses linked list to manage envs
    __placeholder_1: [usize; 2], // env_link

    pub id: usize,
    pub asid: usize,
    pub parent_id: usize,
    pub status: EnvStatus,
    pub pgdir: VA,

    __placeholder_2: [usize; 2], // env_sched_link

    pub priority: u32,

    // IPC
    pub ipc_info: IpcInfo,

    pub user_tlb_mod_entry: usize,

    pub runs: u32,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    /// Create a new empty Env block
    pub const fn new() -> Self {
        Self {
            tf: Trapframe::new(),

            __placeholder_1: [0; 2],

            id: 0,
            asid: 0,
            parent_id: 0,
            status: EnvStatus::Free,
            pgdir: VA(0),

            __placeholder_2: [0; 2],

            priority: 0,

            ipc_info: IpcInfo::new(),

            user_tlb_mod_entry: 0,

            runs: 0,
        }
    }

    /// Get pos of this Env block
    pub const fn pos(&self) -> usize {
        self.id & ((1 << 10) - 1)
    }

    /// Get pgdir of this Env block
    pub fn pgdir(&self) -> PageDirectory {
        PageDirectory {
            page: Page::from(PPN::from(self.pgdir.paddr())),
        }
    }

    /// Get EnvTracker of this Env block, EnvTracker is created from env pos
    const fn tracker(&self) -> EnvTracker {
        EnvTracker::new(self.pos())
    }

    /// Load icode from binary to this env block
    fn load_icode(&mut self, binary: &[u8]) {
        if Elf32::is_elf32_format(binary) {
            let elf = Elf32::from_bytes(binary);
            let ehdr = elf.ehdr();
            for i in 0..ehdr.e_phnum as usize {
                let phdr = elf.phdr(i);
                if phdr.p_type == PT_LOAD as u32
                    && elf_load_seg(
                        phdr,
                        &binary[phdr.p_offset as usize..],
                        load_icode_mapper,
                        self,
                    )
                    .is_err()
                {
                    panic!();
                }
            }
            self.tf.cp0_epc = ehdr.e_entry;
        } else {
            panic!("bad elf at 0x{:p}", binary);
        }
    }

    /// Check if this env's status is EnvStatus::Runnable
    pub fn runnable(&self) -> bool {
        self.status == EnvStatus::Runnable
    }

    /// Set user custom tlm mod exception handler from handler entry
    pub fn set_tlb_mod_entry(&mut self, entry: usize) {
        self.user_tlb_mod_entry = entry;
    }
}

/// Envs array, same as ENVS in mos
#[repr(C, align(4096))]
pub struct Envs {
    env_array: [Env; NENV],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnvTracker {
    pos: usize,
}

impl EnvTracker {
    pub const fn new(pos: usize) -> Self {
        Self { pos }
    }
}

/// Data structure used to manage Env blocks
pub struct EnvManager {
    base_pgdir: PageDirectory,
    free_list: RefCell<Vec<EnvTracker>>,
    schedule_list: RefCell<VecDeque<EnvTracker>>,
    cur: Option<EnvTracker>,
    cur_pgdir: PageDirectory,
}

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvManager {
    /// Create a new empty EnvManager
    pub const fn new() -> Self {
        Self {
            base_pgdir: PageDirectory::empty(),
            free_list: RefCell::new(Vec::new()),
            schedule_list: RefCell::new(VecDeque::new()),
            cur: None,
            cur_pgdir: PageDirectory::empty(),
        }
    }

    /// Init this EnvManager
    pub fn init(&mut self) {
        let mut free_list = Vec::with_capacity(NENV);
        for i in (0..NENV).rev() {
            free_list.push(EnvTracker::new(i));
        }
        let (base_pgdir, _) = PageDirectory::init().expect("failed to init base_pgdir");
        let (ppn, page_count) = PAGE_ALLOCATOR.lock().get_tracker_info();
        map_segment(
            base_pgdir,
            0,
            ppn.into(),
            VA(UPAGES),
            page_count * PAGE_SIZE,
            PteFlags::G,
        );
        map_segment(
            base_pgdir,
            0,
            VA(unsafe { addr_of_mut!(ENVS) } as usize).paddr(),
            VA(UENVS),
            round!(size_of::<Envs>(), PAGE_SIZE),
            PteFlags::G,
        );
        self.base_pgdir = base_pgdir;
        self.free_list = RefCell::new(free_list);
        self.schedule_list = RefCell::new(VecDeque::with_capacity(NENV));
    }

    /// Acquire Env block of current process
    pub fn curenv(&self) -> Option<&'static mut Env> {
        self.cur.map(|tracker| env_at(tracker.pos))
    }

    /// Acquire a free Env block
    ///
    /// # Returns
    ///
    /// free Env block on success, MosError on failure
    pub fn get_free_env(&self) -> Result<&'static mut Env, MosError> {
        if self.free_list.borrow().is_empty() {
            return Err(MosError::NoFreeEnv);
        }

        let tracker = self.free_list.borrow_mut().pop().unwrap();
        Ok(env_at(tracker.pos))
    }

    /// Implementation of envid2env in mos
    /// Acquire Env block from its id
    ///
    /// # Parameters
    ///
    /// * id: env id to acquire
    /// * check_perm: check for permission flags if set to true
    ///
    /// # Returns
    ///
    /// Env of id on success, MosError::BadEnv on failure
    pub fn env_from_id(&self, id: usize, check_perm: bool) -> Result<&'static mut Env, MosError> {
        if id == 0 {
            Ok(self.curenv().unwrap())
        } else {
            let pos = envx(id);
            let env = env_at(pos);
            if env.status == EnvStatus::Free || env.id != id {
                return Err(MosError::BadEnv);
            }
            if check_perm
                && self.curenv().unwrap().id != env.id
                && self.curenv().unwrap().id != env.parent_id
            {
                return Err(MosError::BadEnv);
            }
            Ok(env)
        }
    }

    /// Implementation of env_setup_vm in mos
    /// Set up vm of new env
    ///
    /// # Returns
    ///
    /// Ok(()) on success, MosError on failure
    fn setup_vm(&self, env: &mut Env) -> Result<(), MosError> {
        PageDirectory::init().map(|(pgdir, page)| {
            env.pgdir = pgdir.page.kaddr();
            unsafe {
                ptr::copy_nonoverlapping(
                    (self.base_pgdir.pte_at(VA(UTOP).pdx()) as *const Pte).cast::<u8>(),
                    (env.pgdir().pte_at(VA(UTOP).pdx()) as *mut Pte).cast::<u8>(),
                    size_of::<usize>() * (VA(UVPT).pdx() - VA(UTOP).pdx()),
                );
            }
            *env.pgdir().pte_at(VA(UVPT).pdx()) = Pte::new(page.ppn(), PteFlags::V);
        })
    }

    /// Allocate a new Env block, set its parent id to parent_id
    ///
    /// # Returns
    ///
    /// Allocated Env block on success, MosError on failure
    pub fn alloc(&self, parent_id: usize) -> Result<&'static mut Env, MosError> {
        if let Ok(env) = self.get_free_env() {
            self.setup_vm(env)?;
            env.user_tlb_mod_entry = 0;
            env.runs = 0;
            env.id = mkenvid(env);
            env.asid = match asid_alloc() {
                Ok(asid) => asid,
                Err(_) => return Err(MosError::NoFreeEnv),
            };
            env.parent_id = parent_id;
            env.tf.cp0_status = (STATUS_IM7 | STATUS_IE | STATUS_EXL | STATUS_UM) as u32;
            env.tf.regs[29] = (USTACKTOP - size_of::<i32>() - size_of::<usize>()) as u32;
            Ok(env)
        } else {
            Err(MosError::NoFreeEnv)
        }
    }

    /// Create a Env from binary file, and set its priority
    pub fn create(&self, binary: &[u8], priority: u32) -> &mut Env {
        let env = self.alloc(0).expect("failed to alloc env");
        env.priority = priority;
        env.status = EnvStatus::Runnable;
        env.load_icode(binary);
        self.schedule_list.borrow_mut().push_back(env.tracker());
        env
    }

    /// Free a env
    pub fn env_free(&self, env: &mut Env) {
        if let Some(curenv) = self.curenv() {
            if curenv.id != env.id && curenv.id != env.parent_id {
                warn!(
                    "{:08x} try to free {:08x} which is not its child",
                    curenv.id, env.id
                );
            }
            info!("{:08x} free env {:08x}", curenv.id, env.id);
        } else {
            info!("kernel free env {:08x}", env.id);
        }
        for i in 0..VA(UTOP).pdx() {
            if !env.pgdir().pte_at(i).is_valid() {
                continue;
            }
            let pa: PA = env.pgdir().pte_at(i).ppn().into();
            let pt = pa.kaddr().as_mut_ptr::<Pte>();
            for j in 0..PAGE_SIZE / size_of::<Pte>() {
                let pte = unsafe { &mut *pt.add(j) };
                if pte.is_valid() {
                    env.pgdir()
                        .remove(env.asid, VA((i << PDSHIFT) + (j << PGSHIFT)));
                }
            }
            unsafe { *pt = Pte::empty() };
            page_dec_ref(pa.into());
            tlb_invalidate(env.asid, VA(UVPT + (i << PGSHIFT)));
        }
        pool_remove_user_on_exit(env.id);
        page_dec_ref(env.pgdir().page);
        asid_free(env.asid);
        tlb_invalidate(env.asid, VA(UVPT + (VA(UVPT).pdx() << PGSHIFT)));
        env.status = EnvStatus::Free;
        self.free_list.borrow_mut().push(env.tracker());
        self.schedule_list
            .borrow_mut()
            .retain(|&x| x != env.tracker());
    }

    /// Get first Env of env schedule list
    pub fn get_first(&self) -> Option<&'static mut Env> {
        self.schedule_list
            .borrow()
            .front()
            .map(|tracker| env_at(tracker.pos))
    }

    /// Insert a env to env schedule list
    ///
    /// # Parameters
    ///
    /// * envid: env to be inserted
    pub fn insert_to_end(&self, envid: usize) {
        self.schedule_list
            .borrow_mut()
            .push_back(env_at(envx(envid)).tracker());
    }

    /// Remove a env from env schedule list
    ///
    /// # Parameters
    ///
    /// * envid: env to be removed
    pub fn remove_from_schedule(&self, envid: usize) {
        self.schedule_list
            .borrow_mut()
            .retain(|&x| x != env_at(envx(envid)).tracker());
    }

    /// Move a env to end of the env schedule list
    pub fn move_to_end(&self, env: &Env) {
        let tracker = env.tracker();
        // tracker should be the first element (?)
        assert!(self.schedule_list.borrow_mut().pop_front() == Some(tracker));
        self.schedule_list.borrow_mut().push_back(tracker);
    }

    /// Acquire current page directory
    pub fn cur_pgdir(&mut self) -> &mut PageDirectory {
        &mut self.cur_pgdir
    }
}

/// Implementation of map_segment in mos
/// Map [va, va+size) of virtual address space to physical [pa, pa+size) in the 'pgdir'. Use
/// permission bits 'perm | PTE_V' for the entries.
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

/// Acquire a new envid
fn mkenvid(env: &Env) -> usize {
    static mut ENV_COUNT: usize = 0;
    unsafe {
        ENV_COUNT += 1;
        (ENV_COUNT << 11)
            | ((env as *const Env as usize - addr_of_mut!(ENVS) as usize) / size_of::<Env>())
    }
}

/// Allocate an empty asid
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

/// Free an asid
fn asid_free(asid: usize) {
    let index = asid >> 5;
    let inner = asid & 31;
    unsafe {
        ASID_BITMAP[index] &= !(1 << inner);
    }
}

/// Implementation of ENVX in mos
pub const fn envx(id: usize) -> usize {
    id & ((1 << 10) - 1)
}

/// Acquire env at ENVS[index]
fn env_at(index: usize) -> &'static mut Env {
    if index >= NENV {
        panic!("index out of ENVS limit")
    }
    unsafe { &mut ENVS.env_array[index] }
}

/// Run a env
pub fn env_run(env: &mut Env) -> ! {
    assert!(env.status == EnvStatus::Runnable);
    let mut env_man = ENV_MANAGER.lock();
    if let Some(cur) = env_man.curenv() {
        cur.tf = unsafe { *Trapframe::from_memory(VA(KSTACKTOP - TF_SIZE)) };
    }
    env_man.cur = Some(env.tracker());
    env.runs += 1;

    env_man.cur_pgdir = env.pgdir();
    drop(env_man);
    unsafe { env_pop_trapframe(&mut env.tf, env.asid as u32) }
}

/// Destroy a env
pub fn env_destroy(env: &mut Env) {
    let mut env_man = ENV_MANAGER.lock();
    env_man.env_free(env);
    if env_man.cur.is_some() && env_man.cur.unwrap().pos == env.pos() {
        env_man.cur = None;
        info!("{:08x}: I am killed ...", env.id);
        drop(env_man);
        schedule(true);
    }
}

/// Implementation of env_pop_tf in mos
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
