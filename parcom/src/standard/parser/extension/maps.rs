use std::marker::PhantomData;

use crate::{Parser, ParserResult, RewindStream};

pub struct Map<T: RewindStream, P: Parser<T>, U, F: Fn(P::Output) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<S: RewindStream, P: Parser<S>, U, F: Fn(P::Output) -> U> Parser<S> for Map<S, P, U, F> {
    type Output = U;
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
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
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).map_err(&self.mapping)
    }
}

pub struct MapFault<T: RewindStream, P: Parser<T>, U, F: Fn(P::Fault) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<S: RewindStream, P: Parser<S>, U, F: Fn(P::Fault) -> U> Parser<S> for MapFault<S, P, U, F> {
    type Output = P::Output;
    type Error = P::Error;
    type Fault = U;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).map_fault(&self.mapping)
    }
}
