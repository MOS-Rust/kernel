use alloc::vec::Vec;

use crate::mm::map::Pde;
use crate::pm::trap::Trapframe;

pub enum EnvStatus {
    Free, Runnable, NotRunnable,
}

pub struct Env {
    tf: Trapframe,

    id: u32,
    asid: u32,
    parent_id: u32,
    status: EnvStatus,
    pgdir: Pde,
    env_pri: u32,

    // IPC
    ipc_value: u32,
    ipc_from: u32,
    ipc_recving: u32,
    ipc_dstva: u32,
    ipc_perm: u32,

    env_user_tlb_mod_entry: u32,

    env_runs: u32,
}

pub struct EnvManager {
    envs: Vec<Env>,
    free_list: Vec<EnvTracker>,
}

impl EnvManager {
    const fn new() -> EnvManager {
        EnvManager {
            
        }
    }
}