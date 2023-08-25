use std::marker::PhantomData;

use crate::Parser;

use super::Either;

pub struct Join<T, P0: Parser<T>, P1: Parser<T>> {
    pub(super) parser0: P0,
    pub(super) parser1: P1,
    pub(super) marker: PhantomData<T>,
}

impl<T, P0: Parser<T>, P1: Parser<T>> Parser<T> for Join<T, P0, P1> {
    type Output = (P0::Output, P1::Output);
    type Error = Either<P0::Error, P1::Error>;

    fn parse<S: crate::ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)> {
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
