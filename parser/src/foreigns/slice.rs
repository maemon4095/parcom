use crate::{ParseStream, RewindStream, Stream};

pub struct SliceStream<'me, T> {
    location: usize,
    slice: &'me [T],
}

impl<'me, T> SliceStream<'me, T> {
    pub fn new(slice: &'me [T]) -> Self {
        Self { location: 0, slice }
    }
}

impl<'me, T> Clone for SliceStream<'me, T> {
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            slice: self.slice.clone(),
        }
    }
}

impl<'me, T> Stream for SliceStream<'me, T> {
    type Segment = [T];

    type Iter<'a> = std::iter::Once<&'a [T]>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self.slice)
    }

    fn advance(mut self, count: usize) -> Self {
        self.location = self.location(count);
        self.slice = &self.slice[count..];
        self
    }
}
impl<'me, T> RewindStream for SliceStream<'me, T> {
    type Anchor = Self;

    fn anchor(&self) -> Self::Anchor {
        self.clone()
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor
    }
}
impl<'me, T> ParseStream for SliceStream<'me, T> {
    type Location = usize;

    fn location(&self, index: usize) -> Self::Location {
        self.slice.iter().take(index + 1).count()
    }
}
