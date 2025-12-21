mod segment;
mod segment_stream;

pub mod measured;

use std::future::{Future, IntoFuture};

pub use measured::MeasuredSequence;
pub use segment::SequenceSegment;
pub use segment_stream::SegmentStream;

pub trait Sequence: Sized {
    type Segment: SequenceSegment + ?Sized;
    type Error;

    type Segments<'a>: SegmentStream<SegmentRef = &'a Self::Segment, Error = Self::Error>
    where
        Self: 'a;
    type Advance: IntoFuture<Output = Result<Self, Self::Error>>;

    fn segments<'a>(&'a mut self) -> Self::Segments<'a>;
    fn advance(self, delta: <Self::Segment as SequenceSegment>::Length) -> Self::Advance;
}

pub trait ParseSequence: MeasuredSequence + RewindSequence {}

impl<S: MeasuredSequence + RewindSequence> ParseSequence for S {}

pub trait RewindSequence: Sequence {
    type Anchor;
    type Rewind: Future<Output = Result<Self, Self::Error>>;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind;
}

pub trait BindableSequence: MeasuredSequence {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}

pub trait PeekableSequence: Sequence {
    type Peek<'a>: 'a + Sequence<Segment = Self::Segment, Error = Self::Error>
    where
        Self: 'a;

    fn peek(&mut self) -> Self::Peek<'_>;
}
