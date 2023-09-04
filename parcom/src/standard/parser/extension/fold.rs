use std::marker::PhantomData;

use crate::{Parser, RewindStream};

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
    type Output = A;
    type Error = ();

    fn parse(&self, input: S) -> crate::ParseResult<S, Self::Output, Self::Error> {
        let (mut acc, mut fold) = (self.init)();

        let mut anchor = input.anchor();
        let mut rest = input;
        loop {
            match self.parser.parse(rest) {
                Ok((v, r)) => {
                    anchor = r.anchor();
                    rest = r;
                    acc = fold(acc, v);
                }
                Err((_, r)) => {
                    break Ok((acc, r.rewind(anchor)));
                }
            }
        }
    }
}
