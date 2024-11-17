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

    pub fn notify_first(&self) {
        let inner = &self.inner;
        let mut lock = inner.wakers.lock().unwrap();
        inner.increment_version();
        for waker in lock.drain().take(1) {
            waker.wake();
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

    fn register_waker(&self, waker: &Waker) -> usize {
        let mut lock = self.wakers.lock().unwrap();
        if lock.len() >= WAKER_COUNT_MAX {
            panic!("Too many waiters on the notify.")
        }
        lock.insert(waker.clone())
    }

    fn remove_waker(&self, waker_id: usize) {
        let mut lock = self.wakers.lock().unwrap();
        lock.try_remove(waker_id);
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
            Poll::Ready(())
        } else {
            let id = self.notify.register_waker(cx.waker());
            self.waker_id = Some(id);
            Poll::Pending
        }
    }
}
impl Drop for Notified {
    fn drop(&mut self) {
        if let Some(id) = self.waker_id {
            self.notify.remove_waker(id);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Countdown {
        notify: tokio::sync::Notify,
        count: AtomicUsize,
    }

    unsafe impl Send for Countdown {}
    unsafe impl Sync for Countdown {}

    impl Countdown {
        fn new(count: usize) -> Self {
            Self {
                notify: tokio::sync::Notify::new(),
                count: AtomicUsize::new(count),
            }
        }

        fn countdown(&self) {
            let current = self.count.load(Ordering::Relaxed);
            if current == 0 {
                return;
            }

            let result = self.count.compare_exchange(
                current,
                current - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );

            if Ok(1) == result {
                self.notify.notify_waiters();
            }
        }

        async fn wait(&self) {
            self.notify.notified().await
        }
    }

    #[tokio::test]
    async fn test_all_waiters_notified_on_notify_all() {
        let notify = Notify::new();
        let waiter_count = 0xFF;
        let ready = Arc::new(Countdown::new(waiter_count));
        let mut handles = Vec::new();

        for _ in 0..waiter_count {
            let handle = tokio::spawn({
                let ready = Arc::clone(&ready);
                let notify = notify.clone();
                async move {
                    let notified = notify.notified();
                    ready.countdown();
                    notified.await;
                }
            });

            handles.push(handle);
        }

        ready.wait().await;
        notify.notify_all();

        futures::future::join_all(handles).await;
    }

    #[tokio::test]
    async fn test_the_first_waiter_notified_on_notify_first() {
        let notify = Notify::new();
        let waiter_count = 0xFF;
        let ready = Arc::new(Countdown::new(waiter_count));
        let mut handles = Vec::new();
        let notified_waiters = Arc::new(Mutex::new(Vec::new()));

        for id in 0..waiter_count {
            let handle = tokio::spawn({
                let ready = Arc::clone(&ready);
                let notify = notify.clone();
                let notified_waiters = Arc::clone(&notified_waiters);
                async move {
                    let notified = notify.notified();
                    ready.countdown();
                    notified.await;
                    notified_waiters.lock().unwrap().push(id);
                }
            });

            handles.push(handle);
        }

        ready.wait().await;
        notify.notify_first();

        let _ = futures::future::select_all(handles).await;

        {
            // assert the first waiter was notified
            let lock = notified_waiters.lock().unwrap();
            assert_eq!(lock.len(), 1);
            assert_eq!(lock[0], 0);
        }

        {
            // assert all wakers were removed
            let lock = notify.inner.wakers.lock().unwrap();
            assert_eq!(lock.len(), 0);
        }
    }
}
