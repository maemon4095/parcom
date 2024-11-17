use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, AtomicIsize, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use crate::util::{Notify, OnceInit};

use super::StreamSource;

pub struct GenericStream<S: StreamSource> {
    inner: Arc<StreamInner<S>>,
}

#[derive(Debug)]
struct StreamInner<S: StreamSource> {
    source: S,
    next_node_requested: AtomicBool,
    on_append: Notify,
    head: Node<S::Output>,
}

#[derive(Debug)]
struct Node<T>(Arc<OnceInit<NodeData<T>>>);

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Node<T> {
    async fn get_or_init<S: StreamSource<Output = T>>(
        &self,
        inner: Arc<StreamInner<S>>,
    ) -> &NodeData<T> {
        let inner_node = Arc::clone(&self.0);
        let wait_for_init = self.0.if_uninitialized({
            move || {
                let notified = inner.on_append.notified();
                if inner.next_node_requested.load(Ordering::Acquire) {
                    return futures::future::Either::Left(notified);
                }
                drop(notified);
                inner.next_node_requested.store(true, Ordering::Release);
                let mut source = inner.source.clone();
                futures::future::Either::Right(async move {
                    let s = source.next().await;
                    let next = Node(Arc::new(OnceInit::new()));
                    inner_node
                        .init(
                            match s {
                                Some(segment) => NodeData::Mid { segment, next },
                                None => NodeData::Terminal,
                            },
                            || {
                                inner.on_append.notify_all();
                                inner.next_node_requested.store(false, Ordering::Release);
                            },
                        )
                        .unwrap_or_else(|_| unreachable!());
                })
            }
        });

        if let Some(wait_for_init) = wait_for_init {
            wait_for_init.await;
        }
        self.0.get().unwrap()
    }
}

#[derive(Debug)]
enum NodeData<T> {
    Terminal,
    Mid { segment: T, next: Node<T> },
}

impl<T> Node<T> {}

impl<T> NodeData<T> {
    fn is_terminal(&self) -> bool {
        match self {
            NodeData::Terminal => true,
            _ => false,
        }
    }
}

impl<S: StreamSource> StreamInner<S> {}
