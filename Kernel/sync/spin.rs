use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

pub struct SpinMutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: *mut T,
}

unsafe impl<T: ?Sized + Send> Sync for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}

unsafe impl<T: ?Sized + Send> Sync for SpinMutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Send for SpinMutexGuard<'_, T> {}

impl<T> SpinMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinMutexGuard<T> {
        while self
            .lock
            .compare_exchange_weak(
                false,
                true,
                core::sync::atomic::Ordering::Acquire,
                core::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            while self.is_locked() {}
        }
        SpinMutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(core::sync::atomic::Ordering::Relaxed)
    }
}

impl<'a, T> Deref for SpinMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock
            .store(false, core::sync::atomic::Ordering::Release)
    }
}
