use parcom_core::{Never, ParseResult::*, Parser, ParserResult, ShouldNever};
use std::marker::PhantomData;

pub struct NeverFault<S, P>
where
    P: Parser<S>,
    P::Fault: ShouldNever,
{
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, P> Parser<S> for NeverFault<S, P>
where
    P: Parser<S>,
    P::Fault: ShouldNever,
{
    type Output = P::Output;
    type Error = P::Error;
    type Fault = Never;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input) {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => e.never(),
        }
    }
}
