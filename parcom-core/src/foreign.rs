use core::panic;

use crate::{RewindStream, Stream, StreamSegment};

pub struct Anchor<T> {
    me: T,
}

impl StreamSegment for str {
    type Item<'a> = char;

    type Iter<'a> = std::str::Chars<'a>;

    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl Stream for &str {
    type Segment = str;

    type Iter<'a> = std::iter::Once<&'a str>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self)
    }

    fn advance(self, count: usize) -> Self {
        let mut chars = self.chars();
        for _ in 0..count {
            chars.next();
        }
        chars.as_str()
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

impl<T> StreamSegment for [T] {
    type Item<'a> = &'a T where T:'a;
    type Iter<'a> = std::slice::Iter<'a, T> where T:'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }
}

impl<T> Stream for &[T] {
    type Segment = [T];

    type Iter<'a> = std::iter::Once<&'a [T]>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self)
    }

    fn advance(self, count: usize) -> Self {
        &self[count..]
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
