use parcom_core::{ParseError, ParseResult::*, Parser, ParserResult, RewindStream};
use std::marker::PhantomData;

pub struct Optional<T: RewindStream, P: Parser<T>> {
    parser: P,
    marker: PhantomData<T>,
}

impl<T: RewindStream, P: Parser<T>> Optional<T, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P: Parser<S>> Parser<S> for Optional<S, P> {
    type Output = Result<P::Output, P::Error>;
    type Error = P::Error;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();
        match self.parser.parse(input).await {
            Done(v, r) => Done(Ok(v), r),
            Fail(e, r) if !e.should_terminate() => Done(Err(e), r.rewind(anchor).await),
            Fail(e, r) => Fail(e, r),
        }
    }
}
