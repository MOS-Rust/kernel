//! OS Error Codes

#[allow(dead_code)]
#[derive(Debug)]
pub enum MosError {
    Unspecified = 1,
    /// Environment doesn't exist or otherwise cannot be used in requested action
    BadEnv,
    /// Invalid parameter
    Inval,
    /// Request failed due to memory shortage
    NoMem,
    /// Invalid syscall number
    NoSys,
    /// Attempt to create a new environment beyond the maximum allowed
    NoFreeEnv,
    /// Attempt to send to env that is not receiving
    IpcNotRecv,
    /// No free space left on disk
    NoDisk,
    /// Too many files are open
    MaxOpen,
    /// File or block or pool not found
    NotFound,
    /// Bad path
    BadPath,
    /// File already exists
    FileExists,
    /// File not a valid executable
    NotExec,
    /// Another kind of pool operation is in progress
    PoolBusy,
    /// Did not gracefully release pool when leaving
    PoolNotReleased,
}

impl From<MosError> for u32 {
    fn from(e: MosError) -> u32 {
        (-(e as i32)) as u32
    }
}
