//! OS Error Codes
#![allow(dead_code)] // TODO: Remove this

// OS Error Codes

/// Unspecified or unknown problem
pub const E_UNSPECIFIED: i32 = -1;

/// Environment doesn't exist or otherwise cannot be used in requested action
pub const E_BAD_ENV: i32 = -2;

/// Invalid parameter
pub const E_INVAL: i32 = -3;

/// Request failed due to memory shortage
pub const E_NO_MEM: i32 = -4;

/// Invalid syscall number
pub const E_NO_SYS: i32 = -5;

/// Attempt to create a new environment beyond the maximum allowed
pub const E_NO_FREE_ENV: i32 = -6;

/// Attempt to send to env that is not recving.
pub const E_IPC_NOT_RECV: i32 = -7;

// File system error codes -- only seen in user-level

/// No free space left on disk
pub const E_NO_DISK: i32 = -8;

/// Too many files are open
pub const E_MAX_OPEN: i32 = -9;

/// File or block not found
pub const E_NOT_FOUND: i32 = -10;

/// Bad path
pub const E_BAD_PATH: i32 = -11;

/// File already exists
pub const E_FILE_EXISTS: i32 = -12;

/// File not a valid executable
pub const E_NOT_EXEC: i32 = -13;

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
    /// File or block not found
    NotFound,
    /// Bad path
    BadPath,
    /// File already exists
    FileExists,
    /// File not a valid executable
    NotExec,
}