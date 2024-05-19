use log::info;

use crate::pm::schedule::schedule;

use self::env::EnvManager;

mod elf;
pub mod env;
pub mod ipc;
pub mod schedule;

pub static mut ENV_MANAGER: EnvManager = EnvManager::new();

pub fn init() {
    unsafe { ENV_MANAGER.init() };
    info!("Process manager initialized.");
    unsafe { schedule(true); }
}


#[repr(C)]
pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

/// Include a file as a byte slice aligned as a specific type.
#[macro_export]
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        use $crate::pm::AlignedAs;

        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
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
