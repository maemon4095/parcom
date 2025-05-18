pub mod measured;

use std::future::Future;

pub use measured::MeasuredStream;

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: Stream {
    type Anchor;
    type Rewind: Future<Output = Result<Self, Self::Error>>;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind;
}

pub trait StreamSegment: 'static {
    type Length: Default + std::cmp::Ord;

    fn len(&self) -> Self::Length;
    fn split_at(&self, mid: Self::Length) -> (&Self, &Self);
}

pub trait SegmentIterator {
    type Segment: StreamSegment + ?Sized;
    type Error;

    type Next<'a>: Future<Output = Result<Option<&'a Self::Segment>, Self::Error>>
    where
        Self: 'a;

    fn next(&mut self, size_hint: <Self::Segment as StreamSegment>::Length) -> Self::Next<'_>;
}

pub trait Stream: Sized {
    type Segment: StreamSegment + ?Sized;
    type Error;

    type SegmentIter<'a>: SegmentIterator<Segment = Self::Segment, Error = Self::Error>
    where
        Self: 'a;
    type Advance: Future<Output = Result<Self, Self::Error>>;

    fn segments(&mut self) -> Self::SegmentIter<'_>;
    fn advance(self, delta: <Self::Segment as StreamSegment>::Length) -> Self::Advance;
}

pub trait BindableStream: MeasuredStream {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}

pub trait PeekableStream: Stream {
    type Peek<'a>: 'a + Stream<Segment = Self::Segment, Error = Self::Error>
    where
        Self: 'a;

    fn peek(&mut self) -> Self::Peek<'_>;
}
