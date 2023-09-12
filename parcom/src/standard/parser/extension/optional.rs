use std::marker::PhantomData;

use crate::{Never, ParseResult::*, Parser, ParserResult, RewindStream};

pub struct Optional<T: RewindStream, P: Parser<T>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P: Parser<S>> Parser<S> for Optional<S, P> {
    type Output = Option<P::Output>;
    type Error = Never;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();
        match self.parser.parse(input) {
            Done(v, r) => Done(Some(v), r),
            Fail(_, r) => Done(None, r.rewind(anchor)),
            Fatal(e) => Fatal(e),
        }
    }
}
