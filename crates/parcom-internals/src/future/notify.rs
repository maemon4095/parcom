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
pub struct Notify {
    version: AtomicUsize,
    wakers: Mutex<Slab<Waker>>,
}

impl Notify {
    pub fn new() -> Self {
        Self {
            version: AtomicUsize::new(0),
            wakers: Mutex::new(Slab::new()),
        }
    }

    pub fn send(&self) {
        self.version.fetch_add(1, Ordering::SeqCst);
        let mut wakers = self.wakers.lock().unwrap();
        for waker in wakers.drain() {
            waker.wake();
        }
    }

    pub fn wait(&self) -> Wait<'_> {
        let version = self.version.load(Ordering::SeqCst);
        Wait {
            version,
            signal: self,
        }
    }
}

#[derive(Debug)]
pub struct Wait<'a> {
    version: usize,
    signal: &'a Notify,
}

impl<'a> Future for Wait<'a> {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.version != self.signal.version.load(Ordering::SeqCst) {
            return Poll::Ready(());
        }

        let signal = self.signal;
        let mut wakers = signal.wakers.lock().unwrap();
        let key = wakers.insert(cx.waker().clone());

        if self.version != signal.version.load(Ordering::SeqCst) {
            wakers.remove(key);
            Poll::Ready(())
        } else {
            drop(wakers);
            Poll::Pending
        }
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
    fn test_pending_before_send() {
        let notify = Arc::new(Notify::new());

        let handle = std::thread::spawn({
            move || {
                let wait = notify.wait();
                pollster::block_on(wait)
            }
        });

        std::thread::sleep(Duration::from_secs(1));

        assert!(!handle.is_finished());
    }

    #[test]
    fn test_send() {
        let notify = Arc::new(Notify::new());
        let parallel = 100;
        let before_barrier = Arc::new(Barrier::new(parallel + 1));
        let after_barrier = Arc::new(Barrier::new(parallel + 1));

        for _ in 0..parallel {
            std::thread::spawn({
                let barrier = Arc::clone(&after_barrier);
                let before_barrier = Arc::clone(&before_barrier);
                let notify = notify.clone();

                move || {
                    let wait = notify.wait();
                    before_barrier.wait();
                    pollster::block_on(wait);
                    barrier.wait()
                }
            });
        }

        before_barrier.wait();
        notify.send();
        after_barrier.wait();
    }

    #[test]
    fn test_send_before_wait() {
        let notify = Notify::new();
        let wait = notify.wait();
        let mut wait = std::pin::pin!(wait);

        notify.send();

        let mut cx = std::task::Context::from_waker(Waker::noop());
        assert!(wait.as_mut().poll(&mut cx).is_ready());
    }

    #[test]
    fn test_send_after_wait() {
        let notify = Notify::new();
        let wait = notify.wait();
        let mut wait = std::pin::pin!(wait);

        let mut cx = std::task::Context::from_waker(Waker::noop());

        assert!(wait.as_mut().poll(&mut cx).is_pending());

        notify.send();

        assert!(wait.as_mut().poll(&mut cx).is_ready());
    }
}
