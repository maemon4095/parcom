use crate::generic_stream::{HeadTailNodePair, InnerGenericStream};

use super::{
    control::{Control, Response},
    GenericStream, IntermediateNode, Node, NodePtr,
};
use parcom_core::{SegmentIterator, StreamSegment};
use parcom_streams_core::StreamSource;
use pin_project::pin_project;
use std::{cell::UnsafeCell, future::Future, pin::Pin, sync::Arc, task::Poll};

pub struct SegmentIter<'a, T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    stream: &'a mut InnerGenericStream<T, S>,
    current: Option<NodePtr<T>>,
    current_offset: usize,
    current_len: usize,
}

impl<'a, T, S> SegmentIter<'a, T, S>
where
    T: 'static + Default,
    S: StreamSource<Segment = [T]>,
{
    pub(super) fn new(stream: &'a mut GenericStream<T, S>) -> Self {
        let (current, current_offset, current_len) = match stream.inner.pair.as_ref() {
            Some(v) => {
                let len = unsafe {
                    match &*v.head_ptr.raw.get() {
                        Node::Intermediate(node) => {
                            if node.is_tail() {
                                v.tail_len
                            } else {
                                node.buf.len()
                            }
                        }
                        Node::Sentinel => 0,
                    }
                };

                (Some(v.head_ptr.clone()), v.head_offset, len)
            }
            None => (None, 0, 0),
        };

        Self {
            stream: &mut stream.inner,
            current,
            current_offset,
            current_len,
        }
    }
}

impl<'a, T, S> SegmentIterator for SegmentIter<'a, T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    type Segment = [T];
    type Error = S::Error;
    type Next<'b>
        = SegmentIterNext<'a, 'b, T, S>
    where
        Self: 'b;

    fn next<'b>(
        &'b mut self,
        size_hint: <Self::Segment as StreamSegment>::Length,
    ) -> Self::Next<'b> {
        SegmentIterNext::<'a, 'b, T, S>::new(self, size_hint)
    }
}

#[pin_project(!Unpin, project = SegmentIterNextStateProj)]
enum SegmentIterNextState<'a, 'b, T, S>
where
    'a: 'b,
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    Initial {
        iter: Option<&'b mut SegmentIter<'a, T, S>>,
        size_hint: usize,
    },
    Loading {
        #[pin]
        fut: Pin<Box<dyn 'b + Future<Output = Result<Option<&'b [T]>, S::Error>>>>,
    },
}

#[pin_project]
pub struct SegmentIterNext<'a, 'b, T, S>
where
    'a: 'b,
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    #[pin]
    state: SegmentIterNextState<'a, 'b, T, S>,
}

impl<'a, 'b, T, S> SegmentIterNext<'a, 'b, T, S>
where
    'a: 'b,
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    pub(super) fn new(iter: &'b mut SegmentIter<'a, T, S>, size_hint: usize) -> Self {
        Self {
            state: SegmentIterNextState::Initial {
                iter: Some(iter),
                size_hint,
            },
        }
    }
}

impl<'a, 'b, T, S> Future for SegmentIterNext<'a, 'b, T, S>
where
    'a: 'b,
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    type Output = Result<Option<&'b [T]>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.project().state;
        match state.as_mut().project() {
            SegmentIterNextStateProj::Initial { iter, size_hint } => {
                let iter: &'b mut _ = iter.take().expect("`poll` after completed");
                let Some(node) = iter.current.as_ref().map(|e| unsafe { &mut *e.raw.get() }) else {
                    // ストリームは一度も読み込まれていない。初回の読み込みを行う。
                    let fut = Box::pin(segment_next_initial(iter, *size_hint));
                    state.set(SegmentIterNextState::Loading { fut });
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                };

                let node = match node {
                    Node::Intermediate(v) => v,
                    Node::Sentinel => return Poll::Ready(Ok(None)),
                };

                if iter.current_offset < iter.current_len {
                    // currentはイテレーションされていない。currentのbufを返す。
                    let slice = &node.buf[iter.current_offset..iter.current_len];
                    iter.current_offset = iter.current_len;
                    return Poll::Ready(Ok(Some(slice)));
                }
                // currentはイテレーション済み。
                debug_assert_eq!(iter.current_offset, iter.current_len);

                if let Some(next) = node.next.clone() {
                    // 次のノードが読み込み済みである場合は、次のノードのセグメントを返す。
                    let next_node = unsafe { &mut *next.raw.get() };
                    let next_node = match next_node {
                        Node::Intermediate(v) => v,
                        Node::Sentinel => {
                            iter.current = Some(next);
                            return Poll::Ready(Ok(None));
                        }
                    };

                    iter.current_len = if next_node.is_tail() {
                        let pair = iter.stream.pair.as_mut().unwrap();
                        pair.tail_len
                    } else {
                        next_node.buf.len()
                    };
                    iter.current = Some(next);
                    iter.current_offset = iter.current_len;
                    return Poll::Ready(Ok(Some(&next_node.buf[..iter.current_len])));
                };

                // 読み込み済みのすべてのノードがイテレーション済み。新しく読み込む。
                let fut = Box::pin(segment_next_load(iter, node, *size_hint));
                state.set(SegmentIterNextState::Loading { fut });
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            SegmentIterNextStateProj::Loading { fut } => fut.poll(cx),
        }
    }
}

async fn segment_next_initial<'a, 'b, T, S>(
    iter: &'a mut SegmentIter<'b, T, S>,
    size_hint: usize,
) -> Result<Option<&'a [T]>, <S as StreamSource>::Error>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    loop {
        let control = Control::new(&iter.stream.parameter, &mut []);
        let res = iter.stream.source.next(control, size_hint).await;

        match res {
            Response::PreAllocated(_written) => {
                debug_assert_eq!(_written, 0);
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

                let slice = {
                    let node = unsafe { &*node_ptr.raw.get() };
                    let Node::Intermediate(node) = node else {
                        unreachable!()
                    };
                    &node.buf[..written]
                };

                iter.current = Some(node_ptr.clone());
                iter.current_offset = written;
                iter.current_len = written;

                iter.stream.pair = Some(HeadTailNodePair {
                    head_offset: 0,
                    tail_len: written,
                    head_ptr: node_ptr.clone(),
                    tail_ptr: node_ptr.clone(),
                });

                return Ok(Some(slice));
            }
            Response::Finish => {
                let node_ptr = NodePtr {
                    raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                };

                iter.current = Some(node_ptr.clone());
                iter.current_offset = 0;
                iter.current_len = 0;

                iter.stream.pair = Some(HeadTailNodePair {
                    head_offset: 0,
                    tail_len: 0,
                    head_ptr: node_ptr.clone(),
                    tail_ptr: node_ptr.clone(),
                });

                return Ok(None);
            }
            Response::Error(e) => return Err(e),
        }
    }
}

async fn segment_next_load<'a, 'b, T, S>(
    iter: &'a mut SegmentIter<'b, T, S>,
    node: &'a mut IntermediateNode<T>,
    size_hint: usize,
) -> Result<Option<&'a [T]>, <S as StreamSource>::Error>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    let pair = iter.stream.pair.as_mut().unwrap();

    loop {
        let control = Control::new(&iter.stream.parameter, &mut node.buf[iter.current_len..]);
        let res = iter.stream.source.next(control, size_hint).await;
        match res {
            Response::PreAllocated(writtern) => {
                if writtern == 0 {
                    // レスポンスにデータが無いため、ループする。
                    continue;
                }

                iter.current_len += writtern;
                pair.tail_len = iter.current_len;
                let slice = &node.buf[iter.current_offset..iter.current_len];
                iter.current_offset = iter.current_len;
                return Ok(Some(slice));
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
                node.buf.drain(iter.current_len..);
                node.next = Some(next_ptr.clone());

                pair.tail_ptr = next_ptr.clone();
                pair.tail_len = written;

                iter.current = Some(next_ptr);
                iter.current_len = written;
                iter.current_offset = written;

                let current = iter
                    .current
                    .as_ref()
                    .map(|e| unsafe { &mut *e.raw.get() })
                    .unwrap();
                let Node::Intermediate(current) = current else {
                    unreachable!()
                };

                return Ok(Some(&current.buf[..written]));
            }
            Response::Finish => {
                let next_ptr = NodePtr {
                    raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                };

                // 中間ノードのバッファを縮める。
                node.buf.drain(iter.current_len..);
                node.next = Some(next_ptr.clone());

                pair.tail_ptr = next_ptr.clone();
                pair.tail_len = 0;

                iter.current = Some(next_ptr);
                iter.current_len = 0;
                iter.current_offset = 0;

                return Ok(None);
            }
            Response::Error(e) => return Err(e),
        }
    }
}
