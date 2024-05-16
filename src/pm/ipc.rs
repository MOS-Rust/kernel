#![allow(dead_code)]
use crate::mm::addr::VA;

#[derive(Clone, Copy, Debug)]
pub enum IpcStatus {
    Receiving,
    NotReceiving,
}

#[derive(Clone, Copy, Debug)]
pub struct IpcInfo {
    value: u32,
    from: usize,
    recving: IpcStatus,
    dstva: VA,
    perm: usize,
}

impl IpcInfo {
    pub const fn new() -> IpcInfo {
        IpcInfo {
            value: 0,
            from: 0,
            recving: IpcStatus::NotReceiving,
            dstva: VA(0),
            perm: 0,
        }
    }
}
