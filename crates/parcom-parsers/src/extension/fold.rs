use std::marker::PhantomData;

use parcom_base::Reason;
use parcom_core::{ParseError, ParseResult::*, Parser, ParserResult, RewindStream};

pub struct Fold<S, P, A, FInit, FBody>
where
    S: RewindStream,
    P: Parser<S>,
    FInit: Fn() -> (A, FBody),
    FBody: FnMut(A, P::Output) -> A,
{
    parser: P,
    init: FInit,
    marker: PhantomData<(S, A)>,
}

impl<S, P, A, FInit, FBody> Fold<S, P, A, FInit, FBody>
where
    S: RewindStream,
    P: Parser<S>,
    FInit: Fn() -> (A, FBody),
    FBody: FnMut(A, P::Output) -> A,
{
    pub fn new(parser: P, init: FInit) -> Self {
        Self {
            parser,
            init,
            marker: PhantomData,
        }
    }
}

impl<S, P, A, FInit, FBody> Parser<S> for Fold<S, P, A, FInit, FBody>
where
    S: RewindStream,
    P: Parser<S>,
    FInit: Fn() -> (A, FBody),
    FBody: FnMut(A, P::Output) -> A,
{
    type Output = (A, Reason<P::Error>);
    type Error = P::Error;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let (mut acc, mut fold) = (self.init)();

        let mut anchor = input.anchor();
        let mut rest = input;
        loop {
            match self.parser.parse(rest).await {
                Done(v, r) => {
                    anchor = r.anchor();
                    rest = r;
                    acc = fold(acc, v);
                }
                Fail(e, r) if !e.should_terminate() => {
                    break Done((acc, Reason(e)), r.rewind(anchor).await);
                }
                Fail(e, r) => break Fail(e, r),
            }
        }
    }
}
