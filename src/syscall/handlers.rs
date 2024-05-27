use super::mempool::do_mempool_op;
use crate::{
    error::MosError,
    exception::{Trapframe, TF_SIZE},
    mm::{
        addr::VA,
        layout::{
            is_dev_va_range, is_illegal_user_va, is_illegal_user_va_range, PteFlags, KSTACKTOP,
            UTOP,
        },
        page::{page_alloc, page_dealloc},
    },
    platform::{
        ioread_byte, ioread_half, ioread_word, iowrite_byte, iowrite_half, iowrite_word,
        print_char, read_char,
    },
    pm::{schedule, EnvStatus, IpcStatus, ENV_MANAGER},
};
use alloc::string::String;
use core::ptr;
use log::info;

/// This function is used to print a character on screen.
pub fn sys_putchar(char: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    print_char(char as u8 as char);
    0
}

/// This function is used to print a string of bytes on screen.
pub unsafe fn sys_print_console(s: u32, len: u32, _args3: u32, _args4: u32, _args5: u32) -> u32 {
    if (s + len) as usize > UTOP || s as usize >= UTOP || s.checked_add(len).is_none() {
        return (-(MosError::Inval as i32)) as u32;
    }
    let slice = core::slice::from_raw_parts(s as *const u8, len as usize);
    slice.iter().for_each(|&c| print_char(c as char));
    0
}

/// This function provides the environment id of current process.
pub unsafe fn sys_get_env_id(_arg1: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    ENV_MANAGER.curenv().unwrap().id as u32
}

/// Give up remaining CPU time slice for 'curenv'.
pub unsafe fn sys_yield(_arg1: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    schedule(true)
}

/// This function is used to destroy the current environment.
pub unsafe fn sys_env_destroy(envid: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    let env = ENV_MANAGER.env_from_id(envid as usize, true);
    match env {
        Ok(env) => {
            info!(
                "[{:08x}] destroying {:08x}",
                ENV_MANAGER.curenv().unwrap().id,
                env.id
            );
            ENV_MANAGER.env_destroy(env);
            0
        }
        Err(err) => (-(err as i32)) as u32,
    }
}

/// Register the entry of user space TLB Mod handler of 'envid'.
pub unsafe fn sys_set_tlb_mod_entry(
    envid: u32,
    func: u32,
    _arg3: u32,
    _arg4: u32,
    _arg5: u32,
) -> u32 {
    let env = ENV_MANAGER.env_from_id(envid as usize, false);
    match env {
        Ok(env) => {
            env.set_tlb_mod_entry(func as usize);
            0
        }
        Err(err) => (-(err as i32)) as u32,
    }
}

/// Allocate a physical page and map 'va' to it with 'perm' in the address space of 'envid'.
/// If 'va' is already mapped, that original page is sliently unmapped.
/// 'envid2env' should be used with 'checkperm' set, like in most syscalls, to ensure the target is
/// either the caller or its child.
pub unsafe fn sys_mem_alloc(envid: u32, va: u32, perm: u32, _arg4: u32, _arg5: u32) -> u32 {
    if is_illegal_user_va(va as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.env_from_id(envid as usize, true);
    if let Err(err) = env {
        return (-(err as i32)) as u32;
    }
    let env = env.unwrap();
    if let Some(page) = page_alloc(true) {
        match env.pgdir().insert(
            env.asid,
            page,
            VA(va as usize),
            PteFlags::from_bits_truncate(perm as usize),
        ) {
            Ok(_) => 0,
            Err(err) => {
                page_dealloc(page);
                (-(err as i32)) as u32
            }
        }
    } else {
        (-(MosError::NoMem as i32)) as u32
    }
}

/// Find the physical page mapped at 'srcva' in the address space of env 'srcid', and map 'dstid''s
/// 'dstva' to it with 'perm'.
pub unsafe fn sys_mem_map(srcid: u32, srcva: u32, dstid: u32, dstva: u32, perm: u32) -> u32 {
    if is_illegal_user_va(srcva as usize) || is_illegal_user_va(dstva as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let srcenv = ENV_MANAGER.env_from_id(srcid as usize, true);
    let dstenv = ENV_MANAGER.env_from_id(dstid as usize, true);
    if let Err(err) = srcenv {
        return (-(err as i32)) as u32;
    }
    if let Err(err) = dstenv {
        return (-(err as i32)) as u32;
    }
    let srcenv = srcenv.unwrap();
    let dstenv = dstenv.unwrap();
    if let Some((_, page)) = srcenv.pgdir().lookup(VA(srcva as usize)) {
        match dstenv.pgdir().insert(
            dstenv.asid,
            page,
            VA(dstva as usize),
            PteFlags::from_bits_truncate(perm as usize),
        ) {
            Ok(_) => 0,
            Err(err) => (-(err as i32)) as u32,
        }
    } else {
        (-(MosError::Inval as i32)) as u32
    }
}

/// Unmap the physical page mapped at 'va' in the address space of 'envid'.
/// If no physical page is mapped there, this function silently succeeds.
pub unsafe fn sys_mem_unmap(envid: u32, va: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    if is_illegal_user_va(va as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.env_from_id(envid as usize, true);
    if let Err(err) = env {
        return (-(err as i32)) as u32;
    }
    let env = env.unwrap();
    env.pgdir().remove(env.asid, VA(va as usize));
    0
}

/// Allocate a new env as a child of 'curenv'.
pub unsafe fn sys_exofork(_arg1: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    let env = ENV_MANAGER.alloc(ENV_MANAGER.curenv().unwrap().id);
    match env {
        Ok(env) => {
            env.tf = *Trapframe::from_memory(VA(KSTACKTOP - TF_SIZE));
            env.tf.regs[2] = 0;
            env.status = EnvStatus::NotRunnable;
            env.priority = ENV_MANAGER.curenv().unwrap().priority;
            env.id as u32
        }
        Err(err) => (-(err as i32)) as u32,
    }
}

/// Set 'envid''s 'env_status' to 'status' and update 'env_sched_list'.
pub unsafe fn sys_set_env_status(
    envid: u32,
    status: u32,
    _arg3: u32,
    _arg4: u32,
    _arg5: u32,
) -> u32 {
    let status = match status {
        0 => EnvStatus::NotRunnable,
        1 => EnvStatus::Runnable,
        _ => return (-(MosError::Inval as i32)) as u32,
    };
    let env = ENV_MANAGER.env_from_id(envid as usize, true);
    match env {
        Ok(env) => {
            if status == EnvStatus::Runnable && env.status == EnvStatus::NotRunnable {
                ENV_MANAGER.insert_to_end(env.id);
            } else if status == EnvStatus::NotRunnable && env.status == EnvStatus::Runnable {
                ENV_MANAGER.remove_from_schedule(env.id);
            }
            env.status = status;
            0
        }
        Err(err) => (-(err as i32)) as u32,
    }
}

/// Set envid's trap frame to 'tf'.
pub unsafe fn sys_set_trapframe(envid: u32, tf: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    if is_illegal_user_va_range(tf as usize, TF_SIZE) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.env_from_id(envid as usize, true);
    match env {
        Ok(env) => {
            if env.id == ENV_MANAGER.curenv().unwrap().id {
                ptr::copy_nonoverlapping(
                    tf as *const u8,
                    (KSTACKTOP - TF_SIZE) as *mut u8,
                    TF_SIZE,
                );
                (*(tf as *const Trapframe)).regs[2]
            } else {
                env.tf = *Trapframe::from_memory(VA(tf as usize));
                0
            }
        }
        Err(err) => (-(err as i32)) as u32,
    }
}

/// Kernel panic with message `msg`.
// TODO: There may be a more elegant way to handle this
pub unsafe fn sys_panic(msg: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    let mut str = String::new();
    let mut i = 0;
    loop {
        let ptr = (msg as *const u8).add(i);
        if is_illegal_user_va(ptr as usize) {
            break;
        }
        let c = *ptr;
        if c == 0 {
            break;
        }
        str.push(c as char);
        i += 1;
    }
    panic!("{}", str);
}

/// Try to send a 'value' (together with a page if 'srcva' is not 0) to the target env 'envid'.
pub unsafe fn sys_ipc_try_send(envid: u32, value: u32, srcva: u32, perm: u32, _arg5: u32) -> u32 {
    if srcva != 0 && is_illegal_user_va(srcva as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.env_from_id(envid as usize, false);
    match env {
        Ok(env) => {
            let ipc_info = &mut env.ipc_info;
            if ipc_info.recving == IpcStatus::NotReceiving {
                return (-(MosError::IpcNotRecv as i32)) as u32;
            }
            ipc_info.recving = IpcStatus::NotReceiving;
            ipc_info.value = value;
            ipc_info.from = ENV_MANAGER.curenv().unwrap().id;
            ipc_info.perm = perm as usize | PteFlags::V.bits();

            env.status = EnvStatus::Runnable;
            ENV_MANAGER.insert_to_end(env.id);

            if srcva != 0 {
                if let Some((_, page)) = ENV_MANAGER
                    .curenv()
                    .unwrap()
                    .pgdir()
                    .lookup(VA(srcva as usize))
                {
                    let dstva = ipc_info.dstva;
                    match env.pgdir().insert(
                        env.asid,
                        page,
                        dstva,
                        PteFlags::from_bits_truncate(perm as usize),
                    ) {
                        Ok(_) => 0,
                        Err(err) => (-(err as i32)) as u32,
                    }
                } else {
                    (-(MosError::Inval as i32)) as u32
                }
            } else {
                0
            }
        }

        Err(err) => (-(err as i32)) as u32,
    }
}

/// Wait for a message (a value, together with a page if 'dstva' is not 0) from other envs.
/// 'curenv' is blocked until a message is sent.
pub unsafe fn sys_ipc_recv(dstva: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    if dstva != 0 && is_illegal_user_va(dstva as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.curenv().unwrap();
    let ipc_info = &mut env.ipc_info;
    ipc_info.recving = IpcStatus::Receiving;
    ipc_info.dstva = VA(dstva as usize);
    env.status = EnvStatus::NotRunnable;
    ENV_MANAGER.remove_from_schedule(env.id);
    (*Trapframe::from_memory(VA(KSTACKTOP - TF_SIZE))).regs[2] = 0;
    schedule(true)
}

/// This function gets char from console
pub fn sys_getchar(_arg1: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    let mut c: char;
    loop {
        c = read_char();
        if c != '\0' {
            break;
        }
    }
    c as u32
}

/// This function is used to write data at 'va' with length 'len' to a device physical address
/// 'pa'. Remember to check the validity of 'va' and 'pa'.
/// 
/// 'va' is the starting address of source data, 'len' is the
/// length of data (in bytes), 'pa' is the physical address of
/// the device (maybe with a offset).
pub unsafe fn sys_write_dev(va: u32, pa: u32, len: u32, _arg4: u32, _arg5: u32) -> u32 {
    if len != 1 && len != 2 && len != 4 {
        return (-(MosError::Inval as i32)) as u32;
    }
    if is_illegal_user_va_range(va as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    if !is_dev_va_range(pa as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    match len {
        1 => iowrite_byte(pa as usize, *(va as *const u8)),
        2 => iowrite_half(pa as usize, *(va as *const u16)),
        4 => iowrite_word(pa as usize, *(va as *const u32)),
        _ => unreachable!(),
    }
    0
}

/// This function is used to read data from a device physical address.
pub unsafe fn sys_read_dev(va: u32, pa: u32, len: u32, _arg4: u32, _arg5: u32) -> u32 {
    if len != 1 && len != 2 && len != 4 {
        return (-(MosError::Inval as i32)) as u32;
    }
    if is_illegal_user_va_range(va as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    if !is_dev_va_range(pa as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    match len {
        1 => *(va as *mut u8) = ioread_byte(pa as usize),
        2 => *(va as *mut u16) = ioread_half(pa as usize),
        4 => *(va as *mut u32) = ioread_word(pa as usize),
        _ => unreachable!(),
    }
    0
}

/// Operations on memory pools
///
/// Available operations:
/// - `0`: Create a memory pool
///     Parameter(s): `page_count`
/// - '1': Join an existing memory pool
///     Parameter(s): `poolid`, `va`, `page_count`
/// - '2': Leave a memory pool
///     Parameter(s): `poolid`
/// - '3': Destroy a memory pool
///     Parameter(s): `poolid`
/// - '4': Acquire write access to a memory pool
///     Parameter(s): `poolid`
/// - '5': Release write access to a memory pool
///     Parameter(s): `poolid`
/// - '6': Acquire read access to a memory pool
///     Parameter(s): `poolid`
/// - '7': Release read access to a memory pool
///     Parameter(s): `poolid`
///
/// # Parameters
///
/// - `op`: The operation to be performed
/// - `poolid`: The ID of the memory pool
/// - `va`: The virtual address to be mapped to the pool
/// - `page_count`: The number of pages to be allocated
///
///
pub fn sys_mempool_op(op: u32, poolid: u32, va: u32, page_count: u32, _arg5: u32) -> u32 {
    do_mempool_op(op, poolid, va, page_count)
}
