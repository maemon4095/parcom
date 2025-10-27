mod advance;
mod control;
mod parameter;
mod segment_iter;

use parcom_core::{Stream, StreamSegment};
use parcom_streams_core::StreamSource;
use std::{cell::UnsafeCell, sync::Arc};

pub use advance::Advance;
pub use parameter::GenericStreamParameter;
pub use segment_iter::{SegmentIter, SegmentIterNext};

use crate::generic_stream::control::{Control, Response};

// SAFETY:
//  streamは`Stream::advance(self, delta)`か`Stream::segments(&mut self)`のみで変更されるため、ロック不要。
//  `RewindStream::Anchor`のために`Arc`を使っているが、`RewindStream::rewind`ではデータを変更しない。
//  そのため、`UnsafeCell`で可変参照を取得するのは`Stream::advance`か`Stream::segments`のみであることが保証される。

#[derive(Debug)]
pub struct GenericStream<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    inner: Box<InnerGenericStream<T, S>>,
}

impl<T, S> GenericStream<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    pub fn new(parameter: GenericStreamParameter, source: S) -> Self {
        Self {
            inner: Box::new(InnerGenericStream {
                parameter,
                source,
                pair: None,
            }),
        }
    }
}

#[derive(Debug)]
struct InnerGenericStream<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    parameter: GenericStreamParameter,
    source: S,
    pair: Option<HeadTailNodePair<T>>,
}

impl<T, S> InnerGenericStream<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    fn tail_ptr(&self) -> Option<NodePtr<T>> {
        self.pair.as_ref().map(|p| p.tail_ptr.clone())
    }

    /// tailのバッファを埋めるように読み込みを行う。新しく末尾にノードが追加される場合もある。
    ///
    /// `Ok`の場合、末尾に追加された要素数を返す。
    async fn load(&mut self, size_hint: usize) -> Result<usize, S::Error> {
        if self.pair.is_none() {
            self.load_initial(size_hint).await
        } else {
            self.load_tail(size_hint).await
        }
    }

    async fn load_initial(&mut self, size_hint: usize) -> Result<usize, S::Error> {
        let mut buf = Vec::new();
        let mut remain = size_hint;
        let mut written = 0;
        loop {
            let control = Control::new(&self.parameter, &mut buf[written..]);
            let res = self.source.next(control, remain).await;

            match res {
                Response::PreAllocated(w) => {
                    written += w;
                    remain = buf.len() - written;

                    if remain == 0 {
                        break Ok(written);
                    }
                }
                Response::Allocated(tail_buf, tail_written) => {
                    buf.drain(written..);

                    let tail = IntermediateNode {
                        buf: tail_buf,
                        next: None,
                    };
                    let tail_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Intermediate(tail))),
                    };
                    let head = IntermediateNode {
                        buf,
                        next: Some(tail_ptr.clone()),
                    };
                    let head_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Intermediate(head))),
                    };

                    let total_written = tail_written + written;

                    self.pair = Some(HeadTailNodePair {
                        head_offset: 0,
                        tail_len: tail_written,
                        head_ptr,
                        tail_ptr,
                    });

                    break Ok(total_written);
                }

                Response::Finish => {
                    let pair = if buf.is_empty() {
                        let node_ptr = NodePtr {
                            raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                        };
                        HeadTailNodePair {
                            head_offset: 0,
                            tail_len: 0,
                            head_ptr: node_ptr.clone(),
                            tail_ptr: node_ptr,
                        }
                    } else {
                        let tail_ptr = NodePtr {
                            raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                        };

                        buf.drain(written..);
                        let head = IntermediateNode {
                            buf,
                            next: Some(tail_ptr.clone()),
                        };
                        let head_ptr = NodePtr {
                            raw: Arc::new(UnsafeCell::new(Node::Intermediate(head))),
                        };

                        HeadTailNodePair {
                            head_offset: 0,
                            tail_len: 0,
                            head_ptr,
                            tail_ptr,
                        }
                    };

                    let total_written = written;

                    self.pair = Some(pair);

                    break Ok(total_written);
                }
                Response::Error(e) => {
                    if !buf.is_empty() {
                        let node = IntermediateNode { buf, next: None };
                        let node_ptr = NodePtr {
                            raw: Arc::new(UnsafeCell::new(Node::Intermediate(node))),
                        };

                        self.pair = Some(HeadTailNodePair {
                            head_offset: 0,
                            tail_len: written,
                            head_ptr: node_ptr.clone(),
                            tail_ptr: node_ptr,
                        });
                    }

                    break Err(e);
                }
            }
        }
    }

    async fn load_tail(&mut self, size_hint: usize) -> Result<usize, S::Error> {
        let HeadTailNodePair {
            tail_len, tail_ptr, ..
        } = self.pair.as_mut().unwrap();

        let Node::Intermediate(tail) = (unsafe { &mut *tail_ptr.raw.get() }) else {
            return Ok(0);
        };

        let initial_tail_len = *tail_len;
        let mut current_tail_len = *tail_len;
        let mut remain = size_hint;
        loop {
            let control = Control::new(&self.parameter, &mut tail.buf[current_tail_len..]);
            let res = self.source.next(control, remain).await;

            match res {
                Response::PreAllocated(w) => {
                    current_tail_len += w;
                    remain = tail.buf.len() - current_tail_len;

                    if remain == 0 {
                        let total_written = current_tail_len - initial_tail_len;
                        *tail_len = current_tail_len;
                        break Ok(total_written);
                    }
                }
                Response::Allocated(items, written) => {
                    let next = IntermediateNode {
                        buf: items,
                        next: None,
                    };
                    let next_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Intermediate(next))),
                    };

                    let tail_written = current_tail_len - initial_tail_len;
                    let total_written = tail_written + written;

                    tail.buf.drain(current_tail_len..);
                    tail.next = Some(next_ptr.clone());

                    *tail_len = written;
                    *tail_ptr = next_ptr;

                    break Ok(total_written);
                }

                Response::Finish => {
                    let next_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                    };

                    let tail_written = current_tail_len - initial_tail_len;

                    tail.buf.drain(current_tail_len..);
                    tail.next = Some(next_ptr.clone());

                    *tail_ptr = next_ptr;

                    break Ok(tail_written);
                }
                Response::Error(e) => {
                    *tail_len = current_tail_len;
                    break Err(e);
                }
            }
        }
    }
}

impl<T, S> Stream for GenericStream<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    type Segment = [T];
    type Error = S::Error;
    type SegmentIter<'a>
        = SegmentIter<'a, T, S>
    where
        Self: 'a;
    type Advance = Advance<T, S>;

    fn segments(&mut self) -> Self::SegmentIter<'_> {
        SegmentIter::new(self)
    }

    fn advance(self, delta: <Self::Segment as StreamSegment>::Length) -> Self::Advance {
        Advance::new(self.inner, delta)
    }
}

#[derive(Debug)]
struct HeadTailNodePair<T: 'static + Default> {
    /// NOTE: `head`が`Node::Sentinel`である場合、`head_offset`の値を参照してはならない。
    head_offset: usize,
    /// NOTE: `tail`が`Node::Sentinel`である場合、`tail_len`の値を参照してはならない。
    tail_len: usize,
    head_ptr: NodePtr<T>,
    tail_ptr: NodePtr<T>,
}

#[derive(Debug)]
struct NodePtr<T> {
    raw: Arc<UnsafeCell<Node<T>>>,
}

impl<T> Clone for NodePtr<T> {
    fn clone(&self) -> Self {
        Self {
            raw: Arc::clone(&self.raw),
        }
    }
}

#[derive(Debug)]
enum Node<T> {
    Intermediate(IntermediateNode<T>),
    Sentinel,
}

#[derive(Debug)]
struct IntermediateNode<T> {
    /// NOTE: ノードがtailである場合、`buf.len()`は書き込まれた分より長い場合がある。実際の長さはstreamの`tail_len`を参照する必要がある。 ノードがtailでない場合、`buf.len()`は書き込まれた分と一致する。
    buf: Vec<T>,
    next: Option<NodePtr<T>>,
}

impl<T> IntermediateNode<T> {
    fn is_tail(&self) -> bool {
        self.next.is_none()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stream_source::iterator_source::IteratorSource;
    use parcom_core::SegmentIterator;
    use parcom_streams_core::StreamControl;
    use std::{future::Future, ops::Range, pin::Pin, time::Duration};

    struct DelayedSource<S: StreamSource> {
        inner: S,
        delay: Duration,
        jitter: std::ops::Range<f64>,
    }

    impl<S: StreamSource> DelayedSource<S> {
        pub fn new(source: S, delay: Duration, jitter: Range<f64>) -> Self {
            Self {
                inner: source,
                delay,
                jitter,
            }
        }
    }

    impl<S> StreamSource for DelayedSource<S>
    where
        S: StreamSource,
    {
        type Segment = S::Segment;
        type Error = S::Error;

        type Next<'a, C>
            = Pin<Box<dyn 'a + Future<Output = C::Response>>>
        where
            Self: 'a,
            C: 'a + StreamControl<Segment = Self::Segment, Error = Self::Error>;

        fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
        where
            C: 'a + StreamControl<Segment = Self::Segment, Error = Self::Error>,
        {
            let jitter = rand::random_range(self.jitter.clone());
            let delay = self.delay.mul_f64(jitter);
            Box::pin(async move {
                tokio::time::sleep(delay).await;
                self.inner.next(control, size_hint).await
            })
        }
    }

    #[tokio::test]
    async fn test_load_all() {
        let segments = &["", "abc", "", "d", "efg"];
        let source = DelayedSource::new(
            IteratorSource::new(segments),
            Duration::from_millis(100),
            0.5..1.0,
        );

        let mut stream = GenericStream::new(GenericStreamParameter::default(), source);
        let mut iter = stream.segments();
        let mut buf = Vec::new();
        while let Some(segment) = iter.next(0).await.unwrap() {
            buf.extend_from_slice(segment);
        }

        let expected: Vec<u8> = segments
            .iter()
            .flat_map(|e| e.as_bytes())
            .copied()
            .collect();

        assert_eq!(buf, expected)
    }

    #[tokio::test]
    async fn test_loaded_all() {
        let segments = &["", "abc", "", "d", "efg"];
        let source = DelayedSource::new(
            IteratorSource::new(segments),
            Duration::from_millis(100),
            0.5..1.0,
        );

        let mut stream = GenericStream::new(GenericStreamParameter::default(), source);
        let mut iter = stream.segments();
        while let Some(_) = iter.next(0).await.unwrap() {}

        let mut iter = stream.segments();
        let mut buf = Vec::new();
        while let Some(segment) = iter.next(0).await.unwrap() {
            buf.extend_from_slice(segment);
        }

        let expected: Vec<u8> = segments
            .iter()
            .flat_map(|e| e.as_bytes())
            .copied()
            .collect();

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_loaded_partial() {}

    #[test]
    fn test_advance_fully_loaded_advance_all() {
        todo!()
    }

    #[test]
    fn test_advance_fully_loaded_advance_partial() {
        todo!()
    }

    #[test]
    fn test_advance_partially_loaded_advance_partial() {
        todo!()
    }

    #[test]
    fn test_advance_partially_loaded_advance_all() {
        todo!()
    }
}
