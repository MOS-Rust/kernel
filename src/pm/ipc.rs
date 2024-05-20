use crate::mm::addr::VA;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcStatus {
    NotReceiving = 0,
    Receiving = 1,
}

#[repr(C)]
#[derive(Debug)]
pub struct IpcInfo {
    pub value: u32,
    pub from: usize,
    pub recving: IpcStatus,
    pub dstva: VA,
    pub perm: usize,
}

impl IpcInfo {
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
