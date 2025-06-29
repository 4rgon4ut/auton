use Ordering::{Acquire, Relaxed};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct OnceLock<T> {
    initialized: AtomicBool,
    inner: UnsafeCell<Option<T>>,
}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        OnceLock {
            initialized: AtomicBool::new(false),
            inner: UnsafeCell::new(None),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Acquire)
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            unsafe { (*self.inner.get()).as_ref() } // Updated field name
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe { (*self.inner.get()).as_mut() }
    }

    pub fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if self.is_initialized() {
            // SAFETY: We are guaranteed that the value is initialized
            unsafe { (*self.inner.get()).as_ref().unwrap_unchecked() }
        } else if self
            .initialized
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_ok()
        {
            // winning hart initializes the value
            let val = init();
            // SAFETY: we have exclusive logical access due to winning the CAS
            // write the `Some(val)` into the `UnsafeCell`
            unsafe {
                *self.inner.get() = Some(val);
                (*self.inner.get()).as_ref().unwrap_unchecked()
            }
        } else {
            // losing hart spins until the value is initialized by the winner
            while !self.is_initialized() {
                core::hint::spin_loop();
            }
            // SAFETY: `initialized` is now true, so `inner` is guaranteed to be `Some(T)`.
            unsafe { (*self.inner.get()).as_ref().unwrap_unchecked() }
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        if self.is_initialized() {
            // Use helper for consistency
            Err(value)
        } else if self
            .initialized
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_ok()
        {
            unsafe {
                *self.inner.get() = Some(value);
            }
            Ok(())
        } else {
            Err(value)
        }
    }
}

impl Default for OnceLock<()> {
    fn default() -> Self {
        OnceLock::new()
    }
}

// Ensure OnceLock is Send and Sync, allowing it to be shared across harts safely.
// OnceLock<T> is Send if T is Send, meaning ownership of the OnceLock can be moved between harts.
unsafe impl<T: Send> Send for OnceLock<T> {}
// OnceLock<T> is Sync if T is Send and Sync. This means a shared reference `&OnceLock<T>`
// can be safely accessed by multiple harts.
// - `T: Send` is needed because the value `T` might be moved into the OnceLock from another hart.
// - `T: Sync` is needed because once `T` is initialized, multiple harts might hold
//   shared references (`&T`) to it, and `T` must be safe for concurrent shared access.
unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}
