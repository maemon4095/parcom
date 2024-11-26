mod ensure_init;

use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    pin::Pin,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    task::Poll,
};

use futures::FutureExt;

use crate::util::Notified;

use super::{InnerStream, StreamSource};

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

    async fn ensure_init<S: StreamSource<Output = T>>(
        &self,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) {
        let result = self.0.state.compare_exchange(
            STATE_INITIAL,
            STATE_INITIALIZING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        match result {
            Ok(_) => {
                let mut source = stream.source.clone();
                let on_append = stream.on_append.clone();

                match source.next(size_hint).await {
                    Some(segment) => {
                        let node = NodeData {
                            segment,
                            next: ArcNode::new(),
                        };
                        unsafe {
                            *self.0.data.get() = MaybeUninit::new(node);
                        }
                        self.0
                            .state
                            .store(STATE_INITIALIZED_SOME, Ordering::Release);
                    }
                    None => {
                        self.0
                            .state
                            .store(STATE_INITIALIZED_NONE, Ordering::Release);
                    }
                }

                on_append.notify_all();
            }
            Err(STATE_INITIALIZING) => loop {
                let notified = stream.on_append.notified();
                let state = self.0.state.load(Ordering::Acquire);

                match state {
                    STATE_INITIALIZED_NONE | STATE_INITIALIZED_SOME => {
                        drop(notified);
                        break;
                    }
                    _ => (),
                }

                notified.await;
            },
            Err(STATE_INITIALIZED_NONE) => (),
            Err(STATE_INITIALIZED_SOME) => (),
            Err(_) => unreachable!(),
        }
    }

    pub(super) async fn get_or_init_owned<S: StreamSource<Output = T>>(
        &self,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) -> Option<Node<T>> {
        self.ensure_init(size_hint, stream).await;

        if self.0.state.load(Ordering::Acquire) == STATE_INITIALIZED_SOME {
            Some(Node(self.clone()))
        } else {
            None
        }
    }

    pub(super) async fn get_or_init<S: StreamSource<Output = T>>(
        &self,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) -> Option<&NodeData<T>> {
        self.ensure_init(size_hint, stream).await;
        unsafe { self.get_unchecked() }
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
                        let data = head.get_or_init(0, inner.clone()).await;
                        match data {
                            None => unreachable!(),
                            Some(data) => {
                                assert_eq!(data.segment, 0);
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

            let iterate_count = 0xFFF;
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
                            let data = node.get_or_init(0, inner.clone()).await;
                            match data {
                                None => unreachable!(),
                                Some(NodeData { next, segment }) => {
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
