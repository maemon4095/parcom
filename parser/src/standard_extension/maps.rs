use std::marker::PhantomData;

use crate::Parser;

pub struct Map<T, P: Parser<T>, U, F: Fn(P::Output) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<T, P: Parser<T>, U, F: Fn(P::Output) -> U> Parser<T> for Map<T, P, U, F> {
    type Output = U;
    type Error = P::Error;

    fn parse<S: crate::ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.parser
            .parse(input)
            .map(|(e, r)| ((self.mapping)(e), r))
    }
}

pub struct MapErr<T, P: Parser<T>, U, F: Fn(P::Error) -> U> {
    pub(super) parser: P,
    pub(super) mapping: F,
    pub(super) marker: PhantomData<(T, U)>,
}

impl<T, P: Parser<T>, U, F: Fn(P::Error) -> U> Parser<T> for MapErr<T, P, U, F> {
    type Output = P::Output;
    type Error = U;

    fn parse<S: crate::ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.parser
            .parse(input)
            .map_err(|(e, r)| ((self.mapping)(e), r))
    }
}
