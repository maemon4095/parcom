use std::future::Future;

use crate::{ParseError, ParseResult};

pub trait Parser<S> {
    type Output;
    type Error: ParseError;

    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, Self::Output, Self::Error>>;
}

impl<S, O, E: ParseError, Fut, T: Fn(S) -> Fut> Parser<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E>>,
{
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, O, E>> {
        self(input)
    }
}

pub trait IterativeParserState<S>: Sized {
    type Output;
    type Error: ParseError;

    fn parse_next(
        &mut self,
        input: S,
    ) -> impl Future<Output = ParseResult<S, Result<Self::Output, Self::Error>, Self::Error>>;
}

pub trait IterativeParser<S> {
    type Output;
    type Error: ParseError;

    type State<'a>: IterativeParserState<S, Output = Self::Output, Error = Self::Error>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_>;
}

impl<S, P: IterativeParser<S>> IterativeParser<S> for &P {
    type Output = P::Output;
    type Error = P::Error;
    type State<'a>
        = P::State<'a>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        P::start(self)
    }
}
