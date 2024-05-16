use log::info;

use crate::{pm::schedule::schedule, test};

use self::env::EnvManager;

mod elf;
pub mod env;
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

fn test_loop() {
    let loop_bin = include_bytes!("../../idle.b");
    unsafe { ENV_MANAGER.create(loop_bin, 1) };
}
