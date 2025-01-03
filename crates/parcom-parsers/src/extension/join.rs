use parcom_base::Either;
use parcom_core::{ParseResult::*, Parser, ParserResult, RewindStream};
use std::marker::PhantomData;

pub struct Join<S: RewindStream, P0: Parser<S>, P1: Parser<S>> {
    pub(super) parser0: P0,
    pub(super) parser1: P1,
    pub(super) marker: PhantomData<S>,
}

impl<S: RewindStream, P0: Parser<S>, P1: Parser<S>> Parser<S> for Join<S, P0, P1> {
    type Output = (P0::Output, P1::Output);
    type Error = Either<P0::Error, P1::Error>;
    type Fault = Either<P0::Fault, P1::Fault>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let (item0, rest) = match self.parser0.parse(input).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::First(e), r),
            Fatal(e, r) => return Fatal(Either::First(e), r),
        };

        let (item1, rest) = match self.parser1.parse(rest).await {
            Done(v, r) => (v, r),
            Fail(e, r) => return Fail(Either::Last(e), r),
            Fatal(e, r) => return Fatal(Either::Last(e), r),
        };

        Done((item0, item1), rest)
    }
}
