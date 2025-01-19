pub mod slice;
pub mod str;

use crate::{SegmentIterator, StreamSegment};

pub struct Anchor<T> {
    me: T,
}

pub struct Node<'a, T: ?Sized> {
    me: &'a T,
}
impl<'a, T: ?Sized> AsRef<T> for Node<'a, T> {
    fn as_ref(&self) -> &T {
        self.me
    }
}

pub struct Nodes<'a, T: ?Sized> {
    me: Option<&'a T>,
}

impl<'a, T: ?Sized> SegmentIterator for Nodes<'a, T> {
    type Segment = T;
    type Node = &'a T;
    type Next = std::future::Ready<Option<Self::Node>>;

    fn next(&mut self, _: usize) -> Self::Next {
        std::future::ready(self.me.take())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytesDelta(usize);

impl From<usize> for BytesDelta {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Into<usize> for BytesDelta {
    fn into(self) -> usize {
        self.0
    }
}

impl StreamSegment for str {
    type Delta = BytesDelta;

    fn slice(&self, delta: Self::Delta) -> &Self {
        &self[delta.0..]
    }
}
