use core::hint::spin_loop as cpu_relax;
use core::sync::atomic::{AtomicBool, Ordering};

struct SleepLock {
    lock: AtomicBool,
}

impl SleepLock {
    pub fn acquire(&self) {
        // Keeps trying to update the value (until the value returned is false)
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false {
            while self.lock.load(Ordering::Relaxed) {
                cpu_relax();
            }
        }
    }
}

impl Default for SleepLock {
    fn default() -> Self {
        SleepLock {
            lock: AtomicBool::new(false),
        }
    }
}
