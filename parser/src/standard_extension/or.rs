use std::marker::PhantomData;

use crate::{Parser, RewindStream};

use super::Either;

pub struct Or<T: RewindStream, P0: Parser<T>, P1: Parser<T>> {
    pub(super) parser0: P0,
    pub(super) parser1: P1,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P0: Parser<S>, P1: Parser<S>> Parser<S> for Or<S, P0, P1> {
    type Output = Either<P0::Output, P1::Output>;
    type Error = (P0::Error, P1::Error);

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        let anchor = input.anchor();

        let (err0, rest) = match self.parser0.parse(input) {
            Ok((v, r)) => return Ok((Either::First(v), r)),
            Err(t) => t,
        };
        let input = rest.rewind(anchor);

        let (err1, rest) = match self.parser1.parse(input) {
            Ok((v, r)) => return Ok((Either::Last(v), r)),
            Err(t) => t,
        };

        Err(((err0, err1), rest))
    }
}
