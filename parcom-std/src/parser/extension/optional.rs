use parcom_core::{Never, ParseResult::*, Parser, ParserResult, RewindStream};
use std::marker::PhantomData;

pub struct Optional<T: RewindStream, P: Parser<T>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P: Parser<S>> Parser<S> for Optional<S, P> {
    type Output = Result<P::Output, P::Error>;
    type Error = Never;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();
        match self.parser.parse(input) {
            Done(v, r) => Done(Ok(v), r),
            Fail(e, r) => Done(Err(e), r.rewind(anchor)),
            Fatal(e) => Fatal(e),
        }
    }
}
