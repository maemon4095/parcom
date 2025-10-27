use super::{
    control::Control, control::Response, GenericStream, HeadTailNodePair, InnerGenericStream,
    IntermediateNode, Node, NodePtr,
};
use parcom_streams_core::StreamSource;
use pin_project::pin_project;
use std::{cell::UnsafeCell, future::Future, pin::Pin, sync::Arc, task::Poll};

#[pin_project(!Unpin)]
pub struct Advance<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    #[pin]
    state: AdvanceState<T, S>,
}

impl<T, S> Advance<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    pub(super) fn new(stream: Box<InnerGenericStream<T, S>>, delta: usize) -> Self {
        Self {
            state: AdvanceState::Initial {
                stream: Some(stream),
                delta,
            },
        }
    }
}

#[pin_project(project = AdvanceStateProj)]
enum AdvanceState<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    Initial {
        stream: Option<Box<InnerGenericStream<T, S>>>,
        delta: usize,
    },
    Loading {
        #[pin]
        fut: Pin<Box<dyn Future<Output = Result<GenericStream<T, S>, S::Error>>>>,
    },
}

impl<T, S> Future for Advance<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    type Output = Result<GenericStream<T, S>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.project().state;
        match state.as_mut().project() {
            AdvanceStateProj::Initial { stream, delta } => {
                let Some(stream) = stream.take() else {
                    panic!("`poll` after completed")
                };

                match advance_already_loaded(stream, *delta) {
                    Ok(v) => Poll::Ready(Ok(v)),
                    Err((stream, remain)) => {
                        let fut = advance_load(stream, remain);
                        state.set(AdvanceState::Loading { fut: Box::pin(fut) });
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            AdvanceStateProj::Loading { fut } => fut.poll(cx),
        }
    }
}

fn advance_already_loaded<T, S>(
    mut stream: Box<InnerGenericStream<T, S>>,
    remain: usize,
) -> Result<GenericStream<T, S>, (Box<InnerGenericStream<T, S>>, usize)>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    if remain == 0 {
        return Ok(GenericStream { inner: stream });
    }

    let Some(pair) = stream.pair.take() else {
        return Err((stream, remain));
    };

    let HeadTailNodePair {
        head_offset,
        tail_len,
        head_ptr,
        tail_ptr,
    } = pair;

    let head = unsafe { &*head_ptr.raw.get() };
    let head = match head {
        Node::Intermediate(v) => v,
        Node::Sentinel => panic!("`delta` out of stream"),
    };

    let Some(head_next) = head.next.clone() else {
        // head is tail
        debug_assert!(Arc::ptr_eq(&head_ptr.raw, &tail_ptr.raw));

        let head_len = tail_len - head_offset;
        return if remain <= head_len {
            stream.pair = Some(HeadTailNodePair {
                head_offset: head_offset + remain,
                tail_len,
                head_ptr,
                tail_ptr,
            });
            Ok(GenericStream { inner: stream })
        } else {
            stream.pair = Some(HeadTailNodePair {
                head_offset: tail_len,
                tail_len,
                head_ptr,
                tail_ptr,
            });
            Err((stream, remain - head_len))
        };
    };

    let head_len = head.buf.len() - head_offset;

    if remain <= head_len {
        stream.pair = Some(HeadTailNodePair {
            head_offset: head_offset + remain,
            tail_len,
            head_ptr,
            tail_ptr,
        });
        return Ok(GenericStream { inner: stream });
    }
    let mut remain = remain - head_len;
    let mut current_ptr = head_next;

    loop {
        let current = unsafe { &*current_ptr.raw.get() };
        let current = match current {
            Node::Intermediate(v) => v,
            Node::Sentinel => {
                panic!("`delta` out of stream")
            }
        };

        let Some(next_ptr) = current.next.clone() else {
            // current is tail
            debug_assert!(Arc::ptr_eq(&current_ptr.raw, &tail_ptr.raw));

            let current_len = tail_len;
            return if remain <= current_len {
                stream.pair = Some(HeadTailNodePair {
                    head_offset: remain,
                    tail_len,
                    head_ptr: current_ptr,
                    tail_ptr,
                });
                Ok(GenericStream { inner: stream })
            } else {
                stream.pair = Some(HeadTailNodePair {
                    head_offset: current_len,
                    tail_len,
                    head_ptr: current_ptr,
                    tail_ptr,
                });
                Err((stream, remain - current_len))
            };
        };

        let current_len = current.buf.len();

        if remain <= current_len {
            stream.pair = Some(HeadTailNodePair {
                head_offset: remain,
                tail_len,
                head_ptr: current_ptr,
                tail_ptr,
            });

            return Ok(GenericStream { inner: stream });
        }

        remain -= current_len;
        current_ptr = next_ptr;
    }
}

async fn advance_load<T, S>(
    mut stream: Box<InnerGenericStream<T, S>>,
    remain: usize,
) -> Result<GenericStream<T, S>, S::Error>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    debug_assert!(remain > 0);

    let (pair, remain) = match stream.pair.take() {
        Some(v) => (v, remain),
        None => match advance_load_initial(stream, remain).await? {
            Ok(v) => return Ok(v),
            Err((s, remain)) => {
                stream = s;
                let pair = stream.pair.take().unwrap();
                (pair, remain)
            }
        },
    };

    let HeadTailNodePair {
        head_offset,
        mut tail_len,
        head_ptr,
        mut tail_ptr,
    } = pair;

    debug_assert!(Arc::ptr_eq(&head_ptr.raw, &tail_ptr.raw));
    debug_assert_eq!(head_offset, tail_len);

    loop {
        let tail = unsafe { &mut *tail_ptr.raw.get() };
        let Node::Intermediate(tail) = tail else {
            panic!("`delta` out of stream")
        };

        debug_assert!(tail.is_tail());

        let tail_space = &mut tail.buf[tail_len..];
        let control = Control::new(&stream.parameter, tail_space);
        let res = stream.source.next(control, remain).await;

        match res {
            Response::PreAllocated(written) => {
                if remain <= written {
                    let head_offset = tail_len + remain;
                    tail_len += written;

                    stream.pair = Some(HeadTailNodePair {
                        head_offset,
                        tail_len,
                        head_ptr: tail_ptr.clone(),
                        tail_ptr,
                    });
                    return Ok(GenericStream { inner: stream });
                }

                tail_len += written;
            }
            Response::Allocated(items, written) => {
                let next = IntermediateNode {
                    buf: items,
                    next: None,
                };
                let next_ptr = NodePtr {
                    raw: Arc::new(UnsafeCell::new(Node::Intermediate(next))),
                };

                // 中間ノードのバッファを縮める。
                tail.buf.drain(tail_len..);
                tail.next = Some(next_ptr.clone());

                tail_len = written;
                tail_ptr = next_ptr;

                if remain <= written {
                    let head_offset = remain;
                    stream.pair = Some(HeadTailNodePair {
                        head_offset,
                        tail_len,
                        head_ptr: tail_ptr.clone(),
                        tail_ptr,
                    });
                    return Ok(GenericStream { inner: stream });
                }
            }
            Response::Finish => {
                let next_ptr = NodePtr {
                    raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                };
                tail.next = Some(next_ptr.clone());
                tail_ptr = next_ptr;
                tail_len = 0;
            }
            Response::Error(e) => return Err(e),
        }
    }
}

async fn advance_load_initial<T, S>(
    mut stream: Box<InnerGenericStream<T, S>>,
    remain: usize,
) -> Result<Result<GenericStream<T, S>, (Box<InnerGenericStream<T, S>>, usize)>, S::Error>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    loop {
        let control = Control::new(&stream.parameter, &mut []);
        let res = stream.source.next(control, remain).await;

        match res {
            Response::PreAllocated(_) => {
                // レスポンスにデータが無いため、ループする。
            }
            Response::Allocated(items, written) => {
                let node = IntermediateNode {
                    buf: items,
                    next: None,
                };
                let node_ptr = NodePtr {
                    raw: Arc::new(UnsafeCell::new(Node::Intermediate(node))),
                };

                if remain <= written {
                    let head_offset = remain;
                    let tail_len = written;

                    stream.pair = Some(HeadTailNodePair {
                        head_offset,
                        tail_len,
                        head_ptr: node_ptr.clone(),
                        tail_ptr: node_ptr,
                    });

                    return Ok(Ok(GenericStream { inner: stream }));
                } else {
                    stream.pair = Some(HeadTailNodePair {
                        head_offset: written,
                        tail_len: written,
                        head_ptr: node_ptr.clone(),
                        tail_ptr: node_ptr,
                    });

                    return Ok(Err((stream, remain - written)));
                }
            }
            Response::Finish => {
                let node_ptr = NodePtr {
                    raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                };

                stream.pair = Some(HeadTailNodePair {
                    head_offset: 0,
                    tail_len: 0,
                    head_ptr: node_ptr.clone(),
                    tail_ptr: node_ptr,
                });

                return Ok(Err((stream, remain)));
            }
            Response::Error(e) => return Err(e),
        }
    }
}
