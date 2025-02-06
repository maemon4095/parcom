pub use crate::measured::*;
use std::{future::Future, ops::Deref};

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: Stream {
    type Anchor;
    type Rewind: std::future::Future<Output = Self>;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind;
}

pub trait SegmentIterator: Unpin {
    type Segment: ?Sized + StreamSegment;
    type Node: Deref<Target = Self::Segment>;
    type Next: Future<Output = Option<Self::Node>>;

    fn next(&mut self, size_hint: <Self::Segment as StreamSegment>::Delta) -> Self::Next;
}

pub trait StreamSegment {
    type Delta: Default + std::cmp::Ord;

    fn len(&self) -> Self::Delta;
    fn split_at(&self, mid: Self::Delta) -> (&Self, &Self);
}

pub trait Stream: Sized {
    type Segment: StreamSegment + ?Sized;
    type SegmentIter: SegmentIterator<Segment = Self::Segment>;
    type Advance: Future<Output = Self>;

    fn segments(&self) -> Self::SegmentIter;
    fn advance(self, delta: <Self::Segment as StreamSegment>::Delta) -> Self::Advance;
}

pub trait BindableStream: MeasuredStream {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}
