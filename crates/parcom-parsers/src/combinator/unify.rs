use parcom_base::Either;
use parcom_core::{ParseError, ParseResult::*, Parser, ParserOnce, ParserResult, Stream};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Unify<S, T0, T1, T, P>
where
    S: Stream,
    P: ParserOnce<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    parser: P,
    marker: PhantomData<(S, T)>,
}

impl<S, T0, T1, T, P> Unify<S, T0, T1, T, P>
where
    S: Stream,
    P: ParserOnce<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S, T0, T1, T, P> ParserOnce<S> for Unify<S, T0, T1, T, P>
where
    S: Stream,
    P: ParserOnce<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    type Output = T;
    type Error = P::Error;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse_once(input).await {
            Done(Either::First(v), r) => Done(v.into(), r),
            Done(Either::Last(v), r) => Done(v.into(), r),
            Fail(e, r) => Fail(e, r),
            StreamError(e, r) => StreamError(e, r),
        }
    }
}

impl<S, T0, T1, T, P> Parser<S> for Unify<S, T0, T1, T, P>
where
    S: Stream,
    P: Parser<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input).await {
            Done(e, r) => Done(e.unify(), r),
            Fail(e, r) => Fail(e, r),
            StreamError(e, r) => StreamError(e, r),
        }
    }
}

pub struct UnifyErr<S, T0, T1, T, P>
where
    S: Stream,
    P: ParserOnce<S, Error = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    parser: P,
    marker: PhantomData<(S, T)>,
}

impl<S, T0, T1, T, P> UnifyErr<S, T0, T1, T, P>
where
    S: Stream,
    P: ParserOnce<S, Error = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S, T0, T1, T, P> ParserOnce<S> for UnifyErr<S, T0, T1, T, P>
where
    S: Stream,
    P: ParserOnce<S, Error = Either<T0, T1>>,
    T0: Into<T> + ParseError,
    T1: Into<T> + ParseError,
    T: ParseError,
{
    type Output = P::Output;
    type Error = T;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse_once(input).await {
            Done(v, r) => Done(v, r),
            Fail(Either::First(e), r) => Fail(e.into(), r),
            Fail(Either::Last(e), r) => Fail(e.into(), r),
            StreamError(e, r) => StreamError(e, r),
        }
    }
}

impl<S, T0, T1, T, P> Parser<S> for UnifyErr<S, T0, T1, T, P>
where
    S: Stream,
    P: Parser<S, Error = Either<T0, T1>>,
    T0: Into<T> + ParseError,
    T1: Into<T> + ParseError,
    T: ParseError,
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input).await {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e.unify(), r),
            StreamError(e, r) => StreamError(e, r),
        }
    }
}
