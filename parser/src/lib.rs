pub mod foreigns;
#[cfg(feature = "standard")]
pub mod standard_extension;
#[cfg(feature = "streams")]
pub mod streams;
pub trait Parser<S> {
    type Output;
    type Error;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)>;
}

pub trait ParseStream: RewindStream {
    type Location: Ord;
    fn location(&self, index: usize) -> Self::Location;
}

pub trait RewindStream: Stream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait Stream {
    type Segment: ?Sized;
    type Iter<'a>: 'a + Iterator<Item = &'a Self::Segment>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_>;
    fn advance(self, count: usize) -> Self;
}

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
