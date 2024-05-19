use log::info;

use crate::pm::schedule::schedule;

use self::env::EnvManager;

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