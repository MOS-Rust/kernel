mod elf;
mod env;
mod ipc;
mod schedule;

use crate::mutex::Mutex;
use lazy_static::lazy_static;
use log::info;

pub use env::env_destroy;
use env::EnvManager;
pub use env::EnvStatus;
pub use ipc::IpcStatus;
pub use schedule::schedule;

lazy_static! {
    pub static ref ENV_MANAGER: Mutex<EnvManager> = Mutex::new(EnvManager::new());
}

pub fn init() {
    ENV_MANAGER.lock().init();
    info!("Process manager initialized.");
}

/// Create an environment from a ELF file.
#[macro_export]
macro_rules! env_create {
    ($name: ident, $path: expr) => {
        let $name = include_bytes_align_as!(usize, $path);
        $crate::pm::ENV_MANAGER.lock().create($name, 1);
    };

    ($name: ident, $path: expr, $priority: expr) => {
        let $name = include_bytes_align_as!(usize, $path);
        $crate::pm::ENV_MANAGER.lock().create($name, $priority);
    };
}
