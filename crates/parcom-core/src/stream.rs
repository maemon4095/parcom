pub mod measured;

use std::{future::Future, ops::Deref};

pub use measured::MeasuredStream;

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: Stream {
    type Anchor;
    type Rewind: std::future::Future<Output = Self>;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind;
}

pub trait StreamSegment {
    type Length: Default + std::cmp::Ord;

    fn len(&self) -> Self::Length;
    fn split_at(&self, mid: Self::Length) -> (&Self, &Self);
}

pub trait SegmentIterator {
    type Segment: StreamSegment + ?Sized;
    type Node: Deref<Target = Self::Segment>;
    type Error;
    type Next<'a>: 'a + std::future::Future<Output = Option<Result<Self::Node, Self::Error>>>
    where
        Self: 'a;

    fn next(&mut self, size_hint: <Self::Segment as StreamSegment>::Length) -> Self::Next<'_>;
}

pub trait Stream: Sized {
    type Segment: StreamSegment + ?Sized;
    type Error;

    type SegmentIter: SegmentIterator<Segment = Self::Segment, Error = Self::Error>;
    type Advance: Future<Output = Self>;

    fn segments(&self) -> Self::SegmentIter;
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

    fn peek(&self) -> Self::Peek<'_>;
}
