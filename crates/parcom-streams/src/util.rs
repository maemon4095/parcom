mod notify;
mod once_cell;

pub use notify::{Notified, Notify};
pub use once_cell::{InitializedSharedCell, OnceCell};
use parcom_core::SegmentIterator;

pub struct Nodes<'me, T: ?Sized> {
    me: Option<&'me T>,
}

impl<'me, T: ?Sized> Nodes<'me, T> {
    pub fn new(segment: &'me T) -> Self {
        Self { me: Some(segment) }
    }
}

impl<'me, T: ?Sized> SegmentIterator for Nodes<'me, T> {
    type Segment = T;
    type Node = &'me T;
    type Next = std::future::Ready<Option<Self::Node>>;

    fn next(&mut self, _: usize) -> Self::Next {
        std::future::ready(self.me.take())
    }
}

impl<'me, T: ?Sized> futures::Stream for Nodes<'me, T> {
    type Item = &'me T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(self.me.take())
    }
}
