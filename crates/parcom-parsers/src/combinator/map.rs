use parcom_core::{ParseError, Parser, ParserOnce, ParserResult, Stream};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Map<S: Stream, P: ParserOnce<S>, U, F: FnOnce(P::Output) -> U> {
    parser: P,
    mapping: F,
    marker: PhantomData<(S, U)>,
}

impl<S: Stream, P: ParserOnce<S>, U, F: FnOnce(P::Output) -> U> Map<S, P, U, F> {
    pub fn new(parser: P, mapping: F) -> Self {
        Self {
            parser,
            mapping,
            marker: PhantomData,
        }
    }
}

impl<S: Stream, P: ParserOnce<S>, U, F: FnOnce(P::Output) -> U> ParserOnce<S> for Map<S, P, U, F> {
    type Output = U;
    type Error = P::Error;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parser.parse_once(input).await.map(self.mapping)
    }
}
impl<S: Stream, P: Parser<S>, U, F: Fn(P::Output) -> U> Parser<S> for Map<S, P, U, F> {
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).await.map(&self.mapping)
    }
}

#[derive(Debug)]
pub struct MapErr<S: Stream, P: ParserOnce<S>, U: ParseError, F: FnOnce(P::Error) -> U> {
    parser: P,
    mapping: F,
    marker: PhantomData<(S, U)>,
}

impl<S: Stream, P: ParserOnce<S>, U: ParseError, F: Fn(P::Error) -> U> MapErr<S, P, U, F> {
    pub fn new(parser: P, mapping: F) -> Self {
        Self {
            parser,
            mapping,
            marker: PhantomData,
        }
    }
}

impl<S: Stream, P: ParserOnce<S>, U: ParseError, F: FnOnce(P::Error) -> U> ParserOnce<S>
    for MapErr<S, P, U, F>
{
    type Output = P::Output;
    type Error = U;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parser.parse_once(input).await.map_err(self.mapping)
    }
}

impl<S: Stream, P: Parser<S>, U: ParseError, F: Fn(P::Error) -> U> Parser<S>
    for MapErr<S, P, U, F>
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).await.map_err(&self.mapping)
    }
}
