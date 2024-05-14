use log::info;

use crate::pm::schedule::schedule;

use self::env::EnvManager;

mod elf;
mod env;
mod ipc;
pub mod schedule;

pub static mut ENV_MANAGER: EnvManager = EnvManager::new();

pub fn init() {
    unsafe { ENV_MANAGER.init() };
    info!("Process manager initialized.");
    test_loop();
    schedule(true);
}

fn test_loop() {
    let loop_bin = include_bytes!("../../loop.b");
    unsafe { ENV_MANAGER.create(loop_bin, 1) };
}
