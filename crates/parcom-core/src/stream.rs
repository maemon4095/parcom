pub use crate::measured::*;

pub trait ParseStream: MeasuredStream + RewindStream {}

impl<S: MeasuredStream + RewindStream> ParseStream for S {}

pub trait RewindStream: ParcomStream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait ParcomStream: Sized {
    type Segment: ?Sized;

    fn segments(&self) -> impl Iterator<Item = &'_ Self::Segment>;
    fn advance(self, count: usize) -> Self;
}

pub trait BindableStream: MeasuredStream {
    fn bind<T>(self, location: Self::Location, item: T) -> Self;
    fn get<T>(&self, location: Self::Location) -> Option<&T>;
}
