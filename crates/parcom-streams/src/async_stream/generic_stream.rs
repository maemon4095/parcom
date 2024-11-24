use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::{
        atomic::{AtomicIsize, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use parcom_core::{ParcomSegmentStream, ParcomStream};

use crate::util::Notify;

use super::StreamSource;

#[derive(Clone)]
pub struct GenericStream<S: StreamSource> {
    inner: Arc<InnerStream<S>>,
}

#[derive(Debug)]
struct InnerStream<S: StreamSource> {
    source: S,
    on_append: Notify,
    head: ArcNode<S::Node>,
}

impl<S: StreamSource> GenericStream<S> {
    pub fn new(source: S) -> Self {
        Self {
            inner: Arc::new(InnerStream {
                source,
                on_append: Notify::new(),
                head: ArcNode::new(),
            }),
        }
    }
}

struct Node<T>(ArcNode<T>);

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for Node<T> {
    type Target = NodeData<T>;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> Node<T> {
    fn get(&self) -> &NodeData<T> {
        unsafe { self.0.get_unchecked_some() }
    }
}

#[derive(Debug)]
struct ArcNode<T>(Arc<InnerNode<T>>);

impl<T> Clone for ArcNode<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> ArcNode<T> {
    fn new() -> Self {
        Self(Arc::new(InnerNode {
            state: AtomicU8::new(STATE_INITIAL),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }))
    }

    unsafe fn get_unchecked_some(&self) -> &NodeData<T> {
        (*self.0.data.get()).assume_init_ref()
    }

    unsafe fn get_unchecked(&self) -> Option<&NodeData<T>> {
        if self.0.state.load(Ordering::Acquire) == STATE_INITIALIZED_SOME {
            Some(self.get_unchecked_some())
        } else {
            None
        }
    }

    async fn ensure_init<S: StreamSource<Node = T>>(&self, stream: Arc<InnerStream<S>>) {
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

                match source.next().await {
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

    async fn get_or_init_owned<S: StreamSource<Node = T>>(
        &self,
        stream: Arc<InnerStream<S>>,
    ) -> Option<Node<T>> {
        self.ensure_init(stream).await;

        if self.0.state.load(Ordering::Acquire) == STATE_INITIALIZED_SOME {
            Some(Node(self.clone()))
        } else {
            None
        }
    }

    async fn get_or_init<S: StreamSource<Node = T>>(
        &self,
        stream: Arc<InnerStream<S>>,
    ) -> Option<&NodeData<T>> {
        self.ensure_init(stream).await;
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
struct NodeData<T> {
    segment: T,
    next: ArcNode<T>,
}

struct Segments<S: StreamSource> {
    stream: Arc<InnerStream<S>>,
    offset: usize,
    node: Option<ArcNode<S::Node>>,
}

impl<S: StreamSource> Segments<S> {
    async fn next(mut self: std::pin::Pin<&mut Self>) -> Option<Node<<S as StreamSource>::Node>> {
        let Some(node) = &self.node else {
            return None;
        };

        let node = node.get_or_init_owned(Arc::clone(&self.stream)).await;

        match node {
            Some(node) => {
                self.node = Some(node.next.clone());
                Some(node)
            }
            None => {
                self.node = None;
                None
            }
        }
    }
}

impl<S: StreamSource> futures::Stream for Segments<S> {
    type Item = S::Node;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_or_init_on_same_node_will_call_next_once() {
        #[derive(Debug, Clone)]
        struct Source(Arc<AtomicUsize>);

        impl StreamSource for Source {
            type Output = usize;
            type Node = usize;
            type Future = std::future::Ready<Option<usize>>;

            fn next_with_hint(&mut self, _: usize) -> Self::Future {
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
                        let data = head.get_or_init(inner.clone()).await;
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
            type Node = usize;
            type Future = std::future::Ready<Option<usize>>;

            fn next_with_hint(&mut self, _: usize) -> Self::Future {
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
                            let data = node.get_or_init(inner.clone()).await;
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
