use std::marker::PhantomData;

use crate::Reason;
use parcom_core::{Never, ParseResult::*, Parser, ParserResult, RewindStream};

pub struct Fold<S, P: Parser<S>, A, FInit: Fn() -> (A, FBody), FBody: FnMut(A, P::Output) -> A> {
    pub(super) parser: P,
    pub(super) init: FInit,
    pub(super) marker: PhantomData<(S, A)>,
}

impl<S: RewindStream, P, A, FInit, FBody> Parser<S> for Fold<S, P, A, FInit, FBody>
where
    P: Parser<S>,
    FInit: Fn() -> (A, FBody),
    FBody: FnMut(A, P::Output) -> A,
{
    type Output = (A, Reason<P::Error>);
    type Error = Never;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let (mut acc, mut fold) = (self.init)();

        let mut anchor = input.anchor();
        let mut rest = input;
        loop {
            match self.parser.parse(rest) {
                Done(v, r) => {
                    anchor = r.anchor();
                    rest = r;
                    acc = fold(acc, v);
                }
                Fail(e, r) => {
                    break Done((acc, Reason(e)), r.rewind(anchor));
                }
                Fatal(e) => break Fatal(e),
            }
        }
    }
}
