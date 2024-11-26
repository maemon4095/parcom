use std::{
    mem::MaybeUninit,
    pin::Pin,
    sync::{atomic::Ordering, Arc},
    task::Poll,
};

use super::{
    ArcNode, InnerNode, NodeData, STATE_INITIAL, STATE_INITIALIZED_NONE, STATE_INITIALIZED_SOME,
    STATE_INITIALIZING,
};
use crate::{
    async_stream::{generic_stream::InnerStream, StreamSource},
    util::Notified,
};

pub(super) enum EnsureInit<S: StreamSource> {
    Initial {
        node: Arc<InnerNode<S::Output>>,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    },
    Initializing {
        initialize: S::Future,
        node: Arc<InnerNode<S::Output>>,
        stream: Arc<InnerStream<S>>,
    },
    Waiting {
        node: Arc<InnerNode<S::Output>>,
        notified: Notified,
        stream: Arc<InnerStream<S>>,
    },
    Final,
}

impl<S: StreamSource> EnsureInit<S> {
    pub(super) fn new(
        node: Arc<InnerNode<S::Output>>,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    ) -> Self {
        Self::Initial {
            node,
            size_hint,
            stream,
        }
    }
}

impl<S: StreamSource> std::future::Future for EnsureInit<S> {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let me = unsafe { self.get_unchecked_mut() };
        match me {
            EnsureInit::Initial {
                node,
                size_hint,
                stream,
            } => {
                let result = node.state.compare_exchange(
                    STATE_INITIAL,
                    STATE_INITIALIZING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );

                match result {
                    Ok(_) => {
                        let mut source = stream.source.clone();

                        *me = Self::Initializing {
                            initialize: source.next(*size_hint),
                            node: Arc::clone(&node),
                            stream: Arc::clone(&stream),
                        };

                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(STATE_INITIALIZING) => {
                        let notified = stream.on_append.notified();
                        let state = node.state.load(Ordering::Acquire);

                        match state {
                            STATE_INITIALIZED_NONE | STATE_INITIALIZED_SOME => {
                                drop(notified);
                                *me = Self::Final;
                                Poll::Ready(())
                            }
                            _ => {
                                *me = EnsureInit::Waiting {
                                    node: Arc::clone(&node),
                                    stream: Arc::clone(&stream),
                                    notified,
                                };

                                Poll::Pending
                            }
                        }
                    }
                    Err(STATE_INITIALIZED_NONE) | Err(STATE_INITIALIZED_SOME) => Poll::Ready(()),
                    Err(_) => unreachable!(),
                }
            }
            EnsureInit::Waiting {
                node,
                notified,
                stream,
            } => match Pin::new(&mut *notified).poll(cx) {
                Poll::Ready(_) => {
                    let new_notified = stream.on_append.notified();
                    let state = node.state.load(Ordering::Acquire);

                    match state {
                        STATE_INITIALIZED_NONE | STATE_INITIALIZED_SOME => {
                            drop(new_notified);
                            *me = Self::Final;
                            Poll::Ready(())
                        }
                        _ => {
                            *notified = new_notified;
                            Poll::Pending
                        }
                    }
                }
                Poll::Pending => Poll::Pending,
            },
            EnsureInit::Initializing {
                initialize,
                node,
                stream,
            } => {
                let initialize = unsafe { Pin::new_unchecked(initialize) };
                match initialize.poll(cx) {
                    Poll::Ready(v) => {
                        match v {
                            Some(segment) => {
                                let data = NodeData {
                                    segment,
                                    next: ArcNode::new(),
                                };
                                unsafe {
                                    *node.data.get() = MaybeUninit::new(data);
                                }
                                node.state.store(STATE_INITIALIZED_SOME, Ordering::Release);
                            }
                            None => {
                                node.state.store(STATE_INITIALIZED_NONE, Ordering::Release);
                            }
                        }
                        stream.on_append.notify_all();
                        *me = Self::Final;
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            EnsureInit::Final => {
                panic!("`poll` should not be called on resolved EnsureInit")
            }
        }
    }
}
