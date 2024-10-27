pub use crate::measured::*;
use std::future::Future;

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: ParcomStream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait ParcomNodeStream<S: ?Sized>: futures::Stream<Item = Self::Node> + Unpin {
    type Node: AsRef<S>;
}

impl<S: ?Sized, N: AsRef<S>, B: Unpin + futures::Stream<Item = N>> ParcomNodeStream<S> for B {
    type Node = N;
}

pub trait ParcomStream: Sized {
    type Segment: ?Sized;
    type Nodes: ParcomNodeStream<Self::Segment>;
    type Advance: Future<Output = Self>;

    fn nodes(&self) -> Self::Nodes;
    fn advance(self, count: usize) -> Self::Advance;
}

pub trait BindableStream: MeasuredStream {
    fn bind<T>(self, location: Self::Metrics, item: T) -> Self;
    fn get<T>(&self, location: Self::Metrics) -> Option<&T>;
}
