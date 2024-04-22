use parcom_base::Either;
use parcom_core::{ParseResult::*, Parser, ParserResult, RewindStream};
use std::marker::PhantomData;

pub struct Or<T: RewindStream, P0: Parser<T>, P1: Parser<T>> {
    pub(super) parser0: P0,
    pub(super) parser1: P1,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P0: Parser<S>, P1: Parser<S>> Parser<S> for Or<S, P0, P1> {
    type Output = Either<P0::Output, P1::Output>;
    type Error = (P0::Error, P1::Error);
    type Fault = Either<P0::Fault, P1::Fault>;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();

        let (err0, rest) = match self.parser0.parse(input) {
            Done(v, r) => return Done(Either::First(v), r),
            Fail(e, r) => (e, r),
            Fatal(e, r) => return Fatal(Either::First(e), r),
        };
        let input = rest.rewind(anchor);

        let (err1, rest) = match self.parser1.parse(input) {
            Done(v, r) => return Done(Either::Last(v), r),
            Fail(e, r) => (e, r),
            Fatal(e, r) => return Fatal(Either::Last(e), r),
        };

        Fail((err0, err1), rest)
    }
}
