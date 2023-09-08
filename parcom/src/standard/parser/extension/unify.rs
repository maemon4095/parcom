use std::marker::PhantomData;

use crate::{
    standard::Either,
    ParseResult::{self, *},
    Parser,
};

pub struct Unify<S, T, P: Parser<S, Output = Either<T, T>>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, T, P: Parser<S, Output = Either<T, T>>> Parser<S> for Unify<S, T, P> {
    type Output = T;
    type Error = P::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Done(Either::First(v), r) => Done(v, r),
            Done(Either::Last(v), r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
        }
    }
}

pub struct UnifyErr<S, T, P: Parser<S, Error = Either<T, T>>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, T, P: Parser<S, Error = Either<T, T>>> Parser<S> for UnifyErr<S, T, P> {
    type Output = P::Output;
    type Error = T;

    fn parse(&self, input: S) -> parcom_core::ParseResult<S, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Done(v, r) => Done(v, r),
            Fail(Either::First(v), r) => Fail(v, r),
            Fail(Either::Last(v), r) => Fail(v, r),
        }
    }
}
