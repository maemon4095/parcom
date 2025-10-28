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
    /// データを読み込みストリーム末尾に追加し、書き込まれた要素数を返す。
    /// 戻り値が`0`の場合、データ末端まで到達しており、以降の`load`の呼び出しでデータが追加されることはない。
    async fn load(&mut self, size_hint: usize) -> Result<usize, S::Error> {
        if self.pair.is_none() {
            self.load_initial(size_hint).await
        } else {
            self.load_tail(size_hint).await
        }
    }

    async fn load_initial(&mut self, size_hint: usize) -> Result<usize, S::Error> {
        loop {
            let control = Control::new(&self.parameter, &mut []);
            let res = self.source.next(control, size_hint).await;

            match res {
                Response::PreAllocated(_) => {
                    continue;
                }
                Response::Allocated(buf, written) => {
                    if written == 0 {
                        continue;
                    }

                    let node = IntermediateNode { buf, next: None };
                    let node_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Intermediate(node))),
                    };

                    self.pair = Some(HeadTailNodePair {
                        head_start: 0,
                        tail_end: written,
                        head_ptr: node_ptr.clone(),
                        tail_ptr: node_ptr,
                    });

                    return Ok(written);
                }

                Response::Finish => {
                    let node_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                    };

                    self.pair = Some(HeadTailNodePair {
                        head_start: 0,
                        tail_end: 0,
                        head_ptr: node_ptr.clone(),
                        tail_ptr: node_ptr,
                    });

                    return Ok(0);
                }
                Response::Error(e) => {
                    return Err(e);
                }
            }
        }
    }

    async fn load_tail(&mut self, size_hint: usize) -> Result<usize, S::Error> {
        let HeadTailNodePair {
            tail_end: tail_len,
            tail_ptr,
            ..
        } = self.pair.as_mut().unwrap();

        let Node::Intermediate(tail) = (unsafe { &mut *tail_ptr.raw.get() }) else {
            return Ok(0);
        };

        let initial_tail_len = *tail_len;
        loop {
            let control = Control::new(&self.parameter, &mut tail.buf[initial_tail_len..]);
            let res = self.source.next(control, size_hint).await;

            match res {
                Response::PreAllocated(written) => {
                    // tailのバッファ末尾にデータが追加された。
                    if written == 0 {
                        continue;
                    }

                    *tail_len = initial_tail_len + written;

                    break Ok(written);
                }
                Response::Allocated(items, written) => {
                    // tailのバッファ末尾にデータは追加されず、新しいバッファが追加された。
                    if written == 0 {
                        continue;
                    }

                    let next = IntermediateNode {
                        buf: items,
                        next: None,
                    };
                    let next_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Intermediate(next))),
                    };

                    tail.buf.drain(initial_tail_len..);
                    tail.next = Some(next_ptr.clone());

                    *tail_len = written;
                    *tail_ptr = next_ptr;

                    break Ok(written);
                }
                Response::Finish => {
                    // tailのバッファ末尾にデータは追加されず、ストリームが閉じられた。
                    let next_ptr = NodePtr {
                        raw: Arc::new(UnsafeCell::new(Node::Sentinel)),
                    };

                    tail.buf.drain(initial_tail_len..);
                    tail.next = Some(next_ptr.clone());

                    *tail_ptr = next_ptr;

                    break Ok(0);
                }
                Response::Error(e) => {
                    break Err(e);
                }
            }
        }
    }

    /// すでに読み込まれた分をすべて捨て、ストリームの先頭を現在の末尾まで移動する。
    fn advance_already_loaded_all(&mut self) {
        let Some(pair) = self.pair.as_mut() else {
            return;
        };
        pair.head_ptr = pair.tail_ptr.clone();
        pair.head_start = pair.tail_end;
    }

    /// ストリームの先頭を`delta`だけ進める。
    ///
    /// `delta`の値がすでに読み込まれたデータの長さよりも大きい場合、不足分が返される。
    fn advance_already_loaded(&mut self, delta: usize) -> Result<(), usize> {
        if delta == 0 {
            return Ok(());
        }

        let Some(pair) = self.pair.as_mut() else {
            return Err(delta);
        };
        let head_ptr = pair.head_ptr.clone();
        let head = unsafe { &*head_ptr.raw.get() };
        let Node::Intermediate(head) = head else {
            return Err(delta);
        };

        let Some(head_next_ptr) = head.next.clone() else {
            // headがtailの場合
            let head_len = pair.tail_end - pair.head_start;

            if delta <= head_len {
                pair.head_start += delta;
                return Ok(());
            } else {
                pair.head_start = pair.tail_end;
                return Err(delta - head_len);
            }
        };

        let head_len = head.buf.len() - pair.head_start;
        if delta <= head_len {
            pair.head_start += delta;
            return Ok(());
        }

        let mut remain = delta - head_len;
        let mut node_ptr = head_next_ptr;

        loop {
            let node = unsafe { &*node_ptr.raw.get() };
            let Node::Intermediate(node) = node else {
                pair.head_start = pair.tail_end;
                pair.head_ptr = node_ptr;
                return Err(remain);
            };

            let Some(next_ptr) = node.next.clone() else {
                let node_len = pair.tail_end;
                pair.head_ptr = node_ptr;

                if remain <= node_len {
                    pair.head_start = remain;
                    return Ok(());
                } else {
                    pair.head_start = node_len;
                    return Err(remain - node_len);
                }
            };

            let node_len = node.buf.len();
            if remain <= node_len {
                pair.head_ptr = node_ptr;
                pair.head_start = remain;
                return Ok(());
            }

            remain -= node_len;
            node_ptr = next_ptr;
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
    /// NOTE: `head`が`Node::Sentinel`である場合、`head_start`の値を参照してはならない。
    head_start: usize,
    /// NOTE: `tail`が`Node::Sentinel`である場合、`tail_end`の値を参照してはならない。
    tail_end: usize,
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
    /// NOTE: ノードがtailである場合、`buf.len()`は書き込まれた分より長い場合がある。実際の長さはstreamの`tail_end`を参照する必要がある。 ノードがtailでない場合、`buf.len()`は書き込まれた分と一致する。
    buf: Vec<T>,
    next: Option<NodePtr<T>>,
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

    fn create_fragmented_serial(lengths: impl IntoIterator<Item = usize>) -> Vec<Vec<usize>> {
        lengths
            .into_iter()
            .scan(0, |sum, len| {
                let offset = *sum;
                *sum += len;
                let buf = std::iter::repeat_n((), len)
                    .enumerate()
                    .map(|(i, _)| offset + i)
                    .collect();
                Some(buf)
            })
            .collect()
    }

    fn create_source(
        segments: &[Vec<usize>],
    ) -> impl StreamSource<Segment = [usize], Error = parcom_core::Never> {
        DelayedSource::new(
            IteratorSource::new(segments.to_vec()),
            Duration::from_millis(100),
            0.5..1.0,
        )
    }

    #[tokio::test]
    async fn test_iterate_all() {
        let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
        let source = create_source(&segments);

        let mut stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        let mut iter = stream.segments();
        let mut buf = Vec::new();

        while let Some(segment) = iter.next(0).await.unwrap() {
            buf.extend_from_slice(segment);
        }
        assert_eq!(buf, expected);

        let mut iter = stream.segments();
        buf.clear();
        while let Some(segment) = iter.next(0).await.unwrap() {
            buf.extend_from_slice(segment);
        }

        assert_eq!(buf, expected)
    }

    #[tokio::test]
    async fn test_iterate_partial() {
        let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
        let source = create_source(&segments);

        let mut stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        let mut iter = stream.segments();
        let mut buf = Vec::new();

        let mut discard = 2;
        while let Some(_) = iter.next(0).await.unwrap() {
            discard -= 1;

            if discard == 0 {
                break;
            }
        }
        assert!(expected.starts_with(&buf));

        let mut iter = stream.segments();
        buf.clear();
        while let Some(segment) = iter.next(0).await.unwrap() {
            buf.extend_from_slice(segment);
        }

        assert_eq!(buf, expected)
    }

    #[tokio::test]
    async fn test_advance_all_after_iterate_all() {
        let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
        let source = create_source(&segments);

        let mut stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        let mut iter = stream.segments();

        while let Some(_) = iter.next(0).await.unwrap() {}

        stream.advance(expected.len()).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_advance_over_after_iterate_all() {
        let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
        let source = create_source(&segments);

        let mut stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        let mut iter = stream.segments();

        while let Some(_) = iter.next(0).await.unwrap() {}

        stream.advance(expected.len() + 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_advance_all_before_iteration() {
        let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
        let source = create_source(&segments);

        let stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        stream.advance(expected.len()).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_advance_over_before_iteration() {
        let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
        let source = create_source(&segments);

        let stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        stream.advance(expected.len() + 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_advance() {
        let lengths = [23, 18, 8, 0, 0, 1, 3, 2, 19, 11];
        let segments = create_fragmented_serial(lengths);
        let advance_steps = [3, 1, 4, 12, 0, 9, 1, 3, 32, 20];
        assert_eq!(advance_steps.iter().sum::<usize>(), lengths.iter().sum());
        let source = create_source(&segments);

        let mut stream = GenericStream::new(
            GenericStreamParameter::default().with_min_capacity(16),
            source,
        );

        let expected: Vec<usize> = segments.iter().flatten().copied().collect();

        let mut offset = 0;
        for advance_step in advance_steps {
            offset += advance_step;
            stream = stream.advance(advance_step).await.unwrap();

            let mut iter = stream.segments();
            let mut buf = Vec::new();
            while let Some(segment) = iter.next(0).await.unwrap() {
                buf.extend_from_slice(segment);
            }

            assert_eq!(buf, &expected[offset..])
        }
    }
}
