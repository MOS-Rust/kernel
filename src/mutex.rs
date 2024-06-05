use core::{cell::UnsafeCell, hint::spin_loop, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

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

    #[allow(clippy::mut_from_ref)]
    pub fn lock(&self) -> MutexGuard<T> {
        // let ra: u32;
        // unsafe {asm!(
        //     "move $8, $31",
        //     out("$8") ra,
        // );}
        // debug!("Lock acquired at 0x{:08x} for T: {:?}", ra, core::any::type_name::<T>());
        if self.obtain_lock() {
            MutexGuard { mutex: self }
        } else {
            panic!("Deadlock detected");
        }
    }

    #[allow(dead_code)]
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        //debug!("Lock released for T: {:?}", core::any::type_name::<T>());
        self.mutex.lock.store(false, Ordering::Release);
    }
}