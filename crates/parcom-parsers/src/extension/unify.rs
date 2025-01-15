use parcom_base::Either;
use parcom_core::{ParseError, ParseResult::*, Parser, ParserResult};
use std::marker::PhantomData;

pub struct Unify<S, T0, T1, T, P>
where
    P: Parser<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    parser: P,
    marker: PhantomData<(S, T)>,
}

impl<S, T0, T1, T, P> Unify<S, T0, T1, T, P>
where
    P: Parser<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    pub(super) fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S, T0, T1, T, P> Parser<S> for Unify<S, T0, T1, T, P>
where
    P: Parser<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    type Output = T;
    type Error = P::Error;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input).await {
            Done(e, r) => Done(e.unify(), r),
            Fail(e, r) => Fail(e, r),
        }
    }
}

pub struct UnifyErr<S, T0, T1, T, P>
where
    P: Parser<S, Error = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    parser: P,
    marker: PhantomData<(S, T)>,
}

impl<S, T0, T1, T, P> UnifyErr<S, T0, T1, T, P>
where
    P: Parser<S, Error = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    pub(super) fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S, T0, T1, T, P> Parser<S> for UnifyErr<S, T0, T1, T, P>
where
    P: Parser<S, Error = Either<T0, T1>>,
    T0: Into<T> + ParseError,
    T1: Into<T> + ParseError,
    T: ParseError,
{
    type Output = P::Output;
    type Error = T;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input).await {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e.unify(), r),
        }
    }
}
