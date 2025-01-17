use parcom_core::{ParseError, Parser, ParserResult};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Map<T, P: Parser<T>, U, F: Fn(P::Output) -> U> {
    parser: P,
    mapping: F,
    marker: PhantomData<(T, U)>,
}

impl<T, P: Parser<T>, U, F: Fn(P::Output) -> U> Map<T, P, U, F> {
    pub fn new(parser: P, mapping: F) -> Self {
        Self {
            parser,
            mapping,
            marker: PhantomData,
        }
    }
}

impl<S, P: Parser<S>, U, F: Fn(P::Output) -> U> Parser<S> for Map<S, P, U, F> {
    type Output = U;
    type Error = P::Error;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).await.map(&self.mapping)
    }
}

#[derive(Debug)]
pub struct MapErr<S, P: Parser<S>, U: ParseError, F: Fn(P::Error) -> U> {
    parser: P,
    mapping: F,
    marker: PhantomData<(S, U)>,
}

impl<S, P: Parser<S>, U: ParseError, F: Fn(P::Error) -> U> MapErr<S, P, U, F> {
    pub fn new(parser: P, mapping: F) -> Self {
        Self {
            parser,
            mapping,
            marker: PhantomData,
        }
    }
}

impl<S, P: Parser<S>, U: ParseError, F: Fn(P::Error) -> U> Parser<S> for MapErr<S, P, U, F> {
    type Output = P::Output;
    type Error = U;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).await.map_err(&self.mapping)
    }
}
