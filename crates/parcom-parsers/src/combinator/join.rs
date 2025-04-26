use parcom_base::Either;
use parcom_core::{ParseResult::*, Parser, ParserOnce, ParserResult, RewindStream};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Join<S: RewindStream, P0: ParserOnce<S>, P1: ParserOnce<S>> {
    parser0: P0,
    parser1: P1,
    marker: PhantomData<S>,
}

impl<S: RewindStream, P0: ParserOnce<S>, P1: ParserOnce<S>> Join<S, P0, P1> {
    pub fn new(parser0: P0, parser1: P1) -> Self {
        Self {
            parser0,
            parser1,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P0: ParserOnce<S>, P1: ParserOnce<S>> ParserOnce<S> for Join<S, P0, P1> {
    type Output = (P0::Output, P1::Output);
    type Error = Either<P0::Error, P1::Error>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        let (item0, rest) = match self.parser0.parse_once(input).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::First(e), r),
            StreamError(e, r) => return StreamError(e, r),
        };

        let (item1, rest) = match self.parser1.parse_once(rest).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::Last(e), r),
            StreamError(e, r) => return StreamError(e, r),
        };

        Done((item0, item1), rest)
    }
}

impl<S: RewindStream, P0: Parser<S>, P1: Parser<S>> Parser<S> for Join<S, P0, P1> {
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let (item0, rest) = match self.parser0.parse(input).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::First(e), r),
            StreamError(e, r) => return StreamError(e, r),
        };

        let (item1, rest) = match self.parser1.parse(rest).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::Last(e), r),
            StreamError(e, r) => return StreamError(e, r),
        };

        Done((item0, item1), rest)
    }
}
