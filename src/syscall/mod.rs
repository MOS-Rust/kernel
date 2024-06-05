mod handlers;
mod mempool;

use crate::{error::MosError, exception::Trapframe, pm::ENV_MANAGER};
use core::mem::size_of;
use log::trace;

pub use mempool::pool_remove_user_on_exit;

#[derive(Debug)]
enum Syscall {
    Putchar = 0,
    PrintConsole = 1,
    GetEnvId = 2,
    Yield = 3,
    EnvDestroy = 4,
    SetTlbModEntry = 5,
    MemAlloc = 6,
    MemMap = 7,
    MemUnmap = 8,
    Exofork = 9,
    SetEnvStatus = 10,
    SetTrapframe = 11,
    Panic = 12,
    IpcTrySend = 13,
    IpcRecv = 14,
    Getchar = 15,
    WriteDev = 16,
    ReadDev = 17,
    MempoolOp = 18,
    Unhandled = 19,
}

impl Syscall {
    const fn from_u32(num: u32) -> Self {
        match num {
            0 => Self::Putchar,
            1 => Self::PrintConsole,
            2 => Self::GetEnvId,
            3 => Self::Yield,
            4 => Self::EnvDestroy,
            5 => Self::SetTlbModEntry,
            6 => Self::MemAlloc,
            7 => Self::MemMap,
            8 => Self::MemUnmap,
            9 => Self::Exofork,
            10 => Self::SetEnvStatus,
            11 => Self::SetTrapframe,
            12 => Self::Panic,
            13 => Self::IpcTrySend,
            14 => Self::IpcRecv,
            15 => Self::Getchar,
            16 => Self::WriteDev,
            17 => Self::ReadDev,
            18 => Self::MempoolOp,
            _ => Self::Unhandled,
        }
    }
}

type SyscallHandler = unsafe fn(u32, u32, u32, u32, u32) -> u32;

const SYSCALL_NUM: usize = 19;

const HANDLER_TABLE: [SyscallHandler; SYSCALL_NUM] = [
    /* 00 */ handlers::sys_putchar,
    /* 01 */ handlers::sys_print_console,
    /* 02 */ handlers::sys_get_env_id,
    /* 03 */ handlers::sys_yield,
    /* 04 */ handlers::sys_env_destroy,
    /* 05 */ handlers::sys_set_tlb_mod_entry,
    /* 06 */ handlers::sys_mem_alloc,
    /* 07 */ handlers::sys_mem_map,
    /* 08 */ handlers::sys_mem_unmap,
    /* 09 */ handlers::sys_exofork,
    /* 10 */ handlers::sys_set_env_status,
    /* 11 */ handlers::sys_set_trapframe,
    /* 12 */ handlers::sys_panic,
    /* 13 */ handlers::sys_ipc_try_send,
    /* 14 */ handlers::sys_ipc_recv,
    /* 15 */ handlers::sys_getchar,
    /* 16 */ handlers::sys_write_dev,
    /* 17 */ handlers::sys_read_dev,
    /* 18 */ handlers::sys_mempool_op,
];

#[no_mangle]
pub unsafe extern "C" fn do_syscall(tf: *mut Trapframe) {
    let syscall_num: u32 = (*tf).regs[4];
    if !(0..SYSCALL_NUM as i32).contains(&(syscall_num as i32)) {
        (*tf).regs[2] = (-(MosError::NoSys as i32)) as u32;
        return;
    }
    (*tf).cp0_epc += size_of::<usize>() as u32;
    let handler: SyscallHandler = HANDLER_TABLE[syscall_num as usize];
    let arg1: u32 = (*tf).regs[5];
    let arg2: u32 = (*tf).regs[6];
    let arg3: u32 = (*tf).regs[7];
    let sp: u32 = (*tf).regs[29];
    let arg4: u32 = *(sp as *const u32).add(4);
    let arg5: u32 = *(sp as *const u32).add(5);

    trace!(
        "Env: {:08x}, Syscall: {} \"{:?}\"",
        ENV_MANAGER.lock().curenv().unwrap().id,
        syscall_num,
        Syscall::from_u32(syscall_num)
    );
    trace!(
        "Args: {:08x} {:08x} {:08x} {:08x} {:08x}",
        arg1,
        arg2,
        arg3,
        arg4,
        arg5
    );

    (*tf).regs[2] = handler(arg1, arg2, arg3, arg4, arg5);
}
