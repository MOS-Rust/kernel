use log::info;

use self::env::EnvManager;

mod elf;
mod env;
mod ipc;
mod schedule;

static mut ENV_MANAGER: EnvManager = EnvManager::new();

pub fn init() {
    unsafe { ENV_MANAGER.init() };
    info!("Process manager initialized.")
}
