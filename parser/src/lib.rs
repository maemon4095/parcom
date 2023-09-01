#[cfg(feature = "foreign")]
pub mod foreign;
#[cfg(feature = "packrat")]
pub mod packrat;
#[cfg(feature = "standard")]
pub mod standard;
#[cfg(feature = "stream")]
pub mod stream;

pub type ParseResult<S, P> = Result<(<P as Parser<S>>::Output, S), (<P as Parser<S>>::Error, S)>;

pub trait Parser<S> {
    type Output;
    type Error;

    fn parse(&self, input: S) -> ParseResult<S, Self>;
}

impl<S, P: Parser<S>> Parser<S> for &P {
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        P::parse(self, input)
    }
}

impl<S, O, E> Parser<S> for Box<dyn Parser<S, Output = O, Error = E>> {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.as_ref().parse(input)
    }
}

pub trait Location: Ord {
    fn distance(&self, rhs: &Self) -> usize;
}

pub trait ParseStream: RewindStream {
    type Location: Location;
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
