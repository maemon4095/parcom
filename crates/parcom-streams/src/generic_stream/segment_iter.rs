use super::{GenericStream, InnerGenericStream, Node, NodePtr};
use parcom_core::{SegmentIterator, StreamSegment};
use parcom_streams_core::StreamSource;
use pin_project::pin_project;
use std::{future::Future, pin::Pin, task::Poll};

pub struct SegmentIter<'a, T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    stream: &'a mut InnerGenericStream<T, S>,
    current: Option<NodePtr<T>>,
    current_start: usize,
}

impl<'a, T, S> SegmentIter<'a, T, S>
where
    T: 'static + Default,
    S: StreamSource<Segment = [T]>,
{
    pub(super) fn new(stream: &'a mut GenericStream<T, S>) -> Self {
        let (current, current_start) = match stream.inner.pair.as_ref() {
            Some(v) => (Some(v.head_ptr.clone()), v.head_start),
            None => (None, 0),
        };

        Self {
            stream: &mut stream.inner,
            current,
            current_start,
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

                if let Some(next_ptr) = node.next.clone() {
                    // `node`はtailではない。
                    if iter.current_start < node.buf.len() {
                        // `node`末尾にイテレーション済みでないデータがある場合、`node`末尾のデータを返す。
                        let segment = &node.buf[iter.current_start..];
                        iter.current_start = 0; // 次のノードはheadではない。
                        iter.current = Some(next_ptr);
                        return Poll::Ready(Ok(Some(segment)));
                    }

                    // `node`のデータはすべてイテレーション済み。
                    let next = unsafe { &*next_ptr.raw.get() };
                    let Node::Intermediate(next) = next else {
                        iter.current = Some(next_ptr);
                        iter.current_start = 0;
                        return Poll::Ready(Ok(None));
                    };

                    if let Some(next_next_ptr) = next.next.clone() {
                        // `next`がtailでない場合、nextのデータを返す。
                        iter.current = Some(next_next_ptr);
                        iter.current_start = 0;
                        return Poll::Ready(Ok(Some(&next.buf)));
                    }

                    // `next`はtailであるため、データの読み込みを行ってから返す。
                    iter.current = Some(next_ptr);
                    iter.current_start = 0;
                };

                // `iter.current`はtailであるため、データの読み込みを行ってから返す。
                let fut = Box::pin(segment_next_load(iter, *size_hint));
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
    iter.stream.load_initial(size_hint).await?;
    let pair = iter.stream.pair.as_ref().unwrap();

    let head_ptr = pair.head_ptr.clone();
    let head_start = pair.head_start;
    let head = unsafe { &*head_ptr.raw.get() };

    let Node::Intermediate(head) = head else {
        iter.current = Some(head_ptr);
        return Ok(None);
    };

    if let Some(head_next) = head.next.clone() {
        iter.current = Some(head_next);
        iter.current_start = 0;
        let head_end = head.buf.len();
        let segment = &head.buf[head_start..head_end];
        Ok(Some(segment))
    } else {
        let head_end = pair.tail_end;
        iter.current = Some(head_ptr);
        iter.current_start = head_end;
        let segment = &head.buf[head_start..head_end];
        Ok(Some(segment))
    }
}

async fn segment_next_load<'a, 'b, T, S>(
    iter: &'a mut SegmentIter<'b, T, S>,
    size_hint: usize,
) -> Result<Option<&'a [T]>, <S as StreamSource>::Error>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    iter.stream.load_tail(size_hint).await?;
    let node_ptr = iter.current.clone().unwrap();
    let Node::Intermediate(node) = (unsafe { &*node_ptr.raw.get() }) else {
        return Ok(None);
    };

    match node.next.clone() {
        Some(next_ptr) => {
            // `node`はtailではないため、`node.buf.len()`が`node`のセグメントの長さになる。
            let node_end = node.buf.len();
            if iter.current_start < node_end {
                // `node`の末尾にデータが追加されている。
                let segment = &node.buf[iter.current_start..node_end];
                iter.current = Some(next_ptr);
                iter.current_start = 0; // `node.next`はheadにならない。

                return Ok(Some(segment));
            } else {
                // `node`の末尾にデータは追加されていないため、次のノードのデータを返す。
                let next = unsafe { &*next_ptr.raw.get() };
                let segment = match next {
                    Node::Intermediate(next) => {
                        let pair = iter.stream.pair.as_ref().unwrap();
                        let next_len = if next.next.is_none() {
                            pair.tail_end
                        } else {
                            next.buf.len()
                        };
                        iter.current = Some(next_ptr);
                        iter.current_start = next_len;

                        // `next`はheadにならない。
                        Some(&next.buf[..next_len])
                    }
                    Node::Sentinel => {
                        iter.current = Some(next_ptr);
                        iter.current_start = 0;
                        None
                    }
                };

                Ok(segment)
            }
        }
        _ => {
            // `node`はtailのままであるため、末尾にデータが追加されている。
            let pair = iter.stream.pair.as_ref().unwrap();
            let segment = &node.buf[iter.current_start..pair.tail_end];

            iter.current_start = pair.tail_end;

            Ok(Some(segment))
        }
    }
}
