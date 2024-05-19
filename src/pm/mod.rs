use log::info;

use crate::{error::MosError, mm::map::PageDirectory, pm::schedule::schedule};

use self::env::{Env, EnvManager};

mod elf;
pub mod env;
pub mod ipc;
pub mod schedule;

pub static mut ENV_MANAGER: EnvManager = EnvManager::new();

macro_rules! env_create {
    ($name: ident, $path: expr, $priority: expr) => {
        let $name = include_bytes!($path);
        unsafe {
            ENV_MANAGER.create($name, $priority);
        }
    };
}

pub fn init() {
    unsafe { ENV_MANAGER.init() };
    info!("Process manager initialized.");
    env_create!(test, "../../user/bare/overflow.b", 1);
    unsafe { schedule(true); }
}

// TODO: Deprecated
pub fn env_alloc(parent_id: usize) -> Result<&'static mut Env, MosError> {
    unsafe {ENV_MANAGER.alloc(parent_id)}
}

// TODO: Deprecated
pub fn env_free(env: &mut Env) {
    unsafe {ENV_MANAGER.env_free(env)}
}

// TODO: Deprecated
pub fn get_base_pgdir() -> PageDirectory {
    unsafe {*ENV_MANAGER.base_pgdir() }
}

