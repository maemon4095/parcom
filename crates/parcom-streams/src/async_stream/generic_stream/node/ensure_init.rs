use std::{
    borrow::BorrowMut,
    future::Future,
    mem::{ManuallyDrop, MaybeUninit},
    ops::DerefMut,
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

pub struct EnsureInit<S: StreamSource> {
    state: State<S>,
}

enum State<S: StreamSource> {
    Initial {
        node: Arc<InnerNode<S::Output>>,
        size_hint: usize,
        stream: Arc<InnerStream<S>>,
    },
    Initializing {
        next: ManuallyDrop<S::Future>,
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
        Self {
            state: State::Initial {
                node,
                size_hint,
                stream,
            },
        }
    }
}

impl<S: StreamSource> Future for EnsureInit<S> {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let me = unsafe { &mut self.get_unchecked_mut().state };
        match me {
            State::Initial { node, .. } => {
                let result = node.state.compare_exchange(
                    STATE_INITIAL,
                    STATE_INITIALIZING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );

                let State::Initial {
                    node,
                    stream,
                    size_hint,
                } = std::mem::replace(me, State::Final)
                else {
                    unreachable!()
                };
                match result {
                    Ok(_) => {
                        let mut source = stream.source.clone();

                        *me = State::Initializing {
                            next: ManuallyDrop::new(source.next(size_hint)),
                            node,
                            stream,
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
                                Poll::Ready(())
                            }
                            _ => {
                                *me = State::Waiting {
                                    node,
                                    stream,
                                    notified,
                                };
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            }
                        }
                    }
                    Err(STATE_INITIALIZED_NONE) | Err(STATE_INITIALIZED_SOME) => Poll::Ready(()),
                    Err(_) => unreachable!(),
                }
            }
            State::Waiting {
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
                            *me = State::Final;
                            Poll::Ready(())
                        }
                        _ => {
                            *notified = new_notified;
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    }
                }
                Poll::Pending => Poll::Pending,
            },
            State::Initializing { next, node, stream } => {
                let pinned_next = unsafe { Pin::new_unchecked(next.deref_mut()) };
                match pinned_next.poll(cx) {
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
                        unsafe {
                            ManuallyDrop::drop(next);
                        }
                        *me = State::Final;
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            State::Final => {
                panic!("`poll` should not be called on resolved EnsureInit")
            }
        }
    }
}
