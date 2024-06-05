use crate::{
    error::MosError,
    mm::{
        layout::{is_illegal_user_va_range, PteFlags, PAGE_SIZE},
        page::{page_alloc, page_inc_ref, try_recycle, Page},
        VA,
    },
    pm::ENV_MANAGER,
};
use alloc::{collections::BTreeMap, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};
use log::warn;

static mut POOL_MANAGER: MemPoolManager = MemPoolManager {
    current_id: 1,
    pools: BTreeMap::new(),
};

struct MemPoolManager {
    current_id: u32,
    pools: BTreeMap<u32, MemPool>,
}

struct MemPool {
    id: u32,
    page_count: u32,
    pages: Vec<Page>,
    users: BTreeMap<usize, VA>,
    write_mutex: AtomicBool,
    write_lock: bool,
    writer: usize,
    read_mutex: AtomicBool,
    read_lock: u32,
    readers: Vec<usize>,
}

enum MemPoolOp {
    Create,
    Join,
    Leave,
    Destroy,
    AcquireWriteLock,
    ReleaseWriteLock,
    AcquireReadLock,
    ReleaseReadLock,
}

impl MemPoolOp {
    const fn from_u32(op: u32) -> Option<Self> {
        match op {
            0 => Some(Self::Create),
            1 => Some(Self::Join),
            2 => Some(Self::Leave),
            3 => Some(Self::Destroy),
            4 => Some(Self::AcquireWriteLock),
            5 => Some(Self::ReleaseWriteLock),
            6 => Some(Self::AcquireReadLock),
            7 => Some(Self::ReleaseReadLock),
            _ => None,
        }
    }
}

pub fn do_mempool_op(op: u32, poolid: u32, va: u32, page_count: u32) -> u32 {
    let Some(op) = MemPoolOp::from_u32(op) else {
        return (-(MosError::Inval as i32)) as u32;
    };
    match op {
        MemPoolOp::Create => mempool_create(page_count),
        MemPoolOp::Join => mempool_join(poolid, va, page_count),
        MemPoolOp::Leave => mempool_leave(poolid),
        MemPoolOp::Destroy => mempool_destroy(poolid),
        MemPoolOp::AcquireWriteLock => mempool_acquire_write_lock(poolid),
        MemPoolOp::ReleaseWriteLock => mempool_release_write_lock(poolid),
        MemPoolOp::AcquireReadLock => mempool_acquire_read_lock(poolid),
        MemPoolOp::ReleaseReadLock => mempool_release_read_lock(poolid),
    }
}

fn mempool_create(page_count: u32) -> u32 {
    let id = unsafe { POOL_MANAGER.current_id };
    let mut pool = MemPool {
        id,
        page_count,
        pages: Vec::new(),
        users: BTreeMap::new(),
        write_mutex: AtomicBool::new(false),
        write_lock: false,
        writer: 0,
        read_mutex: AtomicBool::new(false),
        read_lock: 0,
        readers: Vec::new(),
    };
    for _ in 0..page_count {
        let Some(page) = page_alloc(true) else {
            pool.pages.iter().for_each(|&page| try_recycle(page));
            return (-(MosError::NoMem as i32)) as u32;
        };
        page_inc_ref(page);
        pool.pages.push(page);
    }
    unsafe {
        POOL_MANAGER.pools.insert(id, pool);
        POOL_MANAGER.current_id += 1;
    }
    id
}

fn mempool_join(poolid: u32, va: u32, page_count: u32) -> u32 {
    if is_illegal_user_va_range(va as usize, page_count as usize * PAGE_SIZE) {
        return (-(MosError::Inval as i32)) as u32;
    }
    let env = ENV_MANAGER.lock().curenv().unwrap();
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if pool.page_count != page_count || pool.users.contains_key(&env.id) {
            return (-(MosError::Inval as i32)) as u32;
        }
        pool.users.insert(env.id, VA(va as usize));
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn mempool_leave(poolid: u32) -> u32 {
    let env = ENV_MANAGER.lock().curenv().unwrap();
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if !pool.users.contains_key(&env.id) {
            return (-(MosError::Inval as i32)) as u32;
        }
        if (pool.write_lock && pool.writer == env.id)
            || (pool.read_lock > 0 && pool.readers.contains(&env.id))
        {
            return (-(MosError::PoolNotReleased as i32)) as u32;
        }
        pool.users.remove(&env.id);
        // don't free the pool if the last user gracefully leaves
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn mempool_destroy(poolid: u32) -> u32 {
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if !pool.users.is_empty() {
            return (-(MosError::PoolBusy as i32)) as u32;
        }
        free_pool(poolid);
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn mempool_acquire_write_lock(poolid: u32) -> u32 {
    let env = ENV_MANAGER.lock().curenv().unwrap();
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if !pool.users.contains_key(&env.id) {
            return (-(MosError::Inval as i32)) as u32;
        }

        if pool
            .write_mutex
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return (-(MosError::PoolBusy as i32)) as u32;
        };
        if pool
            .read_mutex
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            pool.write_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::PoolBusy as i32)) as u32;
        };
        if pool.write_lock || pool.read_lock > 0 {
            pool.write_mutex.store(false, Ordering::Relaxed);
            pool.read_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::PoolBusy as i32)) as u32;
        }
        pool.write_lock = true;
        pool.writer = env.id;
        let asid = env.asid;
        let va = pool.users.get(&env.id).unwrap();
        let flags = PteFlags::V | PteFlags::D;
        if (va.0..va.0 + pool.page_count as usize * PAGE_SIZE)
            .step_by(PAGE_SIZE)
            .enumerate()
            .try_fold((), |(), (i, va)| {
                let va = VA(va);
                let page = pool.pages[i];
                if env.pgdir().insert(asid, page, va, flags).is_err() {
                    warn!("mempool_acquire_write_lock: insert failed");
                    (0..i).for_each(|j| env.pgdir().remove(asid, VA(va.0 + j * PAGE_SIZE)));
                    Err(())
                } else {
                    Ok(())
                }
            })
            .is_err()
        {
            pool.write_lock = false;
            pool.writer = 0;
            pool.read_mutex.store(false, Ordering::Relaxed);
            pool.write_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::NoMem as i32)) as u32;
        }
        pool.read_mutex.store(false, Ordering::Relaxed);
        pool.write_mutex.store(false, Ordering::Relaxed);
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn mempool_release_write_lock(poolid: u32) -> u32 {
    let env = ENV_MANAGER.lock().curenv().unwrap();
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if !pool.users.contains_key(&env.id) {
            return (-(MosError::Inval as i32)) as u32;
        }
        if pool
            .write_mutex
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return (-(MosError::PoolBusy as i32)) as u32;
        }
        if !pool.write_lock || pool.writer != env.id {
            pool.write_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::Inval as i32)) as u32;
        }
        let asid = env.asid;
        let va = pool.users.get(&env.id).unwrap();
        (va.0..va.0 + pool.page_count as usize * PAGE_SIZE)
            .step_by(PAGE_SIZE)
            .for_each(|va| env.pgdir().remove(asid, VA(va)));
        pool.write_lock = false;
        pool.writer = 0;
        pool.write_mutex.store(false, Ordering::Relaxed);
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn mempool_acquire_read_lock(poolid: u32) -> u32 {
    let env = ENV_MANAGER.lock().curenv().unwrap();
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if !pool.users.contains_key(&env.id) {
            return (-(MosError::Inval as i32)) as u32;
        }
        if pool
            .read_mutex
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return (-(MosError::PoolBusy as i32)) as u32;
        }

        if pool.write_mutex.load(Ordering::Relaxed) || pool.write_lock {
            pool.read_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::PoolBusy as i32)) as u32;
        }
        pool.read_lock += 1;
        pool.readers.push(env.id);
        let asid = env.asid;
        let va = pool.users.get(&env.id).unwrap();
        let flags = PteFlags::V;
        if (va.0..va.0 + pool.page_count as usize * PAGE_SIZE)
            .step_by(PAGE_SIZE)
            .enumerate()
            .try_fold((), |(), (i, va)| {
                let va = VA(va);
                let page = pool.pages[i];
                if env.pgdir().insert(asid, page, va, flags).is_err() {
                    warn!("mempool_acquire_read_lock: insert failed");
                    (0..i).for_each(|j| env.pgdir().remove(asid, VA(va.0 + j * PAGE_SIZE)));
                    Err(())
                } else {
                    Ok(())
                }
            })
            .is_err()
        {
            pool.read_lock -= 1;
            pool.readers.retain(|&reader| reader != env.id);
            pool.read_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::NoMem as i32)) as u32;
        }
        pool.read_mutex.store(false, Ordering::Relaxed);
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn mempool_release_read_lock(poolid: u32) -> u32 {
    let env = ENV_MANAGER.lock().curenv().unwrap();
    if let Some(pool) = unsafe { POOL_MANAGER.pools.get_mut(&poolid) } {
        if !pool.users.contains_key(&env.id) {
            return (-(MosError::Inval as i32)) as u32;
        }
        if pool
            .read_mutex
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return (-(MosError::PoolBusy as i32)) as u32;
        }
        if pool.read_lock == 0 || !pool.readers.contains(&env.id) {
            pool.read_mutex.store(false, Ordering::Relaxed);
            return (-(MosError::Inval as i32)) as u32;
        }
        let asid = env.asid;
        let va = pool.users.get(&env.id).unwrap();
        (va.0..va.0 + pool.page_count as usize * PAGE_SIZE)
            .step_by(PAGE_SIZE)
            .for_each(|va| env.pgdir().remove(asid, VA(va)));
        pool.read_lock -= 1;
        pool.readers.retain(|&reader| reader != env.id);
        pool.read_mutex.store(false, Ordering::Relaxed);
        0
    } else {
        (-(MosError::NotFound as i32)) as u32
    }
}

fn free_pool(poolid: u32) {
    assert!(unsafe { POOL_MANAGER.pools.contains_key(&poolid) });
    let pool = unsafe { POOL_MANAGER.pools.get_mut(&poolid).unwrap() };
    assert!(pool.users.is_empty());
    pool.pages.iter().for_each(|&page| try_recycle(page));
    unsafe {
        POOL_MANAGER.pools.remove(&poolid);
    }
}

pub fn pool_remove_user_on_exit(env_id: usize) {
    for pool in unsafe { POOL_MANAGER.pools.values_mut() } {
        if pool.users.contains_key(&env_id) {
            pool.users.remove(&env_id);
            if pool
                .write_mutex
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                // TODO: find a better way to handle this
                // cause kernel itself to hang
                // maybe a watchdog is needed
                while pool.write_mutex.load(Ordering::Relaxed) {
                    core::hint::spin_loop();
                }
                pool.write_mutex.store(true, Ordering::Relaxed);
            };
            if pool.write_lock && pool.writer == env_id {
                pool.write_lock = false;
                pool.writer = 0;
            }
            pool.write_mutex.store(false, Ordering::Relaxed);

            if pool
                .read_mutex
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                while pool.read_mutex.load(Ordering::Relaxed) {
                    core::hint::spin_loop();
                }
                pool.read_mutex.store(true, Ordering::Relaxed);
            };
            if pool.read_lock > 0 && pool.readers.contains(&env_id) {
                pool.read_lock -= 1;
                pool.readers.retain(|&reader| reader != env_id);
            }
            pool.read_mutex.store(false, Ordering::Relaxed);

            // if the last user exits unexpectedly, free the pool
            if pool.users.is_empty() {
                free_pool(pool.id);
            }
        }
    }
}
