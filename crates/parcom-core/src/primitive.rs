pub mod slice;
pub mod str;

use crate::{Never, SegmentIterator, StreamSegment};

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
    type Error = Never;
    type Next<'b>
        = std::future::Ready<Result<Option<&'b T>, Self::Error>>
    where
        Self: 'b;

    fn next(&mut self, _: <Self::Segment as StreamSegment>::Length) -> Self::Next<'_> {
        std::future::ready(Ok(self.me.take()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BytesDelta(usize);

impl BytesDelta {
    pub const ZERO: Self = Self(0);

    pub fn from_bytes(bytes: usize) -> Self {
        Self(bytes)
    }

    pub fn to_bytes(self) -> usize {
        self.0
    }

    pub fn from_char(char: char) -> Self {
        Self::from_bytes(char.len_utf8())
    }

    pub fn from_str(str: &str) -> Self {
        Self::from_bytes(str.len())
    }
}
