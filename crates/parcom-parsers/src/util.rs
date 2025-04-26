use parcom_core::{Parser, ParserOnce, Stream};
use std::marker::PhantomData;

pub struct Boxed<S: Stream, P: ParserOnce<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S: Stream, P: ParserOnce<S>> Boxed<S, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S: Stream, P: Parser<S>> Parser<S> for Boxed<S, P> {
    fn parse(
        &self,
        input: S,
    ) -> impl std::future::Future<Output = parcom_core::ParseResult<S, Self::Output, Self::Error>>
    {
        Box::pin(self.parser.parse(input))
    }
}

impl<S: Stream, P: ParserOnce<S>> ParserOnce<S> for Boxed<S, P> {
    type Output = P::Output;
    type Error = P::Error;

    fn parse_once(
        self,
        input: S,
    ) -> impl std::future::Future<Output = parcom_core::ParseResult<S, Self::Output, Self::Error>>
    {
        self.parser.parse_once(input)
    }
}
