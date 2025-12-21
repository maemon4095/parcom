use slab::Slab;
use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    task::{Poll, Waker},
};

#[derive(Debug)]
pub struct Signal {
    version: AtomicUsize,
    wakers: Mutex<Slab<Waker>>,
}

impl Signal {
    pub fn new() -> Self {
        Self {
            version: AtomicUsize::new(0),
            wakers: Mutex::new(Slab::new()),
        }
    }

    pub fn send(&self) {
        self.version.fetch_add(1, Ordering::Relaxed);
        let mut wakers = self.wakers.lock().unwrap();
        for waker in wakers.drain() {
            waker.wake();
        }
    }
}

#[derive(Debug)]
pub struct Wait<T: Unpin + std::ops::Deref<Target = Signal>> {
    key: Option<usize>,
    version: usize,
    container: T,
}

impl<T: Unpin + std::ops::Deref<Target = Signal>> Wait<T> {
    pub fn new(container: T) -> Self {
        let version = container.version.load(Ordering::Relaxed);
        Self {
            key: None,
            version,
            container,
        }
    }
}

impl<T: Unpin + std::ops::Deref<Target = Signal>> Future for Wait<T> {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.version != self.container.version.load(Ordering::Relaxed) {
            return Poll::Ready(());
        }

        let old_key = self.key.take();
        let signal = &*self.container;
        let mut wakers = signal.wakers.lock().unwrap();

        let key = match old_key {
            Some(key) => match wakers.get_mut(key) {
                Some(w) => {
                    w.clone_from(cx.waker());
                    key
                }
                None => wakers.insert(cx.waker().clone()),
            },
            None => wakers.insert(cx.waker().clone()),
        };

        if self.version != signal.version.load(Ordering::Relaxed) {
            wakers.remove(key);
            Poll::Ready(())
        } else {
            drop(wakers);
            self.key = Some(key);
            Poll::Pending
        }
    }
}

impl<T: Unpin + std::ops::Deref<Target = Signal>> Drop for Wait<T> {
    fn drop(&mut self) {
        let Some(key) = self.key.take() else {
            return;
        };

        let signal = &*self.container;
        if self.version != signal.version.load(Ordering::Relaxed) {
            return;
        }

        let mut wakers = signal.wakers.lock().unwrap();
        if self.version != signal.version.load(Ordering::Relaxed) {
            return;
        }
        wakers.try_remove(key);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{
        sync::{Arc, Barrier},
        time::Duration,
    };

    #[test]
    fn test_pending_before_notify() {
        let notify = Arc::new(Signal::new());

        let handle = std::thread::spawn({
            let wait = Wait::new(notify);
            move || pollster::block_on(wait)
        });

        std::thread::sleep(Duration::from_secs(1));

        assert!(!handle.is_finished());
    }

    #[test]
    fn test_notified() {
        let notify = Arc::new(Signal::new());
        let parallel = 100;
        let barrier = Arc::new(Barrier::new(parallel + 1));

        let handle = std::thread::spawn({
            let barrier = Arc::clone(&barrier);
            move || barrier.wait()
        });

        for _ in 0..parallel {
            std::thread::spawn({
                let barrier = Arc::clone(&barrier);
                let wait = Wait::new(notify.clone());
                move || {
                    pollster::block_on(wait);
                    barrier.wait()
                }
            });
        }

        notify.send();

        std::thread::sleep(Duration::from_secs(1));

        assert!(handle.is_finished());
    }

    #[test]
    fn test_notified_before_wait() {
        let notify = Signal::new();
        let wait = Wait::new(&notify);

        notify.send();

        pollster::block_on(wait)
    }
}
