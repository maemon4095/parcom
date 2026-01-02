use std::future::Future;

use crate::{ParseError, ParseResult, ParserResult, Sequence};

pub trait Parser<S: Sequence>: ParserOnce<S> {
    fn parse(&self, input: S) -> impl Future<Output = ParserResult<S, Self>>;
}

impl<S: Sequence, O, E: ParseError, Fut, T: Fn(S) -> Fut> Parser<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E>>,
{
    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, O, E>> {
        self(input)
    }
}

impl<S: Sequence, O, E: ParseError, Fut, T: FnOnce(S) -> Fut> ParserOnce<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E>>,
{
    type Output = O;
    type Error = E;

    fn parse_once(self, input: S) -> impl Future<Output = ParseResult<S, O, E>> {
        self(input)
    }
}

pub trait ParserOnce<S: Sequence> {
    type Output;
    type Error: ParseError;

    fn parse_once(self, input: S) -> impl Future<Output = ParserResult<S, Self>>;
}

pub trait IterativeParserState<S: Sequence>: Sized {
    type Output;
    type Error: ParseError;

    fn parse_next(
        &mut self,
        input: S,
    ) -> impl Future<Output = ParseResult<S, Option<Self::Output>, Self::Error>>;
}

pub trait IterativeParserOnce<S: Sequence> {
    type Output;
    type Error: ParseError;
    type StateOnce: IterativeParserState<S, Output = Self::Output, Error = Self::Error>;

    fn parse_iterative_once(self) -> Self::StateOnce;
}

pub trait IterativeParser<S: Sequence>: IterativeParserOnce<S> {
    type State<'a>: IterativeParserState<S, Output = Self::Output, Error = Self::Error>
    where
        Self: 'a;

    fn parse_iterative(&self) -> Self::State<'_>;
}

impl<S: Sequence, P: IterativeParser<S>> IterativeParser<S> for &P {
    type State<'a>
        = P::State<'a>
    where
        Self: 'a;
    fn parse_iterative(&self) -> Self::State<'_> {
        P::parse_iterative(self)
    }
}

impl<'a, S: Sequence, P: IterativeParser<S>> IterativeParserOnce<S> for &'a P {
    type Output = P::Output;
    type Error = P::Error;
    type StateOnce = P::State<'a>;

    fn parse_iterative_once(self) -> Self::StateOnce {
        P::parse_iterative(&self)
    }
}
