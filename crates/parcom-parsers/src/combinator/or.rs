use parcom_base::{Either, EitherBoth};
use parcom_core::{ParseError, ParseResult::*, Parser, ParserOnce, ParserResult, RewindStream};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Or<T: RewindStream, P0: ParserOnce<T>, P1: ParserOnce<T>> {
    parser0: P0,
    parser1: P1,
    marker: PhantomData<T>,
}

impl<T: RewindStream, P0: ParserOnce<T>, P1: ParserOnce<T>> Or<T, P0, P1> {
    pub fn new(parser0: P0, parser1: P1) -> Self {
        Self {
            parser0,
            parser1,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P0: ParserOnce<S>, P1: ParserOnce<S>> ParserOnce<S> for Or<S, P0, P1> {
    type Output = Either<P0::Output, P1::Output>;
    type Error = EitherBoth<P0::Error, P1::Error>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();

        let (err0, rest) = match self.parser0.parse_once(input).await {
            Done(v, r) => return Done(Either::First(v), r),
            Fail(e, r) if !e.should_terminate() => (e, r),
            Fail(e, r) => return Fail(EitherBoth::First(e), r),
            StreamErr(e, r) => return StreamErr(e, r),
        };
        let input = rest.rewind(anchor).await;

        let (err1, rest) = match self.parser1.parse_once(input).await {
            Done(v, r) => return Done(Either::Last(v), r),
            Fail(e, r) if !e.should_terminate() => (e, r),
            Fail(e, r) => return Fail(EitherBoth::Last(e), r),
            StreamErr(e, r) => return StreamErr(e, r),
        };

        Fail(EitherBoth::Both(err0, err1), rest)
    }
}

impl<S: RewindStream, P0: Parser<S>, P1: Parser<S>> Parser<S> for Or<S, P0, P1> {
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();

        let (err0, rest) = match self.parser0.parse(input).await {
            Done(v, r) => return Done(Either::First(v), r),
            Fail(e, r) if !e.should_terminate() => (e, r),
            Fail(e, r) => return Fail(EitherBoth::First(e), r),
            StreamErr(e, r) => return StreamErr(e, r),
        };

        let input = rest.rewind(anchor).await;

        let (err1, rest) = match self.parser1.parse(input).await {
            Done(v, r) => return Done(Either::Last(v), r),
            Fail(e, r) if !e.should_terminate() => (e, r),
            Fail(e, r) => return Fail(EitherBoth::Last(e), r),
            StreamErr(e, r) => return StreamErr(e, r),
        };

        Fail(EitherBoth::Both(err0, err1), rest)
    }
}
