#[cfg(feature = "foreign")]
pub mod foreign;
mod never;
mod parse_result;
mod result;

pub use never::{Never, ShouldNever};
pub use parse_result::ParseResult;
pub use result::Result;

pub type ParserResult<S, P> =
    ParseResult<S, <P as Parser<S>>::Output, <P as Parser<S>>::Error, <P as Parser<S>>::Fault>;

pub trait Parser<S> {
    type Output;
    type Error;
    type Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self>;
}

pub trait Parse<S>: Sized {
    type Error;
    type Fault;
    fn parse(input: S) -> ParseResult<S, Self, Self::Error, Self::Fault>;
}

impl<S, O, E, F, T: Fn(S) -> ParseResult<S, O, E, F>> Parser<S> for T {
    type Output = O;
    type Error = E;
    type Fault = F;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        self(input)
    }
}

impl<S, O, E, F> Parser<S> for Box<dyn Parser<S, Output = O, Error = E, Fault = F>> {
    type Output = O;
    type Error = E;
    type Fault = F;

    fn parse(&self, input: S) -> ParseResult<S, O, E, F> {
        self.as_ref().parse(input)
    }
}

pub trait ParseStream<L: Location<Self::Segment>>: LocatableStream<L> + RewindStream {}

impl<L: Location<Self::Segment>, S: LocatableStream<L> + RewindStream> ParseStream<L> for S {}

pub trait Location<S: ?Sized>: Clone {
    fn create_start() -> Self;
    fn advance(self, segment: &S) -> Self;
}

pub trait LocatableStream<L>: Stream
where
    L: Location<<Self as Stream>::Segment>,
{
    fn location(&self, nth: usize) -> L;
}

pub trait IntoLocatable: Stream {
    type Locatable<L>: LocatableStream<L, Segment = Self::Segment>
    where
        L: Location<Self::Segment>;

    fn into_locatable<L>(self) -> Self::Locatable<L>
    where
        L: Location<Self::Segment>;
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
