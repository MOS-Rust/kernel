use crate::mm::VA;

/// IpcStatus enum for Ipc feature
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IpcStatus {
    NotReceiving = 0,
    Receiving = 1,
}

/// IpcInfo enum, wraping env->env_ipc_* in original mos
#[repr(C)]
#[derive(Debug)]
pub struct IpcInfo {
    pub value: u32,
    pub from: usize,
    pub recving: IpcStatus,
    pub dstva: VA,
    pub perm: usize,
}

/// Info used in ipc
impl IpcInfo {
    /// Create a new empty IpcInfo struct, recving set to IpcStatus::NotReceiving
    pub const fn new() -> Self {
        Self {
            value: 0,
            from: 0,
            recving: IpcStatus::NotReceiving,
            dstva: VA(0),
            perm: 0,
        }
    }
}
