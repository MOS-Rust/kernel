//! This module provides implementations of mutexes for synchronizing access to shared resources.

use core::{cell::UnsafeCell, hint::spin_loop, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};


/// A trait representing a mutex.
pub trait Mutex<T: ?Sized>: Sync {
    /// Creates a new instance of the mutex with the specified data.
    ///
    /// # Arguments
    ///
    /// * `data` - The initial data to be stored in the mutex.
    ///
    /// # Returns
    ///
    /// A new instance of the mutex.
    fn new(data: T) -> Self;

    /// Locks the mutex and returns a guard that provides access to the locked data.
    ///
    /// # Returns
    ///
    /// A guard that provides access to the locked data.
    fn lock(&self) -> impl MutexGuard<T>;

    /// Forces the mutex to unlock, even if it is currently locked.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it can lead to data races if not used correctly.
    /// It should only be used in exceptional circumstances.
    unsafe fn force_unlock(&self);
}

#[allow(drop_bounds)]
/// A trait representing a guard for a mutex-protected resource.
///
/// This trait is used to define the behavior of a guard that provides exclusive access to a mutex-protected resource.
/// It combines the `Deref`, `DerefMut`, and `Drop` traits to allow convenient access and automatic release of the resource.
pub trait MutexGuard<T: ?Sized>: Deref<Target = T> + DerefMut + Drop {}

/// Spin-based mutex implementation
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

/// A guard that provides exclusive access to a spin mutex-protected resource.
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
/// Fake lock implementation
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

/// Fake guard
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
