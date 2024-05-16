use log::info;

use crate::{error::MosError, mm::map::PageDirectory, pm::schedule::schedule, test};

use self::env::{Env, EnvManager};

mod elf;
mod env;
mod ipc;
pub mod schedule;

pub static mut ENV_MANAGER: EnvManager = EnvManager::new();

pub fn init() {
    unsafe { ENV_MANAGER.init() };
    info!("Process manager initialized.");
    test!(EnvTest);
    test_loop();
    schedule(true);
}

pub fn env_alloc(parent_id: usize) -> Result<&'static mut Env, MosError> {
    unsafe {ENV_MANAGER.alloc(parent_id)}
}

pub fn env_free(env: &mut Env) {
    unsafe {ENV_MANAGER.env_free(env)}
}

pub fn get_base_pgdir() -> PageDirectory {
    unsafe {*ENV_MANAGER.base_pgdir() }
}

fn test_loop() {
    let loop_bin = include_bytes!("../../idle.b");
    unsafe { ENV_MANAGER.create(loop_bin, 1) };
}
