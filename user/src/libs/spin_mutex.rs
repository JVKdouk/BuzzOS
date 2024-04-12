use core::cell::UnsafeCell;
use core::default::Default;
use core::fmt;
use core::hint::spin_loop as cpu_relax;
use core::marker::Sync;
use core::ops::{Deref, DerefMut, Drop};
use core::option::Option::{self, None, Some};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub struct SpinMutex<T: ?Sized> {
    lock: AtomicBool,
    cpu: AtomicU8,
    data: UnsafeCell<T>, // We are providing the safety of this cell via locking
}

/// A guard to which the protected data can be accessed
/// When the guard falls out of scope it will release the lock.
#[derive(Debug)]
pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}

impl<T> SpinMutex<T> {
    pub const fn new(data: T) -> SpinMutex<T> {
        SpinMutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
            cpu: AtomicU8::new(0),
        }
    }

    pub fn into_inner(self) -> T {
        let SpinMutex { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized> SpinMutex<T> {
    fn obtain_lock(&self) {
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false {
            while self.lock.load(Ordering::Relaxed) {
                cpu_relax();
            }
        }
    }

    pub fn lock(&self) -> SpinMutexGuard<T> {
        self.obtain_lock();
        SpinMutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }

    pub fn try_lock(&self) -> Option<SpinMutexGuard<T>> {
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
            Some(SpinMutexGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "Mutex {{ data: ")
                .and_then(|()| (&*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "Mutex {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default> Default for SpinMutex<T> {
    fn default() -> SpinMutex<T> {
        SpinMutex::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for SpinMutexGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
