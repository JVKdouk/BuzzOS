// /// A guard that provides mutable data access.
// ///
// /// When the guard falls out of scope it will release the lock.
// pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
//     lock: &'a AtomicBool,
//     data: *mut T,
// }

// pub struct SpinMutex<T: ?Sized, R = Spin> {
//     phantom: PhantomData<R>,
//     pub(crate) lock: AtomicBool,
//     data: UnsafeCell<T>,
// }

// pub struct Mutex<T: ?Sized, R = Spin> {
//     inner: InnerMutex<T, R>,
// }
