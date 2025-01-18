use parcom_core::{IterativeParser, IterativeParserState, ParseResult::*, Parser};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Collect<S, P: IterativeParser<S>, C: Extend<P::Output> + Default> {
    parser: P,
    marker: PhantomData<(S, C)>,
}

impl<S, P: IterativeParser<S>, C: Extend<P::Output> + Default> Collect<S, P, C> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S, P: IterativeParser<S>, C: Extend<P::Output> + Default> Parser<S> for Collect<S, P, C> {
    type Output = (C, P::Error);
    type Error = P::Error;

    async fn parse(&self, input: S) -> parcom_core::ParseResult<S, Self::Output, Self::Error> {
        let mut state = self.parser.start();
        let mut collection = C::default();
        let mut rest = input;
        loop {
            match state.parse_next(rest).await {
                Done(Ok(v), r) => {
                    rest = r;
                    collection.extend(std::iter::once(v));
                }
                Done(Err(e), r) => {
                    return Done((collection, e), r);
                }
                Fail(e, r) => return Fail(e, r),
            }
        }
    }
}
