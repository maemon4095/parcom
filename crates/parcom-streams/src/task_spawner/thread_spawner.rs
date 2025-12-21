use std::{
    future::{Future, IntoFuture},
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use super::TaskSpawner;

#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadSpawner;

pub struct ThreadSpawnerHandle<T> {
    waker: Arc<Mutex<Option<Waker>>>,
    handle: std::thread::JoinHandle<T>,
}

pub struct ThreadSpawnerHandleFuture<T> {
    waker: Arc<Mutex<Option<Waker>>>,
    handle: Option<std::thread::JoinHandle<T>>,
}

impl<T> Future for ThreadSpawnerHandleFuture<T> {
    type Output = Result<T, Box<dyn std::any::Any + Send + 'static>>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let handle = self.handle.as_ref().unwrap();
        if handle.is_finished() {
            let handle = self.handle.take().unwrap();
            return Poll::Ready(handle.join());
        }

        let waker = Arc::clone(&self.waker);
        let mut guard = waker.lock().unwrap();
        if handle.is_finished() {
            let handle = self.handle.take().unwrap();
            return Poll::Ready(handle.join());
        }
        *guard = Some(cx.waker().clone());

        Poll::Pending
    }
}

impl<T> IntoFuture for ThreadSpawnerHandle<T> {
    type Output = Result<T, Box<dyn std::any::Any + Send + 'static>>;
    type IntoFuture = ThreadSpawnerHandleFuture<T>;

    fn into_future(self) -> Self::IntoFuture {
        ThreadSpawnerHandleFuture {
            waker: self.waker,
            handle: Some(self.handle),
        }
    }
}

impl TaskSpawner for ThreadSpawner {
    type Error = Box<dyn std::any::Any + Send + 'static>;
    type Handle<T: Send> = ThreadSpawnerHandle<T>;

    fn spawn<T, F>(&self, task: F) -> Self::Handle<T>
    where
        T: 'static + Send,
        F: 'static + Send + IntoFuture<Output = T>,
    {
        let waker = Arc::new(Mutex::new(None::<Waker>));
        let handle = std::thread::spawn({
            let waker = Arc::clone(&waker);
            move || {
                let value = pollster::block_on(task.into_future());
                let mut guard = waker.lock().unwrap();
                if let Some(waker) = guard.take() {
                    waker.wake();
                }
                value
            }
        });

        ThreadSpawnerHandle { waker, handle }
    }
}
