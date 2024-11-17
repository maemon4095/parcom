use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

#[derive(Debug)]
pub struct OnceInit<T> {
    initialied: AtomicBool,
    mutex: Mutex<()>,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Send for OnceInit<T> {}
unsafe impl<T: Sync + Send> Sync for OnceInit<T> {}

impl<T> OnceInit<T> {
    pub fn new() -> Self {
        Self {
            initialied: AtomicBool::new(false),
            mutex: Mutex::new(()),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.initialied.load(Ordering::Acquire) {
            unsafe { Some((*self.value.get()).assume_init_ref()) }
        } else {
            None
        }
    }

    pub fn if_uninitialized<U>(&self, f: impl FnOnce() -> U) -> Option<U> {
        if self.initialied.load(Ordering::Acquire) {
            return None;
        }

        let lock = self.mutex.lock();

        if !self.initialied.load(Ordering::Acquire) {
            return Some(f());
        }

        drop(lock);
        None
    }

    pub fn init(&self, value: T, f: impl FnOnce()) -> Result<(), T> {
        if self.initialied.load(Ordering::Acquire) {
            return Err(value);
        }

        let lock = self.mutex.lock();

        if self.initialied.load(Ordering::Acquire) {
            return Err(value);
        } else {
            unsafe {
                *self.value.get() = MaybeUninit::new(value);
            }
            self.initialied.store(true, Ordering::Release);
        }

        f();
        drop(lock);
        Ok(())
    }
}
