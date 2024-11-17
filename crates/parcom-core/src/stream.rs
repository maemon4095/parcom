pub use crate::measured::*;
use std::{future::Future, ops::Deref};

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: ParcomStream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait ParcomSegmentStream<S: ?Sized>: futures::Stream<Item = Self::Node> + Unpin {
    type Node: Deref<Target = S>;
}

pub trait ParcomStreamSegment {
    type Offset;

    fn slice(&self, offset: Self::Offset) -> &Self;
    fn advance(&self, count: usize) -> Result<Self::Offset, usize>;
}

impl<S: ?Sized, N: Deref<Target = S>, B: Unpin + futures::Stream<Item = N>> ParcomSegmentStream<S>
    for B
{
    type Node = N;
}

pub trait ParcomStream: Sized {
    type Segment: ?Sized;
    type SegmentStream: ParcomSegmentStream<Self::Segment>;
    type Advance: Future<Output = Self>;

    fn segments(&self) -> Self::SegmentStream;
    fn advance(self, count: usize) -> Self::Advance;
}

pub trait BindableStream: MeasuredStream {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}
