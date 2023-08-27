#[cfg(feature = "standard")]
mod standard_extension;
#[cfg(feature = "streams")]
mod streams;
pub trait Parser<S: ParseStream> {
    type Output;
    type Error;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)>;
}

impl<S: ParseStream, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> Parser<S> for F {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self(input)
    }
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
    type Item;
    type Iter<'a>: 'a + Iterator<Item = &'a [Self::Item]>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_>;
    fn advance(self, count: usize) -> Self;
}

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
