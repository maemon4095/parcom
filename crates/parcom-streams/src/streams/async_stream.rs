use crate::{
    util::{InitializedCell, OnceCell},
    StreamSource,
};
use futures::{stream::Next, StreamExt as _};
use std::sync::Arc;

/// ```text
///   [T]                [T]                [T]
///    ↑                  ↑                  ↑
///   [segment][next] -> [segment][next] -> [segment][next]
///    ↑
/// [head]
/// ```
pub struct StrCharStream<S>
where
    S: StreamSource<Output = String>,
{
    source: S,
    head: Arc<OnceCell<Option<NodeInner>>>,
    offset: usize,
}

struct NodeInner {
    segment: String,
    next: Arc<OnceCell<Option<Self>>>,
}

pub struct Node {
    inner: InitializedCell<Option<NodeInner>>,
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
                        Some(NodeInner {
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
        todo!()
    }
}
