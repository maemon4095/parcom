use std::sync::{Arc, Mutex};

use parcom_core::{SegmentIterator, Stream, StreamSegment};
pub use parcom_streams_core::*;

pub struct GenericStream<T, S: StreamSource<T>> {
    pair: Option<Arc<Mutex<(Node<T>, Node<T>)>>>,
    source: Arc<Mutex<S>>,
}

enum Node<T> {
    Uninit,
    Ready(Arc<ReadyNode<T>>),
    Terminal,
}

struct ReadyNode<T> {
    buf: Vec<T>,
    next: Mutex<Node<T>>,
}

impl<T, S: StreamSource<T>> Stream for GenericStream<T, S> {
    type Segment = [T];
    type Error = S::Error;

    type SegmentIter = Iter<T, S>;
    type Advance = Advance<T, S>;

    fn segments(&mut self) -> Self::SegmentIter {
        todo!()
    }

    fn advance(self, delta: <Self::Segment as StreamSegment>::Length) -> Self::Advance {
        todo!()
    }
}

struct Advance<T, S: StreamSource<T>> {}

struct Iter<T, S: StreamSource<T>> {
    pair: Arc<Mutex<(Node<T>, Node<T>)>>,
    source: Arc<Mutex<S>>,
}

impl<T, S: StreamSource<T>> SegmentIterator for Iter<T, S> {
    type Segment = [T];
    type Error = S::Error;
    type Next<'a>
        = Next<T, S>
    where
        Self: 'a;

    fn next(&mut self, size_hint: <Self::Segment as StreamSegment>::Length) -> Self::Next<'_> {
        todo!()
    }
}

struct Next<T, S: StreamSource<T>> {
    pair: Arc<Mutex<(Node<T>, Node<T>)>>,
    source: Arc<Mutex<S>>,
}
