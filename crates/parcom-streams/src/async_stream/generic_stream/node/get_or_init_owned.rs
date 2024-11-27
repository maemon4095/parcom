use std::{
    future::Future,
    mem::ManuallyDrop,
    ops::DerefMut,
    pin::Pin,
    sync::{atomic::Ordering, Arc},
    task::Poll,
};

use super::{ensure_init::EnsureInit, ArcNode, InnerNode, Node, STATE_INITIALIZED_SOME};
use crate::async_stream::{generic_stream::InnerStream, StreamSource};

pub struct GetOrInitOwned<S: StreamSource> {
    state: State<S>,
}

enum State<S: StreamSource> {
    Initial {
        node: Arc<InnerNode<S::Output>>,
        ensure_init: ManuallyDrop<EnsureInit<S>>,
    },
    Final,
}

impl<S: StreamSource> GetOrInitOwned<S> {
    pub(super) fn new(
        node: Arc<InnerNode<S::Output>>,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) -> Self {
        Self {
            state: State::Initial {
                node: Arc::clone(&node),
                ensure_init: ManuallyDrop::new(EnsureInit::new(node, size_hint, stream)),
            },
        }
    }
}

impl<S: StreamSource> Future for GetOrInitOwned<S> {
    type Output = Option<Node<S::Output>>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let me = unsafe { &mut self.get_unchecked_mut().state };
        match me {
            State::Initial { ensure_init, .. } => unsafe {
                let pinned_ensure_init = Pin::new_unchecked(&mut *ensure_init.deref_mut());
                match pinned_ensure_init.poll(cx) {
                    Poll::Ready(_) => {
                        ManuallyDrop::drop(ensure_init);
                        let State::Initial { node, .. } = std::mem::replace(me, State::Final)
                        else {
                            unreachable!()
                        };
                        if node.state.load(Ordering::Acquire) == STATE_INITIALIZED_SOME {
                            Poll::Ready(Some(Node(ArcNode(node))))
                        } else {
                            Poll::Ready(None)
                        }
                    }
                    Poll::Pending => Poll::Pending,
                }
            },
            State::Final => {
                panic!("`poll` should not be called on resolved GetOrInitOwned")
            }
        }
    }
}
