mod node;

use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    process::Output,
    sync::{
        atomic::{AtomicIsize, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use node::{ArcNode, Node};
use parcom_core::{SegmentIterator, Stream, StreamSegment};

use crate::util::Notify;

use super::StreamSource;

#[derive(Clone)]
pub struct GenericStream<S: StreamSource> {
    inner: Arc<InnerStream<S>>,
}

#[derive(Debug)]
struct InnerStream<S: StreamSource> {
    source: S,
    on_append: Notify,
    head: ArcNode<S::Output>,
}

impl<S: StreamSource> GenericStream<S> {
    pub fn new(source: S) -> Self {
        Self {
            inner: Arc::new(InnerStream {
                source,
                on_append: Notify::new(),
                head: ArcNode::new(),
            }),
        }
    }
}

struct Segments<S: StreamSource> {
    stream: Arc<InnerStream<S>>,
    offset: usize,
    node: Option<ArcNode<S::Output>>,
}
// impl ParcomSegmentIterator for Segments where S::Item: ParcomStreamNode

pub trait StreamNode: Sized + Deref<Target = Self::Segment> {
    type Segment: ?Sized + StreamSegment;

    fn advance(
        self,
        delta: <Self::Segment as StreamSegment>::Length,
    ) -> Result<Self, <Self::Segment as StreamSegment>::Length>;
}
