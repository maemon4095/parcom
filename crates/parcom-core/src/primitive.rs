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

impl<'a, T: ?Sized + StreamSegment> SegmentIterator for Nodes<'a, T> {
    type Segment = T;
    type Node = &'a T;
    type Next = std::future::Ready<Option<Self::Node>>;

    fn next(&mut self, _: T::Delta) -> Self::Next {
        std::future::ready(self.me.take())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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
