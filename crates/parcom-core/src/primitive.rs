use core::panic;

use crate::{ParcomStream, RewindStream};

pub struct Anchor<T> {
    me: T,
}

impl ParcomStream for &str {
    type Segment = str;

    fn segments(&self) -> impl Iterator<Item = &Self::Segment> {
        std::iter::once(*self)
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

impl<T> ParcomStream for &[T] {
    type Segment = [T];

    fn segments(&self) -> impl Iterator<Item = &Self::Segment> {
        std::iter::once(*self)
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
