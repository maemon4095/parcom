use parcom_core::{ParseError, Parser, ParserOnce, ParserResult, Sequence};
use parcom_util::{done, Either};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Unify<S, T0, T1, T, P>
where
    S: Sequence,
    P: ParserOnce<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    parser: P,
    marker: PhantomData<(S, T)>,
}

impl<S, T0, T1, T, P> Unify<S, T0, T1, T, P>
where
    S: Sequence,
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
    S: Sequence,
    P: ParserOnce<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    type Output = T;
    type Error = P::Error;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse_once(input).await? {
            (Either::First(v), r) => done(v.into(), r),
            (Either::Last(v), r) => done(v.into(), r),
        }
    }
}

impl<S, T0, T1, T, P> Parser<S> for Unify<S, T0, T1, T, P>
where
    S: Sequence,
    P: Parser<S, Output = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input).await? {
            (e, r) => done(e.unify(), r),
        }
    }
}

pub struct UnifyErr<S, T0, T1, T, P>
where
    S: Sequence,
    P: ParserOnce<S, Error = Either<T0, T1>>,
    T0: Into<T>,
    T1: Into<T>,
{
    parser: P,
    marker: PhantomData<(S, T)>,
}

impl<S, T0, T1, T, P> UnifyErr<S, T0, T1, T, P>
where
    S: Sequence,
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
    S: Sequence,
    P: ParserOnce<S, Error = Either<T0, T1>>,
    T0: Into<T> + ParseError,
    T1: Into<T> + ParseError,
    T: ParseError,
{
    type Output = P::Output;
    type Error = T;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parser.parse_once(input).await.map_err(|e| {
            e.map_fail(|e| match e {
                Either::First(e) => e.into(),
                Either::Last(e) => e.into(),
            })
        })
    }
}

impl<S, T0, T1, T, P> Parser<S> for UnifyErr<S, T0, T1, T, P>
where
    S: Sequence,
    P: Parser<S, Error = Either<T0, T1>>,
    T0: Into<T> + ParseError,
    T1: Into<T> + ParseError,
    T: ParseError,
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser
            .parse(input)
            .await
            .map_err(|e| e.map_fail(|e| e.unify()))
    }
}
