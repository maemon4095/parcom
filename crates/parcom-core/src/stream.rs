pub use crate::measured::*;
use std::{future::Future, ops::Deref};

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: ParcomStream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait ParcomSegmentIterator: Unpin {
    type Segment: ?Sized;
    type Node: Deref<Target = Self::Segment>;
    type Next: Future<Output = Option<Self::Node>>;

    fn next(&mut self, size_hint: usize) -> Self::Next;
}

pub trait ParcomStreamSegment {
    type Offset;

    fn slice(&self, offset: Self::Offset) -> &Self;
    fn advance(&self, count: usize) -> Result<Self::Offset, usize>;
}

pub trait ParcomStream: Sized {
    type Segment: ?Sized;
    type SegmentStream: ParcomSegmentIterator<Segment = Self::Segment>;
    type Advance: Future<Output = Self>;

    fn segments(&self) -> Self::SegmentStream;
    fn advance(self, count: usize) -> Self::Advance;
}

pub trait BindableStream: MeasuredStream {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}
