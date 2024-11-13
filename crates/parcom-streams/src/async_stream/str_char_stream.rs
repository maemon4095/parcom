use crate::{
    util::{InitializedSharedCell, OnceCell},
    StreamSource,
};
use futures::{FutureExt, StreamExt as _};
use std::sync::Arc;

pub struct StrCharStream<S>
where
    S: StreamSource<Output = String>,
{
    source: S,
    head: Arc<OnceCell<Option<InnerNode>>>,
    offset: usize,
}

struct InnerNode {
    segment: String,
    next: Arc<OnceCell<Option<Self>>>,
}

pub struct Node {
    inner: InitializedSharedCell<Option<InnerNode>>,
    offset: usize,
}

impl std::ops::Deref for Node {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner.as_ref().unwrap().segment[self.offset..]
    }
}

impl<S: StreamSource> parcom_core::ParcomStream for StrCharStream<S>
where
    S: 'static + StreamSource<Output = String> + Send,
    S::Future: Send,
{
    type Segment = str;
    type SegmentStream = futures::stream::BoxStream<'static, Node>;
    type Advance = futures::future::BoxFuture<'static, Self>;

    fn segments(&self) -> Self::SegmentStream {
        let next = Arc::clone(&self.head);
        let offset = self.offset;
        let source = self.source.clone();

        futures::stream::unfold((next, offset), move |(next, offset)| {
            let mut source = source.clone();
            async move {
                let initialized = next
                    .get_or_init_owned(async {
                        let Some(segment) = source.recv().await else {
                            return None;
                        };
                        Some(InnerNode {
                            segment,
                            next: Arc::new(OnceCell::new()),
                        })
                    })
                    .await;

                let Some(e) = initialized.as_ref() else {
                    return None;
                };

                let next = Arc::clone(&e.next);
                let node = Node {
                    inner: initialized,
                    offset,
                };

                Some((node, (next, offset)))
            }
        })
        .boxed()
    }

    fn advance(self, count: usize) -> Self::Advance {
        let mut segments = self.segments();
        let mut rest = count;
        let source = self.source.clone();
        async move {
            while let Some(n) = segments.next().await {
                let len = n.len() - n.offset;
                if len >= rest {
                    return Self {
                        source,
                        head: n.inner.into_cell(),
                        offset: len - rest,
                    };
                }
                rest -= len;
            }

            Self {
                source,
                head: Arc::new(OnceCell::new_initialized(None)),
                offset: 0,
            }
        }
        .boxed()
    }
}
