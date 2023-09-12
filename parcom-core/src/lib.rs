#[cfg(feature = "foreign")]
pub mod foreign;
mod never;
mod parse_result;
mod result;

pub use never::{Never, ShouldNever};
pub use parse_result::ParseResult;
pub use result::Result;
use std::fmt::Debug;

pub trait Parser<S> {
    type Output;
    type Error;
    type Fault;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error, Self::Fault>;
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

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error, Self::Fault> {
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

#[derive(Debug, Clone, Copy)]
pub enum Delta {
    Positive(usize),
    Negative(usize),
}

impl Delta {
    pub fn abs(self) -> usize {
        match self {
            Self::Positive(n) | Self::Negative(n) => n,
        }
    }
}

pub trait ParseRewindStream: ParseStream + RewindStream {}

impl<S: ParseStream + RewindStream> ParseRewindStream for S {}

pub trait Location: Ord {
    /// return self - rhs
    fn delta(&self, rhs: &Self) -> Delta;
}

pub trait ParseStream: Stream {
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
