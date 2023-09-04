use std::marker::PhantomData;

use crate::{standard::Either, Parser};

pub struct Unify<S, T, P: Parser<S, Output = Either<T, T>>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, T, P: Parser<S, Output = Either<T, T>>> Parser<S> for Unify<S, T, P> {
    type Output = T;
    type Error = P::Error;

    fn parse(&self, input: S) -> parcom_core::ParseResult<S, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok((Either::First(v), r)) => Ok((v, r)),
            Ok((Either::Last(v), r)) => Ok((v, r)),
            Err(t) => Err(t),
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
            Ok(t) => Ok(t),
            Err((Either::First(v), r)) => Err((v, r)),
            Err((Either::Last(v), r)) => Err((v, r)),
        }
    }
}
