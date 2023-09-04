use std::marker::PhantomData;

use parcom_core::ParseResult;

use crate::{Parser, RewindStream};

use super::super::Either;

pub struct Join<S: RewindStream, P0: Parser<S>, P1: Parser<S>> {
    pub(super) parser0: P0,
    pub(super) parser1: P1,
    pub(super) marker: PhantomData<S>,
}

impl<S: RewindStream, P0: Parser<S>, P1: Parser<S>> Parser<S> for Join<S, P0, P1> {
    type Output = (P0::Output, P1::Output);
    type Error = Either<P0::Error, P1::Error>;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let (item0, rest) = match self.parser0.parse(input) {
            Ok(t) => t,
            Err((e, r)) => return Err((Either::First(e), r)),
        };

        let (item1, rest) = match self.parser1.parse(rest) {
            Ok(t) => t,
            Err((e, r)) => return Err((Either::Last(e), r)),
        };

        Ok(((item0, item1), rest))
    }
}
