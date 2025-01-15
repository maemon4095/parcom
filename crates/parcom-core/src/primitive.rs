use crate::{RewindStream, SegmentIterator, Stream, StreamSegment};

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

impl<'a> Stream for &'a str {
    type Segment = str;
    type SegmentIter = Nodes<'a, str>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentIter {
        Nodes { me: Some(self) }
    }

    fn advance(self, delta: BytesDelta) -> Self::Advance {
        let delta: usize = delta.into();

        let rest = self.get(delta..).unwrap_or("");
        std::future::ready(rest)
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
}

impl RewindStream for &str {
    type Anchor = Anchor<Self>;
    type Rewind = std::future::Ready<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        let ptr = anchor.me.as_ptr();
        let len = anchor.me.len();
        let offset = unsafe { self.as_ptr().offset_from(ptr) };
        if !offset.is_negative() && (offset as usize) <= len {
            std::future::ready(anchor.me)
        } else {
            panic!("the anchor is not an anchor of this stream.")
        }
    }
}

impl<'a, T> Stream for &'a [T] {
    type Segment = [T];
    type SegmentIter = Nodes<'a, [T]>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentIter {
        Nodes { me: Some(self) }
    }

    fn advance(self, count: usize) -> Self::Advance {
        std::future::ready(&self[count..])
    }
}

impl<T> RewindStream for &[T] {
    type Anchor = Anchor<Self>;
    type Rewind = std::future::Ready<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        let ptr = anchor.me.as_ptr();
        let len = anchor.me.len();
        let offset = unsafe { self.as_ptr().offset_from(ptr) };
        if !offset.is_negative() && (offset as usize) <= len {
            std::future::ready(anchor.me)
        } else {
            panic!("the anchor is not an anchor of this stream.")
        }
    }
}

impl<T> StreamSegment for [T] {
    type Delta = usize;
}
