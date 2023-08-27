use std::marker::PhantomData;

use crate::{ParseStream, Parser};

pub struct Map<T: ParseStream, P: Parser<T>, U, F: Fn(P::Output) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<S: ParseStream, P: Parser<S>, U, F: Fn(P::Output) -> U> Parser<S> for Map<S, P, U, F> {
    type Output = U;
    type Error = P::Error;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.parser
            .parse(input)
            .map(|(e, r)| ((self.mapping)(e), r))
    }
}

pub struct MapErr<T: ParseStream, P: Parser<T>, U, F: Fn(P::Error) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<S: ParseStream, P: Parser<S>, U, F: Fn(P::Error) -> U> Parser<S> for MapErr<S, P, U, F> {
    type Output = P::Output;
    type Error = U;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.parser
            .parse(input)
            .map_err(|(e, r)| ((self.mapping)(e), r))
    }
}
