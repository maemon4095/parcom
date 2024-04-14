mod never;
mod parse_result;
mod unknown;

pub mod primitive;

pub use never::{Never, ShouldNever, ShouldNeverExtension};
pub use parse_result::ParseResult;
pub use unknown::UnknownLocation;

pub type ParserResult<S, P> =
    ParseResult<S, <P as Parser<S>>::Output, <P as Parser<S>>::Error, <P as Parser<S>>::Fault>;

pub trait Parser<S> {
    type Output;
    type Error;
    type Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self>;
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
    fn advance(self, segment: &S) -> Self;
}

pub trait LocatableStream<L>: Stream
where
    L: Location<<Self as Stream>::Segment>,
{
    fn location(&self, nth: usize) -> L;
}

pub trait IntoLocatable: Sized + Stream {
    type Locatable<L>: LocatableStream<L, Segment = Self::Segment>
    where
        L: Location<Self::Segment>;

    fn into_locatable_at<L>(self, location: L) -> Self::Locatable<L>
    where
        L: Location<Self::Segment>;

    fn into_locatable<L>(self) -> Self::Locatable<L>
    where
        L: Location<Self::Segment> + std::default::Default,
    {
        self.into_locatable_at(L::default())
    }
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

pub trait BindableStream<L>: LocatableStream<L>
where
    L: Location<Self::Segment>,
{
    fn bind<T>(self, location: L, item: T) -> Self;
    fn get<T>(&self, location: L) -> Option<&T>;
}
