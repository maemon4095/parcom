mod ensure_init;
mod get_or_init_owned;

use super::{InnerStream, StreamSource};
pub(super) use ensure_init::EnsureInit;
pub(super) use get_or_init_owned::GetOrInitOwned;
use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

pub struct Node<T>(ArcNode<T>);

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.get().segment
    }
}

impl<T> Node<T> {
    fn get(&self) -> &NodeData<T> {
        unsafe { self.0.get_unchecked_some() }
    }
}

#[derive(Debug)]
pub(super) struct ArcNode<T>(Arc<InnerNode<T>>);

impl<T> Clone for ArcNode<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> ArcNode<T> {
    pub(super) fn new() -> Self {
        Self(Arc::new(InnerNode {
            state: AtomicU8::new(STATE_INITIAL),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }))
    }

    pub(super) unsafe fn get_unchecked_some(&self) -> &NodeData<T> {
        (*self.0.data.get()).assume_init_ref()
    }

    pub(super) unsafe fn get_unchecked(&self) -> Option<&NodeData<T>> {
        if self.0.state.load(Ordering::Acquire) == STATE_INITIALIZED_SOME {
            Some(self.get_unchecked_some())
        } else {
            None
        }
    }

    fn ensure_init<S: StreamSource<Output = T>>(
        &self,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) -> EnsureInit<S> {
        EnsureInit::new(Arc::clone(&self.0), size_hint, stream)
    }

    pub(super) fn get_or_init_owned<S: StreamSource<Output = T>>(
        &self,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) -> GetOrInitOwned<S> {
        GetOrInitOwned::new(Arc::clone(&self.0), size_hint, stream)
    }
}

const STATE_INITIAL: u8 = 0;
const STATE_INITIALIZING: u8 = 1;
const STATE_INITIALIZED_NONE: u8 = 2;
const STATE_INITIALIZED_SOME: u8 = 3;

#[derive(Debug)]
struct InnerNode<T> {
    state: AtomicU8,
    data: UnsafeCell<MaybeUninit<NodeData<T>>>,
}

impl<T> Drop for InnerNode<T> {
    fn drop(&mut self) {
        if self.state.load(Ordering::Acquire) != STATE_INITIALIZED_SOME {
            return;
        }
        let mut current = unsafe { self.data.get_mut().assume_init_read() };

        loop {
            let NodeData { segment, next } = current;
            drop(segment);

            let Ok(mut next) = Arc::try_unwrap(next.0) else {
                break;
            };

            if next.state.load(Ordering::Acquire) != STATE_INITIALIZED_SOME {
                break;
            }

            current = unsafe { next.data.get_mut().assume_init_read() };
            std::mem::forget(next);
        }
    }
}

unsafe impl<T: Send> Send for InnerNode<T> {}
unsafe impl<T: Sync> Sync for InnerNode<T> {}

#[derive(Debug)]
pub(super) struct NodeData<T> {
    pub(super) segment: T,
    pub(super) next: ArcNode<T>,
}

#[cfg(test)]
mod test {
    use std::sync::atomic::AtomicUsize;

    use super::super::GenericStream;
    use super::*;

    #[test]
    fn test_get_or_init_on_same_node_will_call_next_once() {
        #[derive(Debug, Clone)]
        struct Source(Arc<AtomicUsize>);

        impl StreamSource for Source {
            type Output = usize;
            type Future = std::future::Ready<Option<usize>>;

            fn next(&mut self, _: usize) -> Self::Future {
                let n = self.0.fetch_add(1, Ordering::Relaxed);

                assert_eq!(n, 0); // assert next called once.

                std::future::ready(Some(n))
            }
        }

        for _ in 0..0xFF {
            test_once();
        }

        fn test_once() {
            let stream = GenericStream::new(Source(Arc::new(AtomicUsize::new(0))));

            let parallel = 0xFF;
            let barrier = Arc::new(std::sync::Barrier::new(parallel));

            let mut handles = Vec::new();

            for _ in 0..parallel {
                let inner = Arc::clone(&stream.inner);
                let head = inner.head.clone();
                let barrier = Arc::clone(&barrier);
                let handle = std::thread::spawn(move || {
                    barrier.wait();
                    pollster::block_on(async move {
                        let data = head.get_or_init_owned(0, inner.clone()).await;
                        match data {
                            None => unreachable!(),
                            Some(n) => {
                                let NodeData { segment, .. } = n.get();
                                assert_eq!(*segment, 0);
                            }
                        }
                    })
                });

                handles.push(handle);
            }

            for h in handles {
                h.join().unwrap();
            }
        }
    }

    #[test]
    fn test_iterate() {
        #[derive(Debug, Clone)]
        struct Source(Arc<AtomicUsize>);

        impl StreamSource for Source {
            type Output = usize;
            type Future = std::future::Ready<Option<usize>>;

            fn next(&mut self, _: usize) -> Self::Future {
                let n = self.0.fetch_add(1, Ordering::Relaxed);
                std::future::ready(Some(n))
            }
        }

        for _ in 0..0xFF {
            test_once();
        }

        fn test_once() {
            let stream = GenericStream::new(Source(Arc::new(AtomicUsize::new(0))));

            let iterate_count = 1024;
            let parallel = 0xFF;
            let barrier = Arc::new(std::sync::Barrier::new(parallel));

            let mut handles = Vec::new();

            for _ in 0..parallel {
                let inner = Arc::clone(&stream.inner);
                let head = inner.head.clone();
                let barrier = Arc::clone(&barrier);
                let handle = std::thread::spawn(move || {
                    barrier.wait();
                    pollster::block_on(async move {
                        let mut node = head;

                        for expected in 0..iterate_count {
                            let data = node.get_or_init_owned(0, inner.clone()).await;
                            match data {
                                None => unreachable!(),
                                Some(n) => {
                                    let NodeData { segment, next } = n.get();
                                    assert_eq!(*segment, expected);
                                    node = next.clone();
                                }
                            }
                        }
                    })
                });

                handles.push(handle);
            }

            for h in handles {
                h.join().unwrap();
            }

            // next called `iterate_count` time
            assert_eq!(stream.inner.source.0.load(Ordering::Relaxed), iterate_count);

            let mut node = stream.inner.head.clone();

            for expected in 0..iterate_count {
                assert_eq!(node.0.state.load(Ordering::Relaxed), STATE_INITIALIZED_SOME);
                let data = unsafe { node.get_unchecked_some() };
                let NodeData { segment, next } = data;
                assert_eq!(*segment, expected);

                node = next.clone();
            }

            // assert last node is not initialized
            assert_eq!(node.0.state.load(Ordering::Relaxed), STATE_INITIAL);
        }
    }
}
