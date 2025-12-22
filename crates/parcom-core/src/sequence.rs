mod segment;
mod segment_stream;

pub mod measured;

use std::future::{Future, IntoFuture};

pub use measured::MeasuredSequence;
pub use segment::SequenceSegment;
pub use segment_stream::SegmentStream;

use crate::buffer_writer::BufferWriter;

pub trait Sequence: Sized {
    type Segment: SequenceSegment + ?Sized;
    type Segments<'a>: SegmentStream<SegmentRef = &'a Self::Segment>
    where
        Self: 'a;
    type Advance: IntoFuture<Output = Self>;

    fn segments<'a>(&'a mut self) -> Self::Segments<'a>;
    fn advance(self, delta: <Self::Segment as SequenceSegment>::Length) -> Self::Advance;
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
    type Peek<'a>: 'a + Sequence<Segment = Self::Segment>
    where
        Self: 'a;

    fn peek(&mut self) -> Self::Peek<'_>;
}

pub trait SequenceSource: Sized {
    type Item;
    type Error;
    type Next<'a, C>: Future<Output = C::Result>
    where
        Self: 'a,
        C: 'a + SequenceControl<Item = Self::Item, Error = Self::Error>;

    fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
    where
        C: 'a + SequenceControl<Item = Self::Item, Error = Self::Error>;
}

pub trait SequenceControl {
    type Item;
    type Result;
    type Error;
    type Writer: BufferWriter<Item = Self::Item, Result = Self::Result, Error = Self::Error>;

    fn request_writer(self, min_capacity: usize) -> Self::Writer;
    fn cancel(self, err: Self::Error) -> Self::Result;
    fn finish(self) -> Self::Result;
}
