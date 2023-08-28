mod slice;
mod str;

use crate::{RewindStream, Stream};

pub use slice::SliceStream;
pub use str::StrStream;

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
        chars.nth(count);
        chars.as_str()
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
    type Anchor = Self;

    fn anchor(&self) -> Self::Anchor {
        self
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor
    }
}

impl RewindStream for &str {
    type Anchor = Self;

    fn anchor(&self) -> Self::Anchor {
        self
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor
    }
}
