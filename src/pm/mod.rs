use log::info;

use self::env::EnvManager;

mod elf;
pub mod env;
pub mod ipc;
pub mod schedule;

pub static mut ENV_MANAGER: EnvManager = EnvManager::new();

pub fn init() {
    unsafe { ENV_MANAGER.init() };
    info!("Process manager initialized.");
}

/// Create an environment from a ELF file.
#[macro_export]
macro_rules! env_create {
    ($name: ident, $path: expr, $priority: expr) => {
        let $name = include_bytes_align_as!(usize, $path);
        unsafe {
            $crate::pm::ENV_MANAGER.create($name, $priority);
        }
    };
}
