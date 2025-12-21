mod advance;
mod loader;
mod segments;

use crate::{
    generic_stream::{advance::Advance, segments::Segments},
    util::identity::Identity,
};
use parcom_core::Sequence;
use parcom_streams_core::{StreamDriver, StreamSource};
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicUsize, Arc, OnceLock},
};

pub use loader::GenericStreamLoader;

type NodePtr<T> = Arc<Node<T>>;

pub struct GenericStream<'me, T, S, D>
where
    T: 'me,
    S: 'me + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'me,
{
    identity: Identity,
    driver_session: D::Session,

    /// NOTE: `head`が`Node::Sentinel`または`Node::Head`である場合、`head_start`の値を参照してはならない。
    head_start: usize,
    head_ptr: NodePtr<T>,

    _phantom: PhantomData<&'me mut ()>,
}

impl<'me, T, S, D> GenericStream<'me, T, S, D>
where
    T: 'me,
    S: 'me + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'me,
{
    pub fn new(source: S) -> Self
    where
        D: Default,
    {
        Self::new_with(source, D::default())
    }

    pub fn new_with(source: S, driver: D) -> Self {
        let node_ptr = Arc::new(Node::Head(OnceLock::new()));
        let identity = Identity::new();
        let loader = GenericStreamLoader::new(source, node_ptr.clone());
        let session = driver.start(loader);
        Self {
            identity,
            driver_session: session,
            head_start: 0,
            head_ptr: node_ptr,
            _phantom: PhantomData,
        }
    }

    fn session_head_pair(&mut self) -> (&mut D::Session, &NodePtr<T>) {
        let session = &mut self.driver_session;
        let node = &self.head_ptr;
        (session, node)
    }
}

impl<'me, T, S, D> Sequence for GenericStream<'me, T, S, D>
where
    T: 'me,
    S: 'me + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'me,
{
    type Segment = [T];
    type Error = S::Error;
    type Segments<'b>
        = Segments<'me, 'b, T, S, D>
    where
        Self: 'b;

    type Advance = Advance<'me, T, S, D>;

    fn segments<'b>(&'b mut self) -> Self::Segments<'b> {
        Segments::new(self)
    }

    fn advance(
        self,
        delta: <Self::Segment as parcom_core::SequenceSegment>::Length,
    ) -> Self::Advance {
        Advance::new(self, delta)
    }
}

#[derive(Debug)]
enum Node<T> {
    Head(OnceLock<Arc<Node<T>>>),
    Intermediate(IntermediateNode<T>),
    Sentinel,
}

#[derive(Debug)]
struct IntermediateNode<T> {
    buf: Vec<T>,
    examined: AtomicUsize,
    next: OnceLock<Arc<Node<T>>>,
}

impl<T> IntermediateNode<T> {
    pub fn new(buf: Vec<T>) -> Self {
        Self {
            buf,
            examined: AtomicUsize::new(0),
            next: OnceLock::new(),
        }
    }
    pub fn new_with(buf: Vec<T>, next: Arc<Node<T>>) -> Self {
        Self {
            buf,
            examined: AtomicUsize::new(0),
            next: OnceLock::from(next),
        }
    }

    pub fn set_examined(&self, examind: usize) -> usize {
        let last_examind = self
            .examined
            .fetch_max(examind, std::sync::atomic::Ordering::Relaxed);

        examind.saturating_sub(last_examind)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stream_source::iterator_source::IteratorSource;
    use crate::task_spawner::TaskSpawner;
    use crate::util::trace_cell::TraceCell;
    use crate::{stream_driver::ConcurrentDriver, util::trace_cell::TRACE_STORE};
    use parcom_core::SegmentStream;
    use parcom_streams_core::{StreamControl, StreamSource};
    use std::future::IntoFuture;
    use std::{future::Future, ops::Range, pin::Pin, time::Duration};

    struct TokioSpawner;

    impl TaskSpawner for TokioSpawner {
        type Error = tokio::task::JoinError;
        type Handle<T: Send> = tokio::task::JoinHandle<T>;

        fn spawn<T, F>(&self, task: F) -> Self::Handle<T>
        where
            T: 'static + Send,
            F: 'static + Send + IntoFuture<Output = T>,
        {
            tokio::task::spawn_blocking(move || {
                let fut = task.into_future();
                tokio::runtime::Handle::current().block_on(fut)
            })
        }
    }

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
        type Item = S::Item;
        type Error = S::Error;

        type Next<'a, C>
            = Pin<Box<dyn 'a + Future<Output = C::Result>>>
        where
            Self: 'a,
            C: 'a + StreamControl<Item = Self::Item, Error = Self::Error>;

        fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
        where
            C: 'a + StreamControl<Item = Self::Item, Error = Self::Error>,
        {
            let jitter = rand::random_range(self.jitter.clone());
            let delay = self.delay.mul_f64(jitter);
            Box::pin(async move {
                tokio::time::sleep(delay).await;
                self.inner.next(control, size_hint).await
            })
        }
    }

    fn create_fragmented_serial(
        lengths: impl IntoIterator<Item = usize>,
    ) -> Vec<Vec<TraceCell<usize>>> {
        lengths
            .into_iter()
            .scan(0, |sum, len| {
                let offset = *sum;
                *sum += len;
                let buf = std::iter::repeat_n((), len)
                    .enumerate()
                    .map(|(i, _)| TraceCell::new(offset + i))
                    .collect();
                Some(buf)
            })
            .collect()
    }

    fn create_source(
        segments: &[Vec<TraceCell<usize>>],
    ) -> impl StreamSource<Item = TraceCell<usize>, Error = parcom_core::Never> {
        DelayedSource::new(
            IteratorSource::new(segments.to_vec()),
            Duration::from_millis(100),
            0.5..1.0,
        )
    }

    #[tokio::test]
    async fn test_iterate_all() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
            let source = create_source(&segments);

            let mut stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            let mut iter = stream.segments();
            let mut buf = Vec::new();

            while let Some(segment) = iter.next().await.unwrap() {
                assert!(!segment.is_empty());
                buf.extend_from_slice(segment);
            }
            assert_eq!(buf, expected);

            let mut iter = stream.segments();
            buf.clear();
            while let Some(segment) = iter.next().await.unwrap() {
                assert!(!segment.is_empty());
                buf.extend_from_slice(segment);
            }

            assert_eq!(buf, expected);
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }

    #[tokio::test]
    async fn test_iterate_partial() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
            let source = create_source(&segments);

            let mut stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            let mut iter = stream.segments();
            let mut buf = Vec::new();

            let mut discard = 2;
            while let Some(_) = iter.next().await.unwrap() {
                discard -= 1;

                if discard == 0 {
                    break;
                }
            }
            assert!(expected.starts_with(&buf));

            let mut iter = stream.segments();
            buf.clear();
            while let Some(segment) = iter.next().await.unwrap() {
                assert!(!segment.is_empty());
                buf.extend_from_slice(segment);
            }

            assert_eq!(buf, expected);
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }

    #[tokio::test]
    async fn test_advance_all_after_iterate_all() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
            let source = create_source(&segments);

            let mut stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            let mut iter = stream.segments();

            while let Some(_) = iter.next().await.unwrap() {}

            stream.advance(expected.len()).await.unwrap();
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_advance_over_after_iterate_all() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
            let source = create_source(&segments);

            let mut stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            let mut iter = stream.segments();

            while let Some(_) = iter.next().await.unwrap() {}

            let _ = stream.advance(expected.len() + 1).await;
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }

    #[tokio::test]
    async fn test_advance_all_before_iteration() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
            let source = create_source(&segments);

            let stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            stream.advance(expected.len()).await.unwrap();
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_advance_over_before_iteration() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let segments = create_fragmented_serial([23, 18, 8, 0, 0, 1, 3, 2, 19, 11]);
            let source = create_source(&segments);

            let stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            let _ = stream.advance(expected.len() + 1).await;
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }

    #[tokio::test]
    async fn test_advance_step() {
        let tracing = TRACE_STORE.start_tracing();
        {
            let lengths = [23, 18, 8, 0, 0, 1, 3, 2, 19, 11];
            let segments = create_fragmented_serial(lengths);
            let advance_steps = [23, 0, 0, 1, 0, 4, 12, 0, 9, 32, 1, 3, 0, 0, 0];
            assert_eq!(advance_steps.iter().sum::<usize>(), lengths.iter().sum());
            let source = create_source(&segments);

            let mut stream =
                GenericStream::new_with(source, ConcurrentDriver::new(64, 128, TokioSpawner));

            let expected: Vec<_> = segments.iter().flatten().cloned().collect();

            let mut offset = 0;
            for advance_step in advance_steps {
                offset += advance_step;
                stream = stream.advance(advance_step).await.unwrap();

                let mut iter = stream.segments();
                let mut buf = Vec::new();
                while let Some(segment) = iter.next().await.unwrap() {
                    assert!(!segment.is_empty());
                    buf.extend_from_slice(segment);
                }

                assert_eq!(buf, &expected[offset..])
            }
        }
        assert_eq!(tracing.allocs(), tracing.drops())
    }
}
