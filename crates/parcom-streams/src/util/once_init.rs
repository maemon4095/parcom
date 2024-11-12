use std::{
    cell::UnsafeCell,
    future::Future,
    mem::MaybeUninit,
    sync::{
        atomic::{self, AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

use futures::lock::Mutex;

#[derive(Debug)]
pub struct OnceCell<T> {
    set: AtomicBool,
    waiters_count: AtomicU32,
    lock: UnsafeCell<MaybeUninit<Mutex<()>>>,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    pub fn new_initialized(value: T) -> Self {
        Self {
            set: AtomicBool::new(true),
            waiters_count: AtomicU32::new(0),
            lock: UnsafeCell::new(MaybeUninit::uninit()),
            value: UnsafeCell::new(MaybeUninit::new(value)),
        }
    }

    pub fn new() -> Self {
        Self {
            set: AtomicBool::new(false),
            waiters_count: AtomicU32::new(0),
            lock: UnsafeCell::new(MaybeUninit::new(Mutex::new(()))),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    fn is_initialized(&self) -> bool {
        self.set.load(Ordering::Acquire)
    }

    unsafe fn get_value_unchecked(&self) -> &T {
        (*self.value.get()).assume_init_ref()
    }

    unsafe fn get_lock_unchecked(&self) -> &Mutex<()> {
        (*self.lock.get()).assume_init_ref()
    }

    async unsafe fn try_ensure_init<E, F: Future<Output = Result<T, E>>>(
        &self,
        f: F,
    ) -> Result<(), E> {
        if self.is_initialized() {
            return Ok(());
        }

        if self.waiters_count.fetch_add(1, Ordering::Relaxed) >= u32::MAX / 2 {
            panic!()
        }

        let lock = self.get_lock_unchecked().lock().await;

        if !self.is_initialized() {
            let v = match f.await {
                Ok(v) => v,
                Err(e) => {
                    drop(lock);
                    self.waiters_count.fetch_sub(1, Ordering::Release);
                    return Err(e);
                }
            };

            (*self.value.get()) = MaybeUninit::new(v);

            self.set.store(true, Ordering::Release);
        }

        drop(lock);

        if self.waiters_count.fetch_sub(1, Ordering::Release) == 1 {
            atomic::fence(Ordering::Acquire);
            (*self.lock.get()).assume_init_drop();
        }

        Ok(())
    }

    pub async fn try_get_or_init<E, F: Future<Output = Result<T, E>>>(
        &self,
        f: F,
    ) -> Result<&T, E> {
        unsafe {
            self.try_ensure_init(f).await?;
            Ok(self.get_value_unchecked())
        }
    }

    pub async fn get_or_init<F: Future<Output = T>>(&self, f: F) -> &T {
        let Ok(v) = self
            .try_get_or_init(async { Ok::<_, Never>(f.await) })
            .await;

        v
    }

    pub async fn try_get_or_init_owned<E, F: Future<Output = Result<T, E>>>(
        self: Arc<OnceCell<T>>,
        f: F,
    ) -> Result<InitializedSharedCell<T>, E> {
        unsafe {
            self.try_ensure_init(f).await?;
            Ok(InitializedSharedCell { inner: self })
        }
    }

    pub async fn get_or_init_owned<F: Future<Output = T>>(
        self: Arc<OnceCell<T>>,
        f: F,
    ) -> InitializedSharedCell<T> {
        let Ok(v) = self
            .try_get_or_init_owned(async { Ok::<_, Never>(f.await) })
            .await;
        v
    }
}

impl<T> Drop for OnceCell<T> {
    fn drop(&mut self) {
        // dropが起きた時点でwaiterは存在しない。
        if self.is_initialized() {
            // 初期化されておりwaiterが存在しない場合、lockは既にdropされている。
            unsafe {
                (*self.value.get()).assume_init_drop();
            }
        } else {
            // 初期化されていない場合は、lockはdropされていない。
            unsafe {
                (*self.lock.get()).assume_init_drop();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitializedSharedCell<T> {
    inner: Arc<OnceCell<T>>,
}

impl<T> InitializedSharedCell<T> {
    pub fn into_cell(self) -> Arc<OnceCell<T>> {
        self.inner
    }
}

impl<T> std::ops::Deref for InitializedSharedCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.get_value_unchecked() }
    }
}

enum Never {}

#[cfg(test)]
mod test {
    use std::{
        sync::{Barrier, Mutex},
        thread,
    };

    use super::*;

    #[test]
    fn test_init_once() {
        for _ in 0..100 {
            test();
        }

        fn test() {
            let cell = Arc::new(OnceCell::new());

            let init_values = Arc::new(Mutex::new(Vec::new()));

            let n = 0xFF;
            let barrier = Arc::new(Barrier::new(n));
            let handles: Vec<_> = (0..0xFF)
                .map(|i| {
                    let barrier = Arc::clone(&barrier);
                    let cell = Arc::clone(&cell);
                    let init_values = Arc::clone(&init_values);

                    thread::spawn(move || {
                        barrier.wait();
                        pollster::block_on(async {
                            cell.get_or_init(async {
                                init_values.lock().unwrap().push(i);
                                i
                            })
                            .await;
                        });
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            let init_values = init_values.lock().unwrap();

            assert_eq!(init_values.len(), 1);

            let cell_value =
                pollster::block_on(async { cell.get_or_init(async { panic!() }).await });

            assert_eq!(*cell_value, init_values[0]);
        }
    }
}
