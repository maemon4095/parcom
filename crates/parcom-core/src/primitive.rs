use crate::{ParcomSegmentIterator, ParcomStream, RewindStream};

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

impl<'a, T: ?Sized> ParcomSegmentIterator for Nodes<'a, T> {
    type Segment = T;
    type Node = &'a T;
    type Next = std::future::Ready<Option<Self::Node>>;

    fn next(&mut self, _: usize) -> Self::Next {
        std::future::ready(self.me.take())
    }
}

impl<'a> ParcomStream for &'a str {
    type Segment = str;
    type SegmentStream = Nodes<'a, str>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentStream {
        Nodes { me: Some(self) }
    }

    fn advance(self, count: usize) -> Self::Advance {
        let mut chars = self.chars();
        for _ in 0..count {
            chars.next();
        }
        std::future::ready(chars.as_str())
    }
}

impl RewindStream for &str {
    type Anchor = Anchor<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        let ptr = anchor.me.as_ptr();
        let len = anchor.me.len();
        let offset = unsafe { self.as_ptr().offset_from(ptr) };
        if !offset.is_negative() && (offset as usize) <= len {
            anchor.me
        } else {
            panic!("the anchor is not an anchor of this stream.")
        }
    }
}

impl<'a, T> ParcomStream for &'a [T] {
    type Segment = [T];
    type SegmentStream = Nodes<'a, [T]>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentStream {
        Nodes { me: Some(self) }
    }

    fn advance(self, count: usize) -> Self::Advance {
        std::future::ready(&self[count..])
    }
}

impl<T> RewindStream for &[T] {
    type Anchor = Anchor<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        let ptr = anchor.me.as_ptr();
        let len = anchor.me.len();
        let offset = unsafe { self.as_ptr().offset_from(ptr) };
        if !offset.is_negative() && (offset as usize) <= len {
            anchor.me
        } else {
            panic!("the anchor is not an anchor of this stream.")
        }
    }
}
