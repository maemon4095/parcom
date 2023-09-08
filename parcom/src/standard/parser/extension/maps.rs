use std::marker::PhantomData;

use crate::{ParseResult, Parser, RewindStream};

pub struct Map<T: RewindStream, P: Parser<T>, U, F: Fn(P::Output) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<S: RewindStream, P: Parser<S>, U, F: Fn(P::Output) -> U> Parser<S> for Map<S, P, U, F> {
    type Output = U;
    type Error = P::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        self.parser.parse(input).map(&self.mapping)
    }
}

pub struct MapErr<T: RewindStream, P: Parser<T>, U, F: Fn(P::Error) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<S: RewindStream, P: Parser<S>, U, F: Fn(P::Error) -> U> Parser<S> for MapErr<S, P, U, F> {
    type Output = P::Output;
    type Error = U;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        self.parser.parse(input).map_err(&self.mapping)
    }
}
