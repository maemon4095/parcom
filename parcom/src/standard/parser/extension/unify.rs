use std::marker::PhantomData;

use crate::{standard::Either, ParseResult::*, Parser, ParserResult};

pub struct Unify<S, T, P: Parser<S, Output = Either<T, T>>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, T, P: Parser<S, Output = Either<T, T>>> Parser<S> for Unify<S, T, P> {
    type Output = T;
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input) {
            Done(Either::First(v), r) => Done(v, r),
            Done(Either::Last(v), r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(e),
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
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input) {
            Done(v, r) => Done(v, r),
            Fail(Either::First(v), r) => Fail(v, r),
            Fail(Either::Last(v), r) => Fail(v, r),
            Fatal(e) => Fatal(e),
        }
    }
}

pub struct UnifyFault<S, T, P: Parser<S, Fault = Either<T, T>>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, T, P: Parser<S, Fault = Either<T, T>>> Parser<S> for UnifyFault<S, T, P> {
    type Output = P::Output;
    type Error = P::Error;
    type Fault = T;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input) {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
            Fatal(Either::First(e)) => Fatal(e),
            Fatal(Either::Last(e)) => Fatal(e),
        }
    }
}
