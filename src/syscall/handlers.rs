use core::ptr::{self, read_volatile, write_volatile};

use alloc::string::String;
use log::{debug, info};

use crate::{
    error::MosError,
    exception::trapframe::{Trapframe, TF_SIZE},
    mm::{
        addr::VA,
        layout::{PteFlags, KSTACKTOP, UTEMP, UTOP},
        page::{page_alloc, page_dealloc},
    },
    platform::{malta::{IDE_BASE, SERIAL_BASE}, print_char, ioread_byte, read_char, ioread_half, ioread_word, iowrite_byte, iowrite_half, iowrite_word},
    pm::{env::EnvStatus, ipc::IpcStatus, schedule::schedule, ENV_MANAGER},
};

pub fn sys_putchar(char: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    print_char(char as u8 as char);
    0
}

pub unsafe fn sys_print_console(s: u32, len: u32, _args3: u32, _args4: u32, _args5: u32) -> u32 {
    if (s + len) as usize > UTOP || s as usize >= UTOP || s.checked_add(len).is_none() {
        return (-(MosError::Inval as i32)) as u32;
    }
    let slice = core::slice::from_raw_parts(s as *const u8, len as usize);
    slice.iter().for_each(|&c| print_char(c as char));
    0
}

pub unsafe fn sys_get_env_id(_arg1: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    ENV_MANAGER.curenv().unwrap().id as u32
}

/// The process actively gives up it's
pub unsafe fn sys_yield(_arg1: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    schedule(true)
}

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

pub unsafe fn sys_set_tlb_mod_entry(envid: u32, func: u32, _arg3: u32, _arg4: u32, _arg5: u32,
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

pub unsafe fn sys_mem_alloc(envid: u32, va: u32, perm: u32, _arg4: u32, _arg5: u32) -> u32 {
    if is_illegal_va(va as usize) {
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

pub unsafe fn sys_mem_map(srcid: u32, srcva: u32, dstid: u32, dstva: u32, perm: u32) -> u32 {
    if is_illegal_va(srcva as usize) || is_illegal_va(dstva as usize) {
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

pub unsafe fn sys_mem_unmap(envid: u32, va: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    if is_illegal_va(va as usize) {
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

pub unsafe fn sys_set_env_status(envid: u32, status: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
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

pub unsafe fn sys_set_trapframe(envid: u32, tf: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    if is_illegal_va_range(tf as usize, TF_SIZE) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.env_from_id(envid as usize, true);
    match env {
        Ok(env) => {
            if env.id == ENV_MANAGER.curenv().unwrap().id {
                ptr::copy_nonoverlapping(
                    tf as *const u8,
                    Trapframe::from_memory(VA(KSTACKTOP - TF_SIZE)) as *mut u8,
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

// TODO: There may be a more elegant way to handle this
pub unsafe fn sys_panic(msg: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    let mut str = String::new();
    let mut i = 0;
    loop {
        let ptr = (msg as *const u8).add(i);
        if is_illegal_va(ptr as usize) {
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

pub unsafe fn sys_ipc_try_send(envid: u32, value: u32, srcva: u32, perm: u32, _arg5: u32) -> u32 {
    if srcva != 0 && is_illegal_va(srcva as usize) {
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
            ipc_info.perm = perm as usize;

            env.status = EnvStatus::Runnable;
            ENV_MANAGER.insert_to_end(env.id);

            if srcva != 0 {
                if let Some((_, page)) = ENV_MANAGER.curenv().unwrap().pgdir().lookup(VA(srcva as usize)) {
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

pub unsafe fn sys_ipc_recv(dstva: u32, _arg2: u32, _arg3: u32, _arg4: u32, _arg5: u32) -> u32 {
    if dstva != 0 && is_illegal_va(dstva as usize) {
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

pub unsafe fn sys_write_dev(va: u32, pa: u32, len: u32, _arg4: u32, _arg5: u32) -> u32 {
    if len != 1 && len != 2 && len != 4 {
        return (-(MosError::Inval as i32)) as u32;
    }
    if is_illegal_va_range(va as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    if !is_dev_va_range(pa as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    // debug!("sys_write_dev: va: {:x}, pa: {:x}, len: {}, val: {:x}", va, pa, len, *(va as *const u32));
    match len {
        1 => iowrite_byte(pa as usize, *(va as *const u8)),
        2 => iowrite_half(pa as usize, *(va as *const u16)),
        4 => iowrite_word(pa as usize, *(va as *const u32)),
        _ => unreachable!(),
    }
    0
}

pub unsafe fn sys_read_dev(va: u32, pa: u32, len: u32, _arg4: u32, _arg5: u32) -> u32 {
    // debug!("sys_read_dev: va: {:x}, pa: {:x}, len: {}", va, pa, len);
    if len != 1 && len != 2 && len != 4 {
        return (-(MosError::Inval as i32)) as u32;
    }
    if is_illegal_va_range(va as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    if !is_dev_va_range(pa as usize, len as usize) {
        return (-(MosError::Inval as i32)) as u32;
    }
    match len {
        1 => *(va as *mut u8 ) = ioread_byte(pa as usize),
        2 => *(va as *mut u16) = ioread_half(pa as usize),
        4 => *(va as *mut u32) = ioread_word(pa as usize),
        _ => unreachable!(),
    }
    0
}

#[inline]
fn is_illegal_va(va: usize) -> bool {
    !(UTEMP..UTOP).contains(&va)
}

#[inline]
fn is_illegal_va_range(va: usize, len: usize) -> bool {
    if len == 0 {
        return false;
    }
    va < UTEMP || va.checked_add(len).is_none() || va.checked_add(len).unwrap() >= UTOP
}

#[inline]
fn is_dev_va_range(va: usize, len: usize) -> bool {
    const CONOLE_ADDR_LEN: usize = 0x20;
    const IDE_ADDR_LEN: usize = 0x8;
    (va >= SERIAL_BASE && va + len <= SERIAL_BASE + CONOLE_ADDR_LEN) || 
    (va >= IDE_BASE && va + len <= IDE_BASE + IDE_ADDR_LEN)
}