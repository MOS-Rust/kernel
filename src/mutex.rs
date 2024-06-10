use core::{cell::UnsafeCell, hint::spin_loop, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

pub trait Mutex<T: ?Sized>: Sync {
    fn new(data: T) -> Self;
    fn lock(&self) -> impl MutexGuard<T>;

    unsafe fn force_unlock(&self);
}

#[allow(drop_bounds)]
pub trait MutexGuard<T: ?Sized>: Deref<Target = T> + DerefMut + Drop {}

pub struct SpinMutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinMutex<T> {}

impl<T> Mutex<T> for SpinMutex<T> {
    fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    fn lock(&self) -> impl MutexGuard<T> {
        // let ra: u32;
        // unsafe {asm!(
        //     "move $8, $31",
        //     out("$8") ra,
        // );}
        // debug!("Lock acquired at 0x{:08x} for T: {:?}", ra, core::any::type_name::<T>());
        if self.obtain_lock() {
            SpinMutexGuard { mutex: self }
        } else {
            panic!("Deadlock detected");
        }
    }

    unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<T> SpinMutex<T> {
    fn obtain_lock(&self) -> bool {
        while self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) != Ok(false) {
            const SPIN_COUNT: usize = 100000;
            let mut tries = 0;
            while self.lock.load(Ordering::Relaxed) {
                spin_loop();
                tries += 1;
                if tries == SPIN_COUNT {
                    return false;
                }
            }
        }
        true
    }
}


pub struct SpinMutexGuard<'a, T: ?Sized> {
    mutex: &'a SpinMutex<T>,
}

impl<T> MutexGuard<T> for SpinMutexGuard<'_, T> {}

impl<'a, T: ?Sized> Deref for SpinMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        //debug!("Lock released for T: {:?}", core::any::type_name::<T>());
        self.mutex.lock.store(false, Ordering::Release);
    }
}

// A real lock is not needed when interrupts are disabled, and only one core is running.
pub struct FakeLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for FakeLock<T> {}

impl<T> Mutex<T> for FakeLock<T> {
    fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    fn lock(&self) -> impl MutexGuard<T> {
        FakeGuard { lock: self }
    }

    unsafe fn force_unlock(&self) {}
}

pub struct FakeGuard<'a, T: ?Sized> {
    lock: &'a FakeLock<T>,
}

impl<T> MutexGuard<T> for FakeGuard<'_, T> {}

impl<'a, T: ?Sized> Deref for FakeGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for FakeGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for FakeGuard<'a, T> {
    fn drop(&mut self) {}
}
