use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, LazyLock};
use std::sync::{Mutex, MutexGuard};

pub static TRACE_STORE: LazyLock<TraceStore> = LazyLock::new(|| TraceStore {
    lock: Mutex::new(()),
    allocs: AtomicUsize::new(0),
    drops: AtomicUsize::new(0),
});

pub struct TraceStore {
    lock: Mutex<()>,
    allocs: AtomicUsize,
    drops: AtomicUsize,
}

impl TraceStore {
    pub fn start_tracing(&'static self) -> TraceStoreTracing {
        let guard = match self.lock.lock() {
            Ok(v) => v,
            Err(v) => v.into_inner(),
        };

        self.allocs.store(0, Ordering::Relaxed);
        self.drops.store(0, Ordering::Relaxed);

        TraceStoreTracing {
            this: self,
            _guard: guard,
        }
    }
}

pub struct TraceStoreTracing {
    this: &'static TraceStore,
    _guard: MutexGuard<'static, ()>,
}

impl TraceStoreTracing {
    pub fn allocs(&self) -> usize {
        self.this.allocs.load(Ordering::Relaxed)
    }

    pub fn drops(&self) -> usize {
        self.this.drops.load(Ordering::Relaxed)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
#[repr(transparent)]
pub struct TraceCell<T>(T);

impl<T: Clone> Clone for TraceCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<T> From<T> for TraceCell<T> {
    fn from(value: T) -> Self {
        TraceCell::new(value)
    }
}

impl<T> TraceCell<T> {
    pub fn new(value: T) -> Self {
        TRACE_STORE.allocs.fetch_add(1, Ordering::Relaxed);
        Self(value)
    }
}

impl<T> Drop for TraceCell<T> {
    fn drop(&mut self) {
        TRACE_STORE.drops.fetch_add(1, Ordering::Relaxed);
    }
}

impl<T> std::ops::Deref for TraceCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for TraceCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
