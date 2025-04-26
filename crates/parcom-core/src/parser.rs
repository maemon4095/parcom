use std::future::Future;

use crate::{ParseError, ParseResult, ParserResult, Stream};

pub trait Parser<S: Stream>: ParserOnce<S> {
    fn parse(&self, input: S) -> impl Future<Output = ParserResult<S, Self>>;
}

impl<S: Stream, O, E: ParseError, Fut, T: Fn(S) -> Fut> Parser<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E>>,
{
    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, O, E>> {
        self(input)
    }
}

impl<S: Stream, O, E: ParseError, Fut, T: FnOnce(S) -> Fut> ParserOnce<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E>>,
{
    type Output = O;
    type Error = E;

    fn parse_once(self, input: S) -> impl Future<Output = ParseResult<S, O, E>> {
        self(input)
    }
}

pub trait ParserOnce<S: Stream> {
    type Output;
    type Error: ParseError;

    fn parse_once(self, input: S) -> impl Future<Output = ParserResult<S, Self>>;
}

pub trait IterativeParserState<S: Stream>: Sized {
    type Output;
    type Error: ParseError;

    fn parse_next(
        &mut self,
        input: S,
    ) -> impl Future<Output = ParseResult<S, Option<Self::Output>, Self::Error>>;
}

pub trait IterativeParserOnce<S: Stream> {
    type Output;
    type Error: ParseError;
    type StateOnce: IterativeParserState<S, Output = Self::Output, Error = Self::Error>;

    fn start_once(self) -> Self::StateOnce;
}

pub trait IterativeParser<S: Stream>: IterativeParserOnce<S> {
    type State<'a>: IterativeParserState<S, Output = Self::Output, Error = Self::Error>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_>;
}

impl<S: Stream, P: IterativeParser<S>> IterativeParser<S> for &P {
    type State<'a>
        = P::State<'a>
    where
        Self: 'a;
    fn start(&self) -> Self::State<'_> {
        P::start(self)
    }
}

impl<'a, S: Stream, P: IterativeParser<S>> IterativeParserOnce<S> for &'a P {
    type Output = P::Output;
    type Error = P::Error;
    type StateOnce = P::State<'a>;

    fn start_once(self) -> Self::StateOnce {
        P::start(&self)
    }
}
