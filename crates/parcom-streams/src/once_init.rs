use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::Notify;

pub struct OnceInit<T> {
    inner: Arc<OnceInitInner<T>>,
}
unsafe impl<T: Sync + Send> Sync for OnceInit<T> {}
unsafe impl<T: Send> Send for OnceInit<T> {}

struct OnceInitInner<T> {
    initialized: AtomicBool,
    item: UnsafeCell<MaybeUninit<T>>,
    on_initialize: Notify,
}

pub struct Setter<T> {
    inner: Arc<OnceInitInner<T>>,
}
unsafe impl<T: Send> Send for Setter<T> {}

impl<T> OnceInit<T> {
    pub fn new() -> (Self, Setter<T>) {
        let inner = Arc::new(OnceInitInner {
            initialized: AtomicBool::new(false),
            item: UnsafeCell::new(MaybeUninit::uninit()),
            on_initialize: Notify::new(),
        });
        (
            Self {
                inner: inner.clone(),
            },
            Setter { inner },
        )
    }

    fn try_get(&self) -> Option<&T> {
        self.inner.try_get()
    }

    pub async fn get(&self) -> &T {
        self.inner.get().await
    }
}

impl<T> Setter<T> {
    pub fn set(self, value: T) {
        unsafe {
            *self.inner.item.get() = MaybeUninit::new(value);
        }

        self.inner.initialized.store(true, Ordering::Relaxed);
        self.inner.on_initialize.notify_waiters();
    }
}

impl<T> OnceInitInner<T> {
    fn try_get(&self) -> Option<&T> {
        if self.initialized.load(Ordering::Relaxed) {
            unsafe { Some((*self.item.get()).assume_init_ref()) }
        } else {
            None
        }
    }

    async fn get(&self) -> &T {
        let future = self.on_initialize.notified();

        tokio::pin!(future);

        future.as_mut().enable();

        if let Some(msg) = self.try_get() {
            return msg;
        }

        future.as_mut().await;

        self.try_get().unwrap()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test() {
        let (cell, setter) = OnceInit::new();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            setter.set(10);
        });

        pollster::block_on(async {
            let n = cell.get().await;

            println!("{}", n);
        })
    }
}
