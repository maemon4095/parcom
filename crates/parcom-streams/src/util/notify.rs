use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::{Poll, Waker},
};

use slab::Slab;

#[derive(Debug, Clone)]
pub struct Notify {
    inner: Arc<NotifyInner>,
}

#[derive(Debug)]
struct NotifyInner {
    wakers: Mutex<Slab<Waker>>,
    version: AtomicUsize,
}

unsafe impl Send for NotifyInner {}
unsafe impl Sync for NotifyInner {}

const WAKER_COUNT_MAX: usize = usize::MAX / 2;

// 0から時計周りに数字をならべたとき、baseに対して反時計周りの半分をbaseより小さい値、時計回りの半分をbaseより大きい値として比較する。
// usizeの取りうる値のパターンは偶数であるため、完全に正対する位置の大小は定まらない。
fn cycle_compare(comparand: usize, base: usize) -> Option<std::cmp::Ordering> {
    const MID: usize = (usize::MAX / 2) + 1;
    let (c, _) = comparand.overflowing_sub(base);
    if c == MID {
        return None;
    }
    if c == 0 {
        return Some(std::cmp::Ordering::Equal);
    }
    if c > MID {
        Some(std::cmp::Ordering::Less)
    } else {
        Some(std::cmp::Ordering::Greater)
    }
}

impl Notify {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NotifyInner {
                wakers: Mutex::new(Slab::new()),
                version: AtomicUsize::new(0),
            }),
        }
    }

    pub fn notify_all(&self) {
        let inner = &self.inner;
        let mut lock = inner.wakers.lock().unwrap();
        inner.increment_version();
        for waker in lock.drain() {
            waker.wake();
        }
    }

    pub fn notified(&self) -> Notified {
        Notified {
            notify: Arc::clone(&self.inner),
            version: self.inner.get_version(),
            waker_id: None,
        }
    }
}

impl NotifyInner {
    fn increment_version(&self) {
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    fn get_version(&self) -> usize {
        self.version.load(Ordering::Relaxed)
    }
}

pub struct Notified {
    notify: Arc<NotifyInner>,
    version: usize,
    waker_id: Option<usize>,
}

impl Future for Notified {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if cycle_compare(self.notify.get_version(), self.version)
            .unwrap()
            .is_gt()
        {
            self.waker_id = None;
            return Poll::Ready(());
        }

        let Self {
            notify,
            version,
            waker_id,
        } = self.get_mut();
        // *1
        let mut lock = notify.wakers.lock().unwrap();
        if lock.len() >= WAKER_COUNT_MAX {
            panic!("Too many waiters on the notify.");
        }

        let id = lock.insert(cx.waker().clone());
        *waker_id = Some(id);

        // *1でlockを獲得する前にnotifyが呼ばれる場合がある。その場合にはwakerが呼ばれないためチェックする。
        if cycle_compare(notify.get_version(), *version)
            .unwrap()
            .is_gt()
        {
            lock.remove(id);
            *waker_id = None;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for Notified {
    fn drop(&mut self) {
        if let Some(id) = self.waker_id {
            let mut lock = self.notify.wakers.lock().unwrap();
            lock.try_remove(id);
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Barrier;

    use super::*;

    #[test]
    fn test_all_waiters_notified_on_notify_all() {
        for _ in 0..0xFF {
            test_once();
        }

        fn test_once() {
            let notify = Notify::new();
            let waiter_count = 0xFF;
            let mut handles = Vec::new();
            let barrier = Arc::new(Barrier::new(waiter_count + 1));

            for _ in 0..waiter_count {
                let notify = notify.clone();
                let barrier = Arc::clone(&barrier);

                let handle = std::thread::spawn(move || {
                    pollster::block_on(async move {
                        let notified = notify.notified();
                        barrier.wait();
                        notified.await
                    })
                });

                handles.push(handle);
            }

            barrier.wait();
            notify.notify_all();

            for h in handles {
                h.join().unwrap();
            }
        }
    }
}
