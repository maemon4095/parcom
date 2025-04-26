mod notify;

pub use notify::{Notified, Notify};
use parcom_core::{Never, SegmentIterator, StreamSegment};

pub struct Nodes<'me, T: ?Sized> {
    me: Option<&'me T>,
}

impl<'me, T: ?Sized> Nodes<'me, T> {
    pub fn new(segment: &'me T) -> Self {
        Self { me: Some(segment) }
    }
}

impl<'me, T: ?Sized + StreamSegment> SegmentIterator for Nodes<'me, T> {
    type Segment = T;
    type Error = Never;
    type Node = &'me T;
    type Next<'a>
        = std::future::Ready<Option<Result<Self::Node, Never>>>
    where
        Self: 'a;

    fn next(&mut self, _: T::Length) -> Self::Next<'_> {
        std::future::ready(self.me.take().map(Ok))
    }
}
