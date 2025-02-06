mod notify;

pub use notify::{Notified, Notify};
use parcom_core::{SegmentIterator, StreamSegment};

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
    type Node = &'me T;
    type Next = std::future::Ready<Option<Self::Node>>;

    fn next(&mut self, _: T::Delta) -> Self::Next {
        std::future::ready(self.me.take())
    }
}
