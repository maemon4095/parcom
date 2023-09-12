use std::marker::PhantomData;

use crate::{Never, ParseResult::*, Parser, ParserResult, ShouldNever};

pub struct NeverFault<S, P: Parser<S>> {
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
            Fatal(e) => Fatal(e.never()),
        }
    }
}
