mod segment;
mod segment_stream;

pub mod measured;

use std::future::{Future, IntoFuture};

pub use measured::MeasuredSequence;
pub use segment::SequenceSegment;
pub use segment_stream::SegmentStream;

pub trait Sequence: Sized {
    type Length;
    type Segment: ?Sized;
    type Segments<'a>: SegmentStream<Segment = Self::Segment, Length = Self::Length>
    where
        Self: 'a;
    type Advance: IntoFuture<Output = Self>;

    fn segments<'a>(&'a mut self) -> Self::Segments<'a>;
    fn advance(self, delta: Self::Length) -> Self::Advance;
}

pub trait ParseSequence: MeasuredSequence + RewindSequence {}

impl<S: MeasuredSequence + RewindSequence> ParseSequence for S {}

pub trait RewindSequence: Sequence {
    type Anchor;
    type Rewind: Future<Output = Self>;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind;
}

pub trait BindableSequence: MeasuredSequence {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}

pub trait PeekableSequence: Sequence {
    type Peek<'a>: 'a + Sequence<Segment = Self::Segment, Length = Self::Length>
    where
        Self: 'a;

    fn peek(&mut self) -> Self::Peek<'_>;
}
