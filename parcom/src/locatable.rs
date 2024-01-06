pub mod measure;

use crate::{LocatableStream, Location, Stream};

pub struct Locatable<S, L>
where
    S::Segment: SliceLike,
    S: Stream,
    L: Location<S::Segment>,
{
    location: L,
    base: S,
}

impl<S, L> Locatable<S, L>
where
    S::Segment: SliceLike,
    S: Stream,
    L: Location<S::Segment>,
{
    pub fn new(base: S) -> Self {
        Self {
            location: L::create_start(),
            base,
        }
    }
}

impl<S, L> Stream for Locatable<S, L>
where
    S::Segment: SliceLike,
    S: Stream,
    L: Location<S::Segment>,
{
    type Segment = S::Segment;

    type Iter<'a> = S::Iter<'a>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        self.base.segments()
    }

    fn advance(mut self, count: usize) -> Self {
        self.location = self.location(count);
        self.base = self.base.advance(count);
        self
    }
}

impl<S, L> LocatableStream<L> for Locatable<S, L>
where
    S::Segment: SliceLike,
    S: Stream,
    L: Location<S::Segment>,
{
    fn location(&self, nth: usize) -> L {
        let mut remain = nth;
        let mut location = self.location.clone();

        for segment in self.segments() {
            if remain <= segment.len() {
                location = location.advance(segment.slice(..remain));
                break;
            } else {
                location = location.advance(segment);
                remain -= segment.len();
            }
        }

        location
    }
}

pub trait SliceLike {
    fn len(&self) -> usize;
    fn slice(&self, range: std::ops::RangeTo<usize>) -> &Self;
}

impl<T> SliceLike for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn slice(&self, range: std::ops::RangeTo<usize>) -> &Self {
        &self[range]
    }
}

impl SliceLike for str {
    fn len(&self) -> usize {
        str::len(self)
    }

    fn slice(&self, range: std::ops::RangeTo<usize>) -> &Self {
        &self[range]
    }
}
