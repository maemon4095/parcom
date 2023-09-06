use parcom_core::Delta;

use crate::{ParseStream, RewindStream, Stream};

pub struct SliceStream<'me, T> {
    location: Location,
    slice: &'me [T],
}

impl<'me, T> SliceStream<'me, T> {
    pub fn new(slice: &'me [T]) -> Self {
        Self {
            location: Location { index: 0 },
            slice,
        }
    }

    fn loc(&self, count: usize) -> Location {
        Location {
            index: self.location.index + self.slice.iter().take(count).count(),
        }
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
        self.location = self.loc(count);
        self.slice = &self.slice[count..];
        self
    }
}
impl<'me, T> RewindStream for SliceStream<'me, T> {
    type Anchor = Anchor<'me, T>;

    fn anchor(&self) -> Self::Anchor {
        Anchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}
impl<'me, T> ParseStream for SliceStream<'me, T> {
    type Location = Location;

    fn location(&self, index: usize) -> Self::Location {
        self.loc(index + 1)
    }
}

pub struct Anchor<'me, T> {
    stream: SliceStream<'me, T>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    index: usize,
}

impl crate::Location for Location {
    fn delta(&self, rhs: &Self) -> Delta {
        if self.index < rhs.index {
            Delta::Negative(rhs.index - self.index)
        } else {
            Delta::Positive(self.index - rhs.index)
        }
    }
}
